use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::DepositMsg;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::GenericBalance;
use angel_core::utils::validate_deposit_fund;

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, WasmMsg, WasmQuery,
};
use cosmwasm_std::{from_slice, to_vec};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg};
use crate::state::{Config, OldConfig, CONFIG, GIFT_CARDS};

// version info for migration info
const CONTRACT_NAME: &str = "gift-cards";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        },
    )?;
    Ok(Response::default())
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let cw20_fund = Asset {
        info: AssetInfoBase::Cw20(deps.api.addr_validate(info.sender.as_str())?),
        amount: cw20_msg.amount,
    };
    match from_binary(&cw20_msg.msg) {
        Ok(ReceiveMsg::TopUp { to_address }) => execute_topup(deps, info, to_address, cw20_fund),
        _ => Err(ContractError::InvalidInputs {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::TopUp { to_address } => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            execute_topup(deps, info, to_address, native_fund)
        }
        ExecuteMsg::Spend { asset, deposit_msg } => execute_spend(deps, info, asset, deposit_msg),
        ExecuteMsg::UpdateConfig {
            owner,
            registrar_contract,
        } => execute_update_config(deps, info, owner, registrar_contract),
    }
}

pub fn execute_topup(
    deps: DepsMut,
    _info: MessageInfo,
    to_address: String,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if the passed token is in "accepted_tokens"
    let topup_token =
        validate_deposit_fund(deps.as_ref(), config.registrar_contract.as_str(), fund)?;

    // make sure it's a non-zero amount
    if topup_token.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // try to load to_address user's card balance. Create a new balance if it does not exist.
    let to_addr = deps.api.addr_validate(&to_address)?;
    let mut card = GIFT_CARDS
        .load(deps.storage, to_addr.clone())
        .unwrap_or(GenericBalance::default());

    // add tokens to the user's Gift Card balance
    match topup_token.info {
        AssetInfoBase::Native(ref denom) => card.add_tokens(Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: topup_token.amount,
        }])),
        AssetInfoBase::Cw20(contract_addr) => card.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: topup_token.amount,
        })),
        _ => unreachable!(),
    };

    // save modified card balance
    GIFT_CARDS.save(deps.storage, to_addr, &card)?;
    Ok(Response::default().add_attribute("action", "topup"))
}

pub fn execute_spend(
    deps: DepsMut,
    info: MessageInfo,
    fund: Asset,
    deposit_msg: DepositMsg,
) -> Result<Response, ContractError> {
    // try to load msg sender's card balance. Throws an error if not found.
    let mut card = GIFT_CARDS.load(deps.storage, info.sender.clone())?;

    // check that the asset is in the user's balance
    let spendable = match fund.info.clone() {
        AssetInfoBase::Native(ref denom) => card.get_denom_amount(denom.to_string()),
        AssetInfoBase::Cw20(contract_addr) => card.get_token_amount(contract_addr),
        _ => unreachable!(),
    };

    // available balance is not zero & is enough to meet requested amount
    if spendable.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }
    if spendable.amount < fund.amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // deduct_tokens from user's balance
    match fund.info.clone() {
        AssetInfoBase::Native(ref denom) => card.deduct_tokens(Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: fund.amount,
        }])),
        AssetInfoBase::Cw20(contract_addr) => card.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: fund.amount,
        })),
        _ => unreachable!(),
    };

    // save modified card balance
    GIFT_CARDS.save(deps.storage, info.sender, &card)?;

    // build deposit msg to desired Accounts contract Endowment
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let message = match &fund.info {
        AssetInfoBase::Native(ref denom) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registrar_config.accounts_contract.unwrap().to_string(),
            msg: to_binary(&deposit_msg).unwrap(),
            funds: vec![Coin {
                denom: denom.clone(),
                amount: fund.amount,
            }],
        }),
        AssetInfo::Cw20(ref contract_addr) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: registrar_config.accounts_contract.unwrap().to_string(),
                amount: fund.amount,
                msg: to_binary(&deposit_msg).unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
        _ => unreachable!(),
    };

    Ok(Response::default()
        .add_attribute("action", "spend")
        .add_message(message))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    registrar_contract: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // only owner can execute changes to config
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = match owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => config.owner,
    };
    config.registrar_contract = match registrar_contract {
        Some(contract) => deps.api.addr_validate(&contract)?,
        None => config.registrar_contract,
    };
    // save modified config
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
    }
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<GenericBalance> {
    Ok(GIFT_CARDS
        .load(deps.storage, deps.api.addr_validate(&address).unwrap())
        .unwrap_or(GenericBalance::default()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        }));
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        }));
    }
    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // setup the new config struct and save to storage
    let data = deps
        .storage
        .get("config".as_bytes())
        .ok_or_else(|| StdError::not_found("Config not found"))?;
    let old_config: OldConfig = from_slice(&data)?;
    deps.storage.set(
        "config".as_bytes(),
        &to_vec(&Config {
            owner: old_config.owner,
            registrar_contract: old_config.registrar_contract,
        })?,
    );

    // Remove the "DEPOSITS" & "BALANCES" maps from storage
    deps.storage.remove("deposit".as_bytes());
    deps.storage.remove("balance".as_bytes());

    Ok(Response::default())
}

#[cfg(test)]
mod tests {}
