// use crate::anchor::{epoch_state, Cw20HookMsg, HandleMsg};
use crate::config;
use crate::config::{
    PendingDepositInfo, PendingRedemptionInfo, PendingWithdrawInfo, BALANCES, PENDING_DEPOSITS,
    PENDING_REDEMPTIONS, PENDING_WITHDRAWS, TOKEN_INFO,
};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{AccountTransferMsg, AccountWithdrawMsg};
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentListResponse,
};
use angel_core::structs::{BalanceInfo, EndowmentEntry};
use angel_core::utils::{deduct_tax, ratio_adjusted_balance};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Order, QueryRequest,
    Response, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};

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
    env: Env,
    info: MessageInfo,
    msg: AccountTransferMsg,
    balance: Balance,
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
            denom: config.input_denom.clone(),
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

    let mut pending_deposits = PENDING_DEPOSITS
        .load(deps.storage, &info.sender)
        .unwrap_or(vec![]);
    pending_deposits.push(PendingDepositInfo {
        id: Uint128::zero(),
        locked: after_taxes_locked,
        liquid: after_taxes_liquid,
    });
    PENDING_DEPOSITS.save(deps.storage, &info.sender, &pending_deposits)?;

    // FAKE DEPOSIT TOKEN INCREASES HERE:
    // Should only be done after a Successful Reply from Anchor SubMsg
    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(BalanceInfo::default());
    investment
        .locked_balance
        .add_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: after_taxes_locked,
            address: env.contract.address.clone(),
        }));
    investment
        .liquid_balance
        .add_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: after_taxes_liquid,
            address: env.contract.address,
        }));
    BALANCES.save(deps.storage, &info.sender, &investment)?;

    Ok(
        Response::new()
            .add_attribute("action", "deposit")
            .add_attribute("sender", info.sender.clone())
            .add_attribute("deposit_amount", info.funds[0].amount)
            .add_attribute("mint_amount", after_taxes.amount), // .add_message(
                                                               // CosmosMsg::Wasm(WasmMsg::Execute {
                                                               //     contract_addr: config.moneymarket.to_string(),
                                                               //     msg: to_binary(&HandleMsg::DepositStable {})?,
                                                               //     funds: vec![after_taxes.clone()],
                                                               // }),
                                                               // ])
    )
}

/// Redeem Stable: Take in an amount of locked/liquid deposit tokens
/// to redeem from the vault for UST to send back to the the Accounts SC
pub fn redeem_stable(
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
            msg: to_binary(&RegistrarQueryMsg::ApprovedEndowmentList {})?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments
        .iter()
        .position(|p| p.address == info.sender.clone());
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // let epoch_state = epoch_state(deps.as_ref(), &config.moneymarket)?;
    // let exchange_rate = Decimal::percent(100); // epoch_state.exchange_rate;

    // update investment holdings balances
    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(BalanceInfo::default());

    investment
        .locked_balance
        .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: msg.locked,
            address: env.contract.address.clone(),
        }));

    investment
        .liquid_balance
        .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: msg.liquid,
            address: env.contract.address,
        }));

    let mut pending_redemptions = PENDING_REDEMPTIONS
        .load(deps.storage, &info.sender)
        .unwrap_or(vec![]);
    pending_redemptions.push(PendingRedemptionInfo {
        id: Uint128::zero(),
        account_address: info.sender.clone(),
        locked: msg.locked,
        liquid: msg.locked,
    });
    PENDING_REDEMPTIONS.save(deps.storage, &info.sender, &pending_redemptions)?;

    Ok(Response::new()
        .add_attribute("action", "redeem")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("redeem_amount", msg.locked + msg.locked)
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
        // TO DO: Move this VaultReceipt msg to be send after a success reply from Anchor!
        .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: info.sender.to_string(),
            msg: to_binary(&&angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                AccountTransferMsg {
                    locked: msg.locked,
                    liquid: msg.locked,
                },
            ))?,
            funds: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: "uusd".to_string(),
                    amount: msg.locked + msg.locked,
                },
            )?],
        }))))
}

