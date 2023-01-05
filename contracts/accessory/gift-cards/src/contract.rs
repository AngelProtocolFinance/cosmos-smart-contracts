use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::GenericBalance;
use angel_core::utils::validate_deposit_fund;

use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, QueryRequest, Response, StdError, StdResult, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg};
use crate::state::{Config, Deposit, BALANCES, CONFIG, DEPOSITS};

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
            keeper: deps.api.addr_validate(&msg.keeper)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            next_deposit: 1_u64,
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
        Ok(ReceiveMsg::Deposit { to_address }) => {
            let sender = deps.api.addr_validate(&cw20_msg.sender)?;
            execute_deposit(deps, sender, to_address, cw20_fund)
        }
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
        ExecuteMsg::Deposit { to_address } => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            execute_deposit(deps, info.sender, to_address, native_fund)
        }
        ExecuteMsg::Claim { deposit, recipient } => execute_claim(deps, info, deposit, recipient),
        ExecuteMsg::Spend {
            asset,
            endow_id,
            locked_percentage,
            liquid_percentage,
        } => execute_spend(
            deps,
            info,
            asset,
            endow_id,
            locked_percentage,
            liquid_percentage,
        ),
        ExecuteMsg::UpdateConfig {
            owner,
            keeper,
            registrar_contract,
        } => execute_update_config(deps, info, owner, keeper, registrar_contract),
    }
}

pub fn execute_deposit(
    deps: DepsMut,
    sender: Addr,
    to_address: Option<String>,
    fund: Asset,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Check if the passed token is in "accepted_tokens"
    let deposit_token = validate_deposit_fund(
        deps.as_ref(),
        config.registrar_contract.as_str(),
        fund.clone(),
    )?;

    // make sure it's a non-zero amount
    if deposit_token.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut deposit = Deposit {
        sender: sender,
        token: fund,
        claimed: false,
    };

    if to_address != None {
        let to_addr = deps.api.addr_validate(&to_address.unwrap())?;

        // deposit should be marked as claimed immediately
        deposit.claimed = true;

        // try to load to_address recipient's balance. Create a new, empty balance if it does not exist.
        let mut bal = BALANCES
            .load(deps.storage, to_addr.clone())
            .unwrap_or(GenericBalance::default());

        // add deposited tokens to the recipient's balance
        match deposit_token.info {
            AssetInfoBase::Native(ref denom) => bal.add_tokens(Balance::from(vec![Coin {
                denom: denom.to_string(),
                amount: deposit_token.amount,
            }])),
            AssetInfoBase::Cw20(contract_addr) => bal.add_tokens(Balance::Cw20(Cw20CoinVerified {
                address: contract_addr,
                amount: deposit_token.amount,
            })),
            _ => unreachable!(),
        };

        // save the balance
        BALANCES.save(deps.storage, to_addr.clone(), &bal)?;
    }

    // save the deposit
    DEPOSITS.save(deps.storage, config.next_deposit, &deposit)?;

    // increment next deposit and save
    config.next_deposit += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "deposit")
        .add_attribute("deposit_id", format!("{}", config.next_deposit - 1)))
}

pub fn execute_claim(
    deps: DepsMut,
    info: MessageInfo,
    deposit_id: u64,
    recipient: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the keeper address can carry out claims (for now)
    if info.sender.ne(&config.keeper) {
        return Err(ContractError::Unauthorized {});
    }

    // check that the deposit is still unclaimed
    let mut deposit = DEPOSITS.load(deps.storage, deposit_id)?;
    if deposit.claimed {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Deposit has already been claimed".to_string(),
        }));
    }
    // mark deposit as claimed & save
    deposit.claimed = true;
    DEPOSITS.save(deps.storage, deposit_id, &deposit)?;

    let to_addr = deps.api.addr_validate(&recipient)?;
    // try to load to_address recipient's balance. Create a new, empty balance if it does not exist.
    let mut bal = BALANCES
        .load(deps.storage, to_addr.clone())
        .unwrap_or(GenericBalance::default());

    // add deposited tokens to the recipient's balance
    match deposit.token.info {
        AssetInfoBase::Native(ref denom) => bal.add_tokens(Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: deposit.token.amount,
        }])),
        AssetInfoBase::Cw20(contract_addr) => bal.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: deposit.token.amount,
        })),
        _ => unreachable!(),
    };

    // save the balance
    BALANCES.save(deps.storage, to_addr.clone(), &bal)?;

    Ok(Response::default().add_attribute("action", "claim"))
}

pub fn execute_spend(
    deps: DepsMut,
    info: MessageInfo,
    fund: Asset,
    endow_id: u32,
    locked_percentage: Decimal,
    liquid_percentage: Decimal,
) -> Result<Response, ContractError> {
    // try to load msg sender's balance. Throws an error if not found.
    let mut bal = BALANCES.load(deps.storage, info.sender.clone())?;

    // check that the asset is in the user's balance
    let spendable = match fund.info.clone() {
        AssetInfoBase::Native(ref denom) => bal.get_denom_amount(denom.to_string()),
        AssetInfoBase::Cw20(contract_addr) => bal.get_token_amount(contract_addr),
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
        AssetInfoBase::Native(ref denom) => bal.deduct_tokens(Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: fund.amount,
        }])),
        AssetInfoBase::Cw20(contract_addr) => bal.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: fund.amount,
        })),
        _ => unreachable!(),
    };

    // save modified balance
    BALANCES.save(deps.storage, info.sender, &bal)?;

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
            msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::Deposit(
                angel_core::messages::accounts::DepositMsg {
                    id: endow_id,
                    locked_percentage,
                    liquid_percentage,
                },
            ))
            .unwrap(),
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
                msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::Deposit(
                    angel_core::messages::accounts::DepositMsg {
                        id: endow_id,
                        locked_percentage,
                        liquid_percentage,
                    },
                ))
                .unwrap(),
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
    keeper: Option<String>,
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

    config.keeper = match keeper {
        Some(keeper) => deps.api.addr_validate(&keeper)?,
        None => config.keeper,
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
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Deposit { deposit_id } => to_binary(&query_deposit(deps, deposit_id)?),
    }
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<GenericBalance> {
    Ok(BALANCES
        .load(deps.storage, deps.api.addr_validate(&address).unwrap())
        .unwrap_or(GenericBalance::default()))
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_deposit(deps: Deps, deposit_id: u64) -> StdResult<Deposit> {
    let deposit = DEPOSITS.load(deps.storage, deposit_id)?;
    Ok(deposit)
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

    Ok(Response::default())
}
