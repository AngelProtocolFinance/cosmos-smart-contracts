use crate::anchor::{epoch_state, Cw20HookMsg, HandleMsg};
use crate::config;
use crate::config::{LIQUID_BALANCES, LOCKED_BALANCES, TOKEN_INFO};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::AccountTransferMsg;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentListResponse,
};
use angel_core::structs::EndowmentEntry;
use angel_core::utils::deduct_tax;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order,
    QueryRequest, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;

pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: Addr,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only the registrar contract can update it's address in the config
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed args
    config.registrar_contract = new_registrar;
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

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
    let after_taxes_locked = after_taxes
        .amount
        .clone()
        .multiply_ratio(msg.locked, info.funds[0].amount);
    let after_taxes_liquid = after_taxes
        .amount
        .clone()
        .multiply_ratio(msg.liquid, info.funds[0].amount);

    // update supply
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply += after_taxes.amount;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // add minted amount to Endowment's Locked/Liquid balances
    LOCKED_BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + after_taxes_locked)
        },
    )?;
    LIQUID_BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + after_taxes_liquid)
        },
    )?;

    Ok(Response::new()
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
            // TEMP PLACEHOLDER TO MOVE UUSD DEPOSITS OUT OF VAULT
            CosmosMsg::Bank(BankMsg::Send {
                to_address: config.owner.to_string(),
                amount: vec![Coin {
                    amount: after_taxes.amount.clone(),
                    denom: "uusd".to_string(),
                }],
            }),
        ]))
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
    let exchange_rate = Decimal::percent(100); // epoch_state.exchange_rate;

    let after_taxes = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.input_denom,
            amount: info.funds[0].amount,
        },
    )?;
    let after_taxes_locked = after_taxes
        .amount
        .clone()
        .multiply_ratio(msg.locked, info.funds[0].amount);
    let after_taxes_liquid = after_taxes
        .amount
        .clone()
        .multiply_ratio(msg.liquid, info.funds[0].amount);

    // lower balance
    LOCKED_BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance
                .unwrap_or_default()
                .checked_sub(after_taxes_locked)?)
        },
    )?;
    LIQUID_BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance
                .unwrap_or_default()
                .checked_sub(after_taxes_liquid)?)
        },
    )?;

    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(after_taxes.amount)?;
        Ok(info)
    })?;

    Ok(Response::new()
        .add_attribute("action", "redeem")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("deposit_amount", info.funds[0].amount)
        .add_attribute("mint_amount", after_taxes.amount)
        // .add_submessage(SubMsg {
        //     id: 0,
        //     msg: CosmosMsg::Wasm(WasmMsg::Execute {
        //         contract_addr: config.yield_token.to_string(),
        //         msg: to_binary(&Cw20ExecuteMsg::Send {
        //             contract: config.moneymarket.to_string(),
        //             amount: after_taxes.amount * Decimal::from(exchange_rate),
        //             msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
        //         })?,
        //         funds: vec![],
        //     }),
        //     gas_limit: None,
        //     reply_on: ReplyOn::Success,
        // })
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: info.sender.to_string(),
            msg: to_binary(&&angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                AccountTransferMsg {
                    locked: Uint256::from(after_taxes_locked),
                    liquid: Uint256::from(after_taxes_liquid),
                },
            ))?,
            funds: vec![Coin {
                denom: "uusd".to_string(),
                amount: after_taxes.amount,
            }],
        })))
}

pub fn harvest(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let taxes_collected = Uint128::zero();

    let locked_accounts: Result<Vec<_>, _> = LOCKED_BALANCES
        .keys(deps.storage, None, None, Order::Ascending)
        .map(String::from_utf8)
        .collect();

    for account in locked_accounts.unwrap().iter() {
        let account_address = deps.api.addr_validate(account)?;
        let transfer_amt =
            LOCKED_BALANCES.load(deps.storage, &account_address)? * Decimal::percent(1);
        let taxes_owed = transfer_amt * Decimal::percent(2);

        // lower locked balance
        LOCKED_BALANCES.update(
            deps.storage,
            &account_address,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(transfer_amt)?)
            },
        )?;
        // add to liquid balance (less taxes owed to AP Treasury)
        LIQUID_BALANCES.update(
            deps.storage,
            &account_address,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + transfer_amt - taxes_owed)
            },
        )?;

        taxes_collected += taxes_owed;
    }

    // add taxes collected to the liquid balance of the AP Treasury
    LIQUID_BALANCES.update(
        deps.storage,
        &config.owner,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + taxes_collected)
        },
    )?;

    Ok(Response::new().add_attribute("action", "transfer"))
}
