// use crate::anchor::{epoch_state, Cw20HookMsg, HandleMsg};
use crate::config;
use crate::config::{BALANCES, TOKEN_INFO};
use crate::utils::deduct_tax;
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::AccountTransferMsg;
use angel_core::responses::registrar::EndowmentListResponse;
use angel_core::structs::EndowmentEntry;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, Uint128, WasmMsg, WasmQuery,
};
// use cw20::Cw20ExecuteMsg;

pub fn deposit_stable(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    // check that the depositor is an approved Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::ApprovedEndowmentList {})?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments.iter().position(|p| p.address == info.sender);
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

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

    // update supply
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply += after_taxes.amount;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // add minted amount to recipient balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + after_taxes.amount)
        },
    )?;

    let res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("deposit_amount", info.funds[0].amount)
        .add_attribute("mint_amount", after_taxes.amount)
        .add_messages(vec![
            // CosmosMsg::Wasm(WasmMsg::Execute {
            //     contract_addr: config.moneymarket.to_string(),
            //     msg: to_binary(&HandleMsg::DepositStable {})?,
            //     funds: vec![after_taxes.clone()],
            // }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: info.sender.to_string(),
                msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                    AccountTransferMsg {
                        locked: Uint256::from(after_tax_locked),
                        liquid: Uint256::from(after_tax_liquid),
                    },
                ))?,
                funds: vec![Coin {
                    denom: "PDTv1".to_string(),
                    amount: after_taxes.amount,
                }],
            }),
        ]);

    Ok(res)
}

pub fn redeem_stable(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    // check that the depositor is an approved Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::ApprovedEndowmentList {})?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments.iter().position(|p| p.address == info.sender);
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // let epoch_state = epoch_state(deps.as_ref(), &config.moneymarket)?;
    let _exchange_rate = Decimal::percent(95); // epoch_state.exchange_rate;

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

    // lower balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance
                .unwrap_or_default()
                .checked_sub(after_taxes.amount)?)
        },
    )?;

    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(after_taxes.amount)?;
        Ok(info)
    })?;

    let res = Response::new()
        .add_attribute("action", "redeem")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("deposit_amount", info.funds[0].amount)
        .add_attribute("mint_amount", after_taxes.amount)
        .add_messages(vec![
            // CosmosMsg::Wasm(WasmMsg::Execute {
            //     contract_addr: config.yield_token.to_string(),
            //     msg: to_binary(&Cw20ExecuteMsg::Send {
            //         contract: config.moneymarket.to_string(),
            //         amount: info.funds[0].amount * Decimal::from(exchange_rate),
            //         msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
            //     })?,
            //     funds: vec![],
            // }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: info.sender.to_string(),
                msg: to_binary(&&angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                    AccountTransferMsg {
                        locked: Uint256::from(after_tax_locked),
                        liquid: Uint256::from(after_tax_liquid),
                    },
                ))?,
                funds: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: after_taxes.amount,
                }],
            }),
        ]);

    Ok(res)
}
