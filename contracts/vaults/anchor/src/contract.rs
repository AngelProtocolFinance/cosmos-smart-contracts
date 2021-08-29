use crate::anchor;
use crate::config;
use crate::executers;
use crate::msg::{InitMsg, MigrateMsg};
use crate::queriers;
use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{ExecuteMsg, QueryMsg};
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};

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
    // let anchor_config = anchor::config(deps.as_ref(), &moneymarket)?;

    let config = config::Config {
        owner: info.sender,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        moneymarket,
        input_denom: "uusd".to_string(), // anchor_config.stable_denom.clone(),
        yield_token: deps.api.addr_validate(&msg.registrar_contract)?, // deps.api.addr_validate(&anchor_config.aterra_contract)?,
    };

    config::store(deps.storage, &config)?;

    // create initial accounts
    let total_supply = Uint128::zero();

    // store token info
    let token_info = config::TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        mint: None,
        total_supply,
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
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::Deposit(msg) => executers::deposit_stable(deps, env, info, msg), // UST -> DP (Account)
        ExecuteMsg::Redeem(msg) => executers::redeem_stable(deps, env, info, msg), // DP -> UST (Account)
        ExecuteMsg::Harvest {} => executers::harvest(deps, env, info), // DP -> DP shuffle (taxes collected)
    }
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
        QueryMsg::ExchangeRate { input_denom: _ } => {
            // let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;

            to_binary(&ExchangeRateResponse {
                exchange_rate: Decimal256::percent(95), // epoch_state.exchange_rate,
                yield_token_supply: Uint256::from(42069u64), // epoch_state.aterra_supply,
            })
        }
        QueryMsg::Deposit { amount } => to_binary(&anchor::deposit_stable_msg(
            deps,
            &config.moneymarket,
            &config.input_denom,
            amount.into(),
        )?),
        QueryMsg::Redeem { amount } => to_binary(&anchor::redeem_stable_msg(
            deps,
            &config.moneymarket,
            &config.yield_token,
            amount.into(),
        )?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
