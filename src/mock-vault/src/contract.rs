use crate::errors::ContractError;
use crate::executers;
use crate::msg::{
    deposit_stable_msg, redeem_stable_msg, ConfigResponse, ExchangeRateResponse, ExecuteMsg,
    InstantiateMsg, MigrateMsg, QueryMsg,
};
use crate::queriers;
use crate::state::{Config, TokenInfo, CONFIG, TOKEN_INFO};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, Uint256,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Balance;

// version info for future migration info
const CONTRACT_NAME: &str = "mock-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            moneymarket: deps.api.addr_validate(&msg.moneymarket)?,
            input_denom: msg.input_denom.clone(),
            yield_token: deps.api.addr_validate(&msg.yield_token)?,
            next_pending_id: 0,
            tax_per_block: msg.tax_per_block,
            harvest_to_liquid: msg.harvest_to_liquid,
        },
    )?;

    TOKEN_INFO.save(
        deps.storage,
        &TokenInfo {
            name: msg.name,
            symbol: msg.symbol.clone(),
            decimals: msg.decimals,
            mint: None,
            total_supply: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_attribute("register_vault", msg.symbol))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { new_owner } => executers::update_owner(deps, info, new_owner),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        // -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Deposit {} => {
            executers::deposit_stable(deps, env, info.clone(), Balance::from(info.funds))
        }
        // Redeem is only called by the SC when setting up new strategies.
        // Pulls all existing strategy amounts back to Account in UST.
        // Then re-Deposits according to the Strategies set.
        // -Deposit Token/Yield Token (Vault) --> +UST (Account) --> -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Redeem { account_addr } => {
            executers::redeem_stable(deps, env, info, account_addr)
        } // -Deposit Token/Yield Token (Account) --> +UST (outside beneficiary)
        ExecuteMsg::Withdraw(msg) => executers::withdraw_stable(deps, env, info, msg), // DP (Account Locked) -> DP (Account Liquid + Treasury Tax)
        ExecuteMsg::Harvest {
            last_earnings_harvest,
            last_harvest_fx,
        } => executers::harvest(deps, env, info, last_earnings_harvest, last_harvest_fx), // DP -> DP shuffle (taxes collected)
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    match msg {
        QueryMsg::VaultConfig {} => to_binary(&queriers::query_vault_config(deps)),
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            input_denom: config.input_denom.clone(),
            yield_token: config.yield_token.to_string(),
        }),
        QueryMsg::Balance { address } => to_binary(&queriers::query_balance(deps, address)),
        QueryMsg::TokenInfo {} => to_binary(&queriers::query_token_info(deps)),
        // ANCHOR-SPECIFIC QUERIES BELOW THIS POINT!
        QueryMsg::ExchangeRate { input_denom: _ } => to_binary(&ExchangeRateResponse {
            exchange_rate: Decimal256::one(),
            yield_token_supply: Uint256::zero(),
        }),
        QueryMsg::Deposit { amount } => to_binary(&deposit_stable_msg(
            &config.moneymarket,
            &config.input_denom,
            amount,
        )?),
        QueryMsg::Redeem { amount } => to_binary(&redeem_stable_msg(
            &config.moneymarket,
            &config.yield_token,
            amount,
        )?),
        _ => unimplemented!(),
    }
}

#[entry_point]
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
