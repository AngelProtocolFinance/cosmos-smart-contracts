use crate::anchor;
use crate::anchor::register_deposit_token;
use crate::config;
use crate::msg::{InitMsg, MigrateMsg};
use angel_portals::error::ContractError;
use angel_portals::portal_msg::{ExecuteMsg, QueryMsg};
use angel_portals::portal_rsp::{ConfigResponse, ExchangeRateResponse};
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw20::MinterResponse;
use terraswap::token::InstantiateMsg as Cw20InitMsg;

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    let moneymarket = deps.api.addr_validate(&msg.moneymarket)?;
    let anchor_config = anchor::config(deps.as_ref(), &moneymarket)?;

    let config = config::Config {
        owner: info.sender.clone(),
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        deposit_token: info.sender,
        moneymarket,
        input_denom: anchor_config.stable_denom.clone(),
        yield_token: deps.api.addr_validate(&anchor_config.aterra_contract)?,
    };

    config::store(deps.storage, &config)?;

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.deposit_token_code_id, // terraswap docs have wasm code for each network
        admin: Some(env.contract.address.to_string()),
        label: "new portal deposit token".to_string(),
        funds: vec![],
        msg: to_binary(&Cw20InitMsg {
            name: "Angel Protocol - Portal Deposit Token - Anchor".to_string(),
            symbol: "PDTv1".to_string(),
            decimals: 6u8,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: env.contract.address.to_string(),
                cap: None,
            }),
        })?,
    };

    Ok(Response::new()
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&angel_core::registrar_msg::PortalAddMsg {
                    address: env.contract.address.to_string(),
                })
                .unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        })
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(wasm_msg),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }))
}

pub fn handle(_deps: DepsMut, _env: Env, _msg: ExecuteMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

// Replies back from the CW20 Deposit Token Init calls to a portal SC should
// be caught and handled to register the newly created Deposit Token Addr
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => register_deposit_token(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    match msg {
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            input_denom: config.input_denom.clone(),
            yield_token: config.yield_token.clone().to_string(),
            deposit_token: config.deposit_token.clone().to_string(),
        }),
        QueryMsg::ExchangeRate { input_denom: _ } => {
            let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;

            to_binary(&ExchangeRateResponse {
                exchange_rate: epoch_state.exchange_rate.clone(),
                yield_token_supply: epoch_state.aterra_supply.clone(),
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

pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
