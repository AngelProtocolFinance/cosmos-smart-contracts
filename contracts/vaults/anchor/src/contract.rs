use crate::anchor;
use crate::config;
use crate::executers;
use crate::msg::{InitMsg, MigrateMsg};
use crate::queriers;
use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{ExecuteMsg, QueryMsg};
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
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

    let config = config::Config {
        owner: info.sender,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        moneymarket,
        input_denom: anchor_config.stable_denom.clone(),
        yield_token: deps.api.addr_validate(&anchor_config.aterra_contract)?,
        next_pending_id: 0,
        tax_per_block: msg.tax_per_block,
        last_harvest: env.block.height,
    };

    config::store(deps.storage, &config)?;

    // store token info
    let token_info = config::TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        mint: None,
        total_supply: Uint128::zero(),
    };
    config::TOKEN_INFO.save(deps.storage, &token_info)?;

    Ok(Response::new()
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&angel_core::messages::registrar::ExecuteMsg::VaultAdd(
                    angel_core::messages::registrar::VaultAddMsg {
                        vault_addr: env.contract.address.to_string(),
                        input_denom: config.input_denom,
                        yield_token: config.yield_token.to_string(),
                    },
                ))
                .unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        })
        .add_attribute("register_vault", token_info.symbol))
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
        ExecuteMsg::Deposit(msg) => {
            executers::deposit_stable(deps, env, info.clone(), msg, Balance::from(info.funds))
        }
        // Redeem is only called by the SC when setting up new strategies.
        // Pulls all existing strategy amounts back to Account in UST.
        // Then re-Deposits according to the Strategies set.
        // -Deposit Token/Yield Token (Vault) --> +UST (Account) --> -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Redeem {} => executers::redeem_stable(deps, env, info), // -Deposit Token/Yield Token (Account) --> +UST (outside beneficiary)
        ExecuteMsg::Withdraw(msg) => executers::withdraw_stable(deps, env, info, msg), // DP (Account Locked) -> DP (Account Liquid + Treasury Tax)
        ExecuteMsg::Harvest {} => executers::harvest(deps, env, info), // DP -> DP shuffle (taxes collected)
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
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            input_denom: config.input_denom.clone(),
            yield_token: config.yield_token.to_string(),
        }),
        QueryMsg::Balance { address } => to_binary(&queriers::query_balance(deps, address)),
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
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
