// use crate::anchor::{epoch_state, Cw20HookMsg, HandleMsg};
use crate::config;
use crate::config::{BALANCES, TOKEN_INFO};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::AccountTransferMsg;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentListResponse,
};
use angel_core::structs::{BalanceInfo, EndowmentEntry};
use angel_core::utils::deduct_tax;
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, QueryRequest,
    ReplyOn, Response, StdError, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified};

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

    // update investment holdings balances
    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(BalanceInfo::default());
    investment
        .locked_balance
        .add_tokens(Balance::from(vec![Coin {
            denom: config.input_denom.clone(),
            amount: after_taxes_locked,
        }]));
    investment
        .locked_balance
        .add_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: after_taxes_locked,
            address: env.contract.address.clone(),
        }));
    investment
        .locked_balance
        .add_tokens(Balance::from(vec![Coin {
            denom: config.input_denom,
            amount: after_taxes_liquid,
        }]));
    investment
        .locked_balance
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

/// Redeem Stable: Take in an amount of locked/liquid deposit tokens to redeem from the vault for UST
pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    let mut token_info = TOKEN_INFO.load(deps.storage)?;

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

    let redeem_locked = msg.locked * exchange_rate;
    let redeem_liquid = msg.liquid * exchange_rate;

    // update investment holdings balances
    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(BalanceInfo::default());
    investment
        .locked_balance
        .deduct_tokens(Balance::from(vec![Coin {
            denom: config.input_denom.clone(),
            amount: redeem_locked,
        }]));
    investment
        .locked_balance
        .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: redeem_locked,
            address: env.contract.address.clone(),
        }));
    investment
        .liquid_balance
        .deduct_tokens(Balance::from(vec![Coin {
            denom: config.input_denom.clone(),
            amount: redeem_liquid,
        }]));
    investment
        .liquid_balance
        .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: redeem_liquid,
            address: env.contract.address.clone(),
        }));

    if token_info.total_supply < msg.liquid + msg.locked {
        let err = format!(
            "lock_req:{},liq_req:{},vault_bal:{}",
            msg.locked, msg.liquid, token_info.total_supply
        );
        return Err(ContractError::Std {
            0: StdError::GenericErr { msg: err },
        });
    } else {
        token_info.total_supply -= msg.liquid + msg.locked;
    }

    Ok(Response::new()
        .add_attribute("action", "redeem")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("redeem_amount", redeem_locked + redeem_liquid) // .add_submessage(SubMsg {
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
        // .add_submessage(SubMsg::new(BankMsg::Send {
        //     to_address: info.sender.to_string(),
        //     amount: vec![Coin {
        //         amount: redeem_locked + redeem_liquid,
        //         denom: "uusd".to_string(),
        //     }],
        // }))
        .add_submessage(SubMsg {
            id: 200,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: info.sender.to_string(),
                msg: to_binary(&&angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                    AccountTransferMsg {
                        locked: redeem_locked,
                        liquid: redeem_liquid,
                    },
                ))?,
                funds: vec![Coin {
                    amount: Uint128::from(redeem_locked) + redeem_liquid,
                    denom: "uusd".to_string(),
                }],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }))
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
            .unwrap()
            .clone()
            .amount;

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
    BALANCES.save(
        deps.storage,
        &deps.api.addr_validate(&registrar_config.treasury)?,
        &treasury_account,
    )?;

    Ok(Response::new().add_attribute("action", "transfer"))
}
