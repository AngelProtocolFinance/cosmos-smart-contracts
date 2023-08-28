use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20CoinVerified};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, Deposit, GenericBalance, BALANCES, CONFIG, DEPOSITS};

// version info for migration info
const CONTRACT_NAME: &str = "gift-cards";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const AP_CROSSCHAIN_ADDR: &str = "juno1gkay4pd877tagydvcc8uvasxfuvz7nedpstp83"; // AWS GC Transfers

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
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

pub fn execute_claim(
    deps: DepsMut,
    info: MessageInfo,
    deposit_id: u64,
    recipient: String,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;

    // only the keeper address can carry out claims (for now)
    if info.sender.ne(&config.keeper) {
        return Err(StdError::GenericErr {
            msg: "Unauthorized".to_string(),
        });
    }

    // check that the deposit is still unclaimed
    let mut deposit = DEPOSITS.load(deps.storage, deposit_id)?;
    if deposit.claimed {
        return Err(StdError::GenericErr {
            msg: "Deposit has already been claimed".to_string(),
        });
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
) -> Result<Response, StdError> {
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
        return Err(StdError::GenericErr {
            msg: "InvalidZeroAmount".to_string(),
        });
    }
    if spendable.amount < fund.amount {
        return Err(StdError::GenericErr {
            msg: "InsufficientFunds".to_string(),
        });
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

    // Build transfer msg to AP CrossChain Juno multisig wallet to forward to new Polygon Endowments
    let ap_addr: Addr = deps.api.addr_validate(&AP_CROSSCHAIN_ADDR)?;
    let message = match &fund.info {
        AssetInfoBase::Native(ref denom) => CosmosMsg::Bank(BankMsg::Send {
            to_address: ap_addr.to_string(),
            amount: vec![Coin {
                denom: denom.clone(),
                amount: fund.amount,
            }],
        }),
        AssetInfo::Cw20(ref contract_addr) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                recipient: ap_addr.to_string(),
                amount: fund.amount,
            })
            .unwrap(),
            funds: vec![],
        }),
        _ => unreachable!(),
    };

    Ok(Response::default()
        .add_message(message)
        .add_attributes(vec![
            ("action", "spend"),
            ("endow_id", endow_id.to_string().as_str()),
            ("locked_percentage", locked_percentage.to_string().as_str()),
            ("liquid_percentage", liquid_percentage.to_string().as_str()),
        ])
    )
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    keeper: Option<String>,
    registrar_contract: Option<String>,
) -> Result<Response, StdError> {
    let mut config = CONFIG.load(deps.storage)?;
    // only owner can execute changes to config
    if info.sender.ne(&config.owner) {
        return Err(StdError::GenericErr {
            msg: "Unauthorized".to_string(),
        });
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
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, StdError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        });
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        });
    }
    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
