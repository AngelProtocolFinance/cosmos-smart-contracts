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
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order,
    QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
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
                    amount: after_taxes.amount,
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
    let token_info = TOKEN_INFO.load(deps.storage)?;

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

    // Reduce the Endowment's Locked/Liquid balances accordingly
    LOCKED_BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance
                .unwrap_or_default()
                .checked_sub(Uint128::from(msg.locked))?)
        },
    )?;
    LIQUID_BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance
                .unwrap_or_default()
                .checked_sub(Uint128::from(msg.liquid))?)
        },
    )?;

    // let epoch_state = epoch_state(deps.as_ref(), &config.moneymarket)?;
    let exchange_rate = Decimal256::percent(100); // epoch_state.exchange_rate;

    let redeem_locked = msg.locked * exchange_rate;
    let redeem_liquid = msg.liquid * exchange_rate;

    if token_info.total_supply < Uint128::from(redeem_liquid + redeem_locked) {
        let err = format!(
            "lock_req:{},liq_req:{},vault_bal:{}",
            redeem_locked, redeem_liquid, token_info.total_supply
        );
        return Err(ContractError::Std {
            0: StdError::GenericErr { msg: err },
        });
    }

    Ok(Response::new()
        .add_attribute("action", "redeem")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("redeem_amount", redeem_locked + redeem_liquid)
        // .add_submessage(SubMsg {
        //     id: 42,
        //     msg: CosmosMsg::Wasm(WasmMsg::Execute {
        //         contract_addr: config.yield_token.to_string(),
        //         msg: to_binary(&Cw20ExecuteMsg::Send {
        //             contract: config.moneymarket.to_string(),
        //             amount: redeem.amount * Decimal::from(exchange_rate),
        //             msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
        //         })?,
        //         funds: vec![],
        //     }),
        //     gas_limit: None,
        //     reply_on: ReplyOn::Success,
        // })
        // TO DO: Reply from Vault with UST redeemed should trigger the two submessages below ??
        .add_submessage(SubMsg::new(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                amount: Uint128::from(redeem_locked) + Uint128::from(redeem_liquid),
                denom: "uusd".to_string(),
            }],
        }))
        .add_submessage(SubMsg {
            id: 200,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: info.sender.to_string(),
                msg: to_binary(&&angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                    AccountTransferMsg {
                        transfer_id: msg.transfer_id,
                        locked: Uint256::from(redeem_locked),
                        liquid: Uint256::from(redeem_liquid),
                    },
                ))?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }))
}

pub fn harvest(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // pull registrar SC config to fetch the 1) Tax Rate and 2) Treasury Addr
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::Config {})?,
        }))?;

    let mut taxes_collected = Uint128::zero();

    let locked_accounts: Result<Vec<_>, _> = LOCKED_BALANCES
        .keys(deps.storage, None, None, Order::Ascending)
        .map(String::from_utf8)
        .collect();

    for account in locked_accounts.unwrap().iter() {
        let account_address = deps.api.addr_validate(account)?;
        let transfer_amt =
            LOCKED_BALANCES.load(deps.storage, &account_address)? * Decimal::percent(10);
        let taxes_owed = transfer_amt * registrar_config.tax_rate;

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
        &deps.api.addr_validate(&registrar_config.treasury)?,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + taxes_collected)
        },
    )?;

    Ok(Response::new().add_attribute("action", "transfer"))
}
