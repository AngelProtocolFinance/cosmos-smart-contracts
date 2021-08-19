use crate::anchor;
use crate::anchor::register_deposit_token;
use crate::anchor::HandleMsg;
use crate::config;
use crate::msg::{InitMsg, MigrateMsg};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::messages::vault::{AccountTransferMsg, ExecuteMsg, QueryMsg};
use angel_core::responses::registrar::EndowmentListResponse;
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};
use angel_core::structs::EndowmentEntry;
use angel_core::utils::deduct_tax;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply,
    ReplyOn, Response, StdResult, SubMsg, WasmMsg, WasmQuery,
};
use cw20::{Cw20ExecuteMsg, MinterResponse};
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
        label: "vault deposit token".to_string(),
        funds: vec![],
        msg: to_binary(&Cw20InitMsg {
            name: "Angel Protocol - Vault Deposit Token - Anchor".to_string(),
            symbol: "PDTv1".to_string(),
            decimals: 6u8,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: env.contract.address.to_string(),
                cap: None,
            }),
        })?,
    };

    Ok(Response::new().add_submessage(SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(wasm_msg),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit(msg) => deposit_stable(deps, env, info, msg), // UST -> DP (Account)
        ExecuteMsg::Redeem(msg) => redeem_stable(deps, env, info, msg),   // DP -> UST (Account)
    }
}

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    // check that the depositor is an approved Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::ApprovedEndowmentList {})?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments
        .iter()
        .position(|p| p.address.to_string() == info.sender.to_string());
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    let config = config::read(deps.storage)?;
    let after_taxes = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.input_denom,
            amount: info.funds[0].amount,
        },
    )?;

    let after_tax_locked = after_taxes
        .amount
        .clone()
        .multiply_ratio(msg.locked, info.funds[0].amount);
    let after_tax_liquid = after_taxes
        .amount
        .clone()
        .multiply_ratio(msg.liquid, info.funds[0].amount);

    let res = Response::new().add_messages(vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.moneymarket.to_string(),
            msg: to_binary(&HandleMsg::DepositStable {})?,
            funds: vec![after_taxes.clone()],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.deposit_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: env.contract.address.to_string(),
                amount: after_taxes.amount.clone(),
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: info.sender.to_string(),
            msg: to_binary(&AccountTransferMsg {
                locked: Uint256::from(after_tax_locked),
                liquid: Uint256::from(after_tax_liquid),
            })?,
            funds: vec![Coin {
                denom: "PDTv1".to_string(),
                amount: after_taxes.amount.clone(),
            }],
        }),
    ]);

    Ok(res)
}

pub fn redeem_stable(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

// Replies back from the CW20 Deposit Token Init calls to a vault SC should
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
        // DepositAmountOf { account } => ,
        // TotalDepositAmount {} => ,
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
