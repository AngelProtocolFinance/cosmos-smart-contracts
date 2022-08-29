use crate::anchor;
use crate::config;
use crate::executers;
use crate::msg::{InitMsg, MigrateMsg};
use crate::queriers;
use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{ExecuteMsg, QueryMsg};
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};
use angel_core::structs::AccountType;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Balance;

// version info for future migration info
const CONTRACT_NAME: &str = "anchor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let moneymarket = deps.api.addr_validate(&msg.moneymarket)?;
    let anchor_config = anchor::config(deps.as_ref(), &moneymarket)?;
    let sibling_vault = match msg.sibling_vault {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => env.contract.address, // can set later with update_config
    };
    let config = config::Config {
        owner: info.sender,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        acct_type: msg.acct_type,
        sibling_vault,
        moneymarket,
        input_denom: anchor_config.stable_denom.clone(),
        yield_token: deps.api.addr_validate(&anchor_config.aterra_contract)?,
        next_pending_id: 0,
        tax_per_block: msg.tax_per_block,
        last_harvest: env.block.height,
        last_harvest_fx: None,
        harvest_to_liquid: msg.harvest_to_liquid,
    };

    config::store(deps.storage, &config)?;

    // init special AP treasury's token balance (separate from Endowments ID based tracking)
    config::TREASURY_TOKENS.save(deps.storage, &Uint128::zero())?;

    // store token info
    let token_info = config::TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        mint: None,
        total_supply: Uint128::zero(),
    };
    config::TOKEN_INFO.save(deps.storage, &token_info)?;

    Ok(Response::new().add_attribute("register_vault", token_info.symbol))
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
        ExecuteMsg::Deposit { endowment_id } => executers::deposit_stable(
            deps,
            env,
            info.clone(),
            Balance::from(info.funds),
            endowment_id,
        ),
        // Redeem is only called by the SC when setting up new strategies.
        // Pulls all existing strategy amounts back to Account in UST.
        // Then re-Deposits according to the Strategies set.
        // -Deposit Token/Yield Token (Vault) --> +UST (Account) --> -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Redeem {
            endowment_id,
            amount,
        } => executers::redeem_stable(deps, env, info, endowment_id, amount), // -Deposit Token/Yield Token (Account) --> +UST (outside beneficiary)
        // move N assets for an endowment from a vault (if an AccountType::Liquid)
        // over to it's sibling vault (if set and an AccountType::Locked)
        ExecuteMsg::ReinvestToLocked { id, amount } => {
            executers::reinvest_to_locked_execute(deps, env, info, id, amount)
        }
        ExecuteMsg::Withdraw(msg) => executers::withdraw_stable(deps, env, info, msg), // DP (Account Locked) -> DP (Account Liquid + Treasury Tax)
        ExecuteMsg::Harvest {
            collector_address,
            collector_share,
        } => executers::harvest(deps, env, info, collector_address, collector_share), // DP -> DP shuffle (taxes collected)
    }
}

/// Replies back to the Vault from the Anchor MoneyMarket contract:
/// SubMsg IDs are matched back with the PENDING storage to match the
/// incoming and outgoing funds and any further processing steps performed
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    executers::process_anchor_reply(deps, env, msg.id, msg.result)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    match msg {
        QueryMsg::VaultConfig {} => to_binary(&queriers::query_vault_config(deps)),
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            input_denom: config.input_denom.clone(),
            yield_token: config.yield_token.to_string(),
        }),
        QueryMsg::Balance { endowment_id } => {
            to_binary(&queriers::query_balance(deps, endowment_id))
        }
        QueryMsg::TokenInfo {} => to_binary(&queriers::query_token_info(deps)),
        // ANCHOR-SPECIFIC QUERIES BELOW THIS POINT!
        QueryMsg::ExchangeRate { input_denom: _ } => {
            let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;

            to_binary(&ExchangeRateResponse {
                exchange_rate: epoch_state.exchange_rate,
                yield_token_supply: epoch_state.aterra_supply,
            })
        }
        QueryMsg::Deposit { amount } => to_binary(&anchor::deposit_stable_msg(
            &config.moneymarket,
            &config.input_denom,
            amount,
        )?),
        QueryMsg::Redeem { amount } => to_binary(&anchor::redeem_stable_msg(
            &config.moneymarket,
            &config.yield_token,
            amount,
        )?),
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