/// Withdraw Stable: Takes in an amount of locked/liquid deposit tokens
/// to withdraw from the vault for UST to send back to a beneficiary
pub fn withdraw_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
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

    // reduce the total supply of CW20 deposit tokens
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply -= msg.locked + msg.liquid;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // update investment holdings balances
    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(BalanceInfo::default());

    investment
        .locked_balance
        .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: msg.locked,
            address: env.contract.address.clone(),
        }));

    investment
        .liquid_balance
        .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: msg.liquid,
            address: env.contract.address,
        }));

    let mut pending_withdraws = PENDING_WITHDRAWS
        .load(deps.storage, &info.sender)
        .unwrap_or(vec![]);
    pending_withdraws.push(PendingWithdrawInfo {
        id: Uint128::zero(),
        beneficiary: msg.beneficiary.clone(),
        locked: msg.locked,
        liquid: msg.locked,
    });
    PENDING_WITHDRAWS.save(deps.storage, &info.sender, &pending_withdraws)?;

    Ok(Response::new()
        .add_attribute("action", "redeem")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("withdraw_amount", msg.locked + msg.locked)
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
        // TO DO: Reply from Anchor with UST redeemed should trigger sending back to beneficiary
        .add_submessage(SubMsg::new(BankMsg::Send {
            to_address: msg.beneficiary.to_string(),
            amount: vec![Coin {
                amount: msg.locked + msg.liquid,
                denom: "uusd".to_string(),
            }],
        })))
}

pub fn harvest(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // pull registrar SC config to fetch: 1) Tax Rate and 2) Treasury Addr
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::Config {})?,
        }))?;

    let mut treasury_account = BALANCES
        .load(
            deps.storage,
            &deps.api.addr_validate(&registrar_config.treasury)?,
        )
        .unwrap_or(BalanceInfo::default());
    let accounts: Result<Vec<_>, _> = BALANCES
        .keys(deps.storage, None, None, Order::Ascending)
        .map(String::from_utf8)
        .collect();

    let mut deposit_token = Cw20CoinVerified {
        address: env.contract.address.clone(),
        amount: Uint128::zero(),
    };
    let mut taxes_collected = Uint128::zero();
    for account in accounts.unwrap().iter() {
        let account_address = deps.api.addr_validate(account)?;
        let mut balances = BALANCES.load(deps.storage, &account_address)?;
        let locked_deposit_amount: Uint128 = balances
            .locked_balance
            .cw20_list()
            .iter()
            .filter(|token| token.address == env.contract.address.clone())
            .next()
            .unwrap_or(&Cw20Coin {
                amount: Uint128::zero(),
                address: env.contract.address.to_string(),
            })
            .clone()
            .amount;

        // proceed to shuffle balances if we have a non-zero amount
        if locked_deposit_amount > Uint128::zero() {
            let transfer_amt = locked_deposit_amount * registrar_config.tax_rate;
            let taxes_owed = transfer_amt * registrar_config.tax_rate;

            // lower locked balance
            deposit_token.amount = transfer_amt.clone();
            balances
                .locked_balance
                .deduct_tokens(Balance::Cw20(deposit_token.clone()));

            // add to liquid balance (less taxes owed to AP Treasury)
            deposit_token.amount = transfer_amt - taxes_owed;
            balances
                .liquid_balance
                .add_tokens(Balance::Cw20(deposit_token.clone()));
            taxes_collected += taxes_owed;

            // add taxes collected to the liquid balance of the AP Treasury
            deposit_token.amount = taxes_owed.clone();
            treasury_account
                .liquid_balance
                .add_tokens(Balance::Cw20(deposit_token.clone()));

            BALANCES.save(deps.storage, &account_address, &balances)?;
        }
    }
    BALANCES.save(
        deps.storage,
        &deps.api.addr_validate(&registrar_config.treasury)?,
        &treasury_account,
    )?;

    Ok(Response::new().add_attribute("action", "transfer"))
}
