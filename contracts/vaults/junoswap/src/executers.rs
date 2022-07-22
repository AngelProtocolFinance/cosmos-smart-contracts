use crate::config::{Config, PendingInfo, BALANCES, PENDING, TOKEN_INFO};
use crate::wasmswap::{swap_msg, InfoResponse};
use crate::{config, wasmswap};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{AccountWithdrawMsg, ExecuteMsg, UpdateConfigMsg};
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentListResponse,
};
use angel_core::structs::{BalanceInfo, EndowmentEntry};
use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    Order, QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128,
    WasmMsg, WasmQuery,
};
use cw20::{Balance, Denom};

// wallet that we use for regular, automated harvests of vault
const CRON_WALLET: &str = "terra1janh9rs6pme3tdwhyag2lmsr2xv6wzhcrjz0xx";

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only the owner/admin of the contract can update their address in the configs
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed args
    config.owner = new_owner;
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

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

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.pool_addr = match msg.swap_pool_addr {
        Some(ref addr) => deps.api.addr_validate(&addr)?,
        None => config.pool_addr,
    };

    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(&config.pool_addr, &wasmswap::QueryMsg::Info {})?;

    config.pool_lp_token_addr = deps.api.addr_validate(&swap_pool_info.lp_token_address)?;
    config.input_denoms = vec![swap_pool_info.token1_denom, swap_pool_info.token2_denom];
    config.staking_addr = match msg.staking_addr {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.staking_addr,
    };

    config.harvest_to_liquid = msg.harvest_to_liquid.unwrap_or(config.harvest_to_liquid);

    // Add more addresses to `config.routes`
    for addr in msg.routes.add {
        if !config.routes.contains(&addr) {
            config.routes.push(addr);
        }
    }

    // Remove the addresses from `config.routes`
    for addr in msg.routes.remove {
        if config.routes.contains(&addr) {
            let id = config
                .routes
                .iter()
                .enumerate()
                .find(|(_, v)| **v == addr)
                .unwrap()
                .0;
            config.routes.swap_remove(id);
        }
    }

    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deposit_denom: Denom,
    deposit_amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    if !config.input_denoms.contains(&deposit_denom) {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    if deposit_amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // check that the depositor is an Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::EndowmentList {
                name: None,
                owner: None,
                status: None,
                tier: None,
                un_sdg: None,
                endow_type: None,
            })?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments.iter().position(|p| p.address == info.sender);
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", deposit_amount);
    // let submessage_id = config.next_pending_id;
    // PENDING.save(
    //     deps.storage,
    //     &submessage_id.to_be_bytes(),
    //     &PendingInfo {
    //         typ: "deposit".to_string(),
    //         accounts_address: info.sender.clone(),
    //         beneficiary: None,
    //         fund: None,
    //         locked: asset.amount,
    //         liquid: Uint128::zero(),
    //     },
    // )?;
    // config.next_pending_id += 1;
    // config::store(deps.storage, &config)?;

    // let msgs = swap_msg(&config, asset)?;

    // if msgs.len() == 1 {
    //     res = res.add_submessage(SubMsg {
    //         id: submessage_id,
    //         msg: msgs[0].clone(),
    //         reply_on: ReplyOn::Always,
    //         gas_limit: None,
    //     });
    // } else {
    //     res = res.add_submessage(SubMsg::new(msgs[0].clone()));
    //     res = res.add_submessage(SubMsg {
    //         id: submessage_id,
    //         msg: msgs[1].clone(),
    //         reply_on: ReplyOn::Always,
    //         gas_limit: None,
    //     });
    // }

    // 1. Add the "swap" message
    res = res.add_messages(swap_msg(&config, &deposit_denom, deposit_amount)?);

    // 2. Add the "add_liquidity" message
    let in_denom = deposit_denom;
    let out_denom = if in_denom == config.input_denoms[0] {
        config.input_denoms[1].clone()
    } else {
        config.input_denoms[0].clone()
    };
    let in_denom_bal_before =
        query_denom_balance(&deps, &in_denom, env.contract.address.to_string());
    let out_denom_bal_before =
        query_denom_balance(&deps, &out_denom, env.contract.address.to_string());
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::AddLiquidity {
            in_denom,
            out_denom,
            in_denom_bal_before,
            out_denom_bal_before,
        })
        .unwrap(),
        funds: vec![],
    }));

    //

    // 3rd message: (handle_reply_lp_token) + stake_lp_tokens

    Ok(res)
}

/// Redeem Stable: Take in an amount of locked/liquid deposit tokens
/// to redeem from the vault for stablecoins to send back to the the Accounts SC
pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    account_addr: Addr,
) -> Result<Response, ContractError> {
    // let mut config = config::read(deps.storage)?;

    // // check that the depositor is a Registered Accounts SC
    // let endowments_rsp: EndowmentListResponse =
    //     deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
    //         contract_addr: config.registrar_contract.to_string(),
    //         msg: to_binary(&RegistrarQueryMsg::EndowmentList {
    //             name: None,
    //             owner: None,
    //             status: None,
    //             tier: None,
    //             un_sdg: None,
    //             endow_type: None,
    //         })?,
    //     }))?;
    // let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    // let pos = endowments
    //     .iter()
    //     .position(|p| p.address == info.sender.clone());

    // // reject if the sender was found not in the list of endowments
    // // OR if the sender is not the Registrar SC (ie. we're closing the endowment)
    // if pos == None && info.sender != config.registrar_contract {
    //     return Err(ContractError::Unauthorized {});
    // }

    // // use arg account_addr to lookup Balances
    // let mut investment = BALANCES
    //     .load(deps.storage, &account_addr)
    //     .unwrap_or_else(|_| BalanceInfo::default());

    // // grab total tokens for locked and liquid balances
    // let total_redemption = investment.get_token_amount(env.contract.address.clone());

    // // reduce the total supply of CW20 deposit tokens by redemption amount
    // let mut token_info = TOKEN_INFO.load(deps.storage)?;
    // token_info.total_supply -= total_redemption;
    // TOKEN_INFO.save(deps.storage, &token_info)?;

    // // update investment holdings balances to zero
    // investment.set_token_balances(Balance::Cw20(Cw20CoinVerified {
    //     amount: Uint128::zero(),
    //     address: env.contract.address,
    // }));

    // BALANCES.save(deps.storage, &account_addr, &investment)?;

    // let submessage_id = config.next_pending_id;
    // PENDING.save(
    //     deps.storage,
    //     &submessage_id.to_be_bytes(),
    //     &PendingInfo {
    //         typ: "redeem".to_string(),
    //         accounts_address: account_addr,
    //         beneficiary: None,
    //         fund: None,
    //         amount: total_redemption,
    //     },
    // )?;
    // config.next_pending_id += 1;
    // config::store(deps.storage, &config)?;

    // Ok(Response::new()
    //     .add_attribute("action", "redeem_from_vault")
    //     .add_attribute("sender", info.sender)
    //     .add_attribute("redeem_amount", total_redemption)
    //     .add_submessage(SubMsg {
    //         id: submessage_id,
    //         msg: redeem_stable_msg(&config.moneymarket, &config.yield_token, total_redemption)?,
    //         gas_limit: None,
    //         reply_on: ReplyOn::Always,
    //     }))
    Ok(Response::default())
}

/// Withdraw Stable: Takes in an amount of locked/liquid deposit tokens
/// to withdraw from the vault for UST to send back to a beneficiary
pub fn withdraw_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    //     let mut config = config::read(deps.storage)?;

    //     // check that the tx sender is an Accounts SC
    //     let endowments_rsp: EndowmentListResponse =
    //         deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
    //             contract_addr: config.registrar_contract.to_string(),
    //             msg: to_binary(&RegistrarQueryMsg::EndowmentList {
    //                 name: None,
    //                 owner: None,
    //                 status: None,
    //                 tier: None,
    //                 un_sdg: None,
    //                 endow_type: None,
    //             })?,
    //         }))?;
    //     let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    //     let pos = endowments.iter().position(|p| p.address == info.sender);

    //     // reject if the sender was found not in the list of endowments
    //     if pos == None {
    //         return Err(ContractError::Unauthorized {});
    //     }

    //     // reduce the total supply of CW20 deposit tokens
    //     let withdraw_total = msg.amount;
    //     let mut token_info = TOKEN_INFO.load(deps.storage)?;
    //     token_info.total_supply -= withdraw_total;
    //     TOKEN_INFO.save(deps.storage, &token_info)?;

    //     let mut investment = BALANCES
    //         .load(deps.storage, &info.sender)
    //         .unwrap_or_else(|_| BalanceInfo::default());

    //     // check the account has enough balance to cover the withdraw
    //     let balance = investment.get_token_amount(env.contract.address.clone());
    //     if balance < msg.amount {
    //         return Err(ContractError::CannotExceedCap {});
    //     }

    //     // update investment holdings balances
    //     investment.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
    //         amount: msg.amount,
    //         address: env.contract.address.clone(),
    //     }));
    //     BALANCES.save(deps.storage, &info.sender, &investment)?;

    //     let submessage_id = config.next_pending_id;
    //     PENDING.save(
    //         deps.storage,
    //         &submessage_id.to_be_bytes(),
    //         &PendingInfo {
    //             typ: "withdraw".to_string(),
    //             accounts_address: info.sender.clone(),
    //             beneficiary: Some(msg.beneficiary.clone()),
    //             fund: None,
    //             amount: msg.amount,
    //         },
    //     )?;
    //     config.next_pending_id += 1;
    //     config::store(deps.storage, &config)?;

    //     Ok(Response::new()
    //         .add_attribute("action", "withdraw_from_vault")
    //         .add_attribute("sender", info.sender)
    //         .add_attribute("withdraw_amount", withdraw_total)
    //         .add_submessage(SubMsg {
    //             id: submessage_id,
    //             msg: redeem_stable_msg(&config.moneymarket, &config.yield_token, withdraw_total)?,
    //             gas_limit: None,
    //             reply_on: ReplyOn::Always,
    //         }))
    Ok(Response::default())
}

pub fn harvest(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collector_address: String,
    collector_share: Decimal,
) -> Result<Response, ContractError> {
    //     let mut config = config::read(deps.storage)?;

    //     if info.sender != config.registrar_contract && info.sender.to_string() != *CRON_WALLET {
    //         return Err(ContractError::Unauthorized {});
    //     }

    //     let curr_epoch = anchor::epoch_state(deps.as_ref(), &config.moneymarket)?;

    //     let harvest_earn_rate = Decimal::from(
    //         (curr_epoch.exchange_rate - config.last_harvest_fx.unwrap_or(curr_epoch.exchange_rate))
    //             / config.last_harvest_fx.unwrap_or(curr_epoch.exchange_rate),
    //     );

    //     config.last_harvest = env.block.height;
    //     config.last_harvest_fx = Some(curr_epoch.exchange_rate);
    //     config::store(deps.storage, &config)?;

    //     // pull registrar SC config to fetch the Treasury Tax Rate
    //     let registrar_config: RegistrarConfigResponse =
    //         deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
    //             contract_addr: config.registrar_contract.to_string(),
    //             msg: to_binary(&RegistrarQueryMsg::Config {})?,
    //         }))?;
    //     let treasury_addr = deps.api.addr_validate(&registrar_config.treasury)?;
    //     let collector_addr = deps.api.addr_validate(&collector_address)?;
    //     let mut harvest_account = BalanceInfo::default();

    //     // iterate over all accounts and shuffle DP tokens from Locked to Liquid
    //     // set aside a small amount for treasury
    //     let accounts: Result<Vec<_>, _> = BALANCES
    //         .keys(deps.storage, None, None, Order::Ascending)
    //         .map(String::from_utf8)
    //         .collect();
    //     for account in accounts.unwrap().iter() {
    //         let account_address = deps.api.addr_validate(account)?;
    //         let mut balances = BALANCES
    //             .load(deps.storage, &account_address)
    //             .unwrap_or_else(|_| BalanceInfo::default());

    //         // CALCULATE ALL AMOUNTS TO BE COLLECTED FOR TAXES AND TO BE TRANSFERED
    //         // UPFRONT BEFORE PERFORMING ANY ACTUAL BALANCE SHUFFLES
    //         // calculate harvest taxes owed on liquid balance earnings
    //         let taxes_owed = balances.get_token_amount(env.contract.address.clone())
    //             * harvest_earn_rate
    //             * registrar_config.tax_rate;

    //         // calulate amount of earnings to be harvested from locked >> liquid balance
    //         // reduce harvest amount by any locked taxes owed on those earnings
    //         let transfer_amt = balances.get_token_amount(env.contract.address.clone())
    //             * harvest_earn_rate
    //             * config.harvest_to_liquid
    //             - taxes_owed;

    //         // deduct liquid taxes if we have a non-zero amount
    //         if taxes_owed > Uint128::zero() {
    //             let deposit_token = Cw20CoinVerified {
    //                 address: env.contract.address.clone(),
    //                 amount: taxes_owed,
    //             };
    //             // lower liquid balance
    //             balances.deduct_tokens(Balance::Cw20(deposit_token.clone()));

    //             // add taxes collected to the liquid balance of the Collector
    //             harvest_account.add_tokens(Balance::Cw20(deposit_token.clone()));
    //         }

    //         // proceed to shuffle balances if we have a non-zero amount
    //         if transfer_amt > Uint128::zero() {
    //             let deposit_token = Cw20CoinVerified {
    //                 address: env.contract.address.clone(),
    //                 amount: transfer_amt,
    //             };

    //             // lower balance
    //             balances.deduct_tokens(Balance::Cw20(deposit_token.clone()));

    //             // TO DO: add a msg send to endowment the transfer amnt here
    //         }

    //         BALANCES.save(deps.storage, &account_address, &balances)?;
    //     }

    //     if harvest_account.get_token_amount(env.contract.address.clone()) > Uint128::zero() {
    //         // Withdraw all DP Tokens from Treasury and send to Collector Contract and/or the AP Treasury Wallet
    //         let withdraw_total = harvest_account.get_token_amount(env.contract.address);
    //         let mut withdraw_leftover = withdraw_total;

    //         let mut res = Response::new()
    //             .add_attribute("action", "harvest_redeem_from_vault")
    //             .add_attribute("sender", info.sender)
    //             .add_attribute("withdraw_amount", withdraw_total);

    //         // Harvested Amount is split by collector split input percentage
    //         if !collector_share.is_zero() && collector_share <= Decimal::one() {
    //             let submessage_id = config.next_pending_id;
    //             PENDING.save(
    //                 deps.storage,
    //                 &submessage_id.to_be_bytes(),
    //                 &PendingInfo {
    //                     typ: "withdraw".to_string(),
    //                     accounts_address: collector_addr.clone(),
    //                     beneficiary: Some(collector_addr.clone()),
    //                     fund: None,
    //                     amount: withdraw_total * collector_share,
    //                 },
    //             )?;
    //             withdraw_leftover = withdraw_total - (withdraw_total * collector_share);
    //             config.next_pending_id += 1;
    //             res = res
    //                 .add_attribute("collector_addr", collector_addr)
    //                 .add_submessage(SubMsg {
    //                     id: submessage_id,
    //                     msg: redeem_stable_msg(
    //                         &config.moneymarket,
    //                         &config.yield_token,
    //                         withdraw_total * collector_share,
    //                     )?,
    //                     gas_limit: None,
    //                     reply_on: ReplyOn::Always,
    //                 });
    //         }

    //         // Remainder (if any) is sent to AP Treasury Address
    //         if withdraw_leftover > Uint128::zero() {
    //             let submessage_id = config.next_pending_id;
    //             PENDING.save(
    //                 deps.storage,
    //                 &submessage_id.to_be_bytes(),
    //                 &PendingInfo {
    //                     typ: "withdraw".to_string(),
    //                     accounts_address: treasury_addr.clone(),
    //                     beneficiary: Some(treasury_addr.clone()),
    //                     fund: None,
    //                     amount: withdraw_leftover,
    //                 },
    //             )?;
    //             config.next_pending_id += 1;
    //             config::store(deps.storage, &config)?;
    //             res = res
    //                 .add_attribute("treasury_addr", treasury_addr)
    //                 .add_submessage(SubMsg {
    //                     id: submessage_id,
    //                     msg: redeem_stable_msg(
    //                         &config.junoswap_pool,
    //                         &config.yield_token,
    //                         withdraw_leftover,
    //                     )?,
    //                     gas_limit: None,
    //                     reply_on: ReplyOn::Always,
    //                 });
    //         }
    //         Ok(res)
    //     } else {
    //         Ok(Response::new()
    //             .add_attribute("action", "harvest_redeem_from_vault")
    //             .add_attribute("sender", info.sender))
    //     }
    Ok(Response::default())
}

pub fn process_junoswap_pool_reply(
    deps: DepsMut,
    env: Env,
    id: u64,
    result: SubMsgResult,
) -> Result<Response, ContractError> {
    // pull up the pending transaction details from storage
    let transaction = PENDING.load(deps.storage, &id.to_be_bytes())?;

    // remove this pending transaction
    PENDING.remove(deps.storage, &id.to_be_bytes());

    match result {
        SubMsgResult::Ok(subcall) => {
            // // Grab the Amount returned from Anchor (UST/aUST)
            // let mut anchor_amount = Uint128::zero();
            // for event in subcall.events {
            //     if event.ty == "wasm" {
            //         let deposit_attr: Attribute = Attribute::new("action", "deposit_stable");
            //         if event.attributes.clone().contains(&deposit_attr) {
            //             for attr in event.attributes.clone() {
            //                 if attr.key == "mint_amount" {
            //                     anchor_amount = Uint128::from(attr.value.parse::<u128>().unwrap());
            //                     break;
            //                 }
            //             }
            //         }

            //         let redeem_attr: Attribute = Attribute::new("action", "redeem_stable");
            //         if event.attributes.contains(&redeem_attr) {
            //             for attr in event.attributes {
            //                 if attr.key == "redeem_amount" {
            //                     anchor_amount = Uint128::from(attr.value.parse::<u128>().unwrap());
            //                     break;
            //                 }
            //             }
            //         }
            //     }
            // }

            // // Get the correct Anchor returned amount split by Locked/Liquid ratio in the transaction
            // let res = match transaction.typ.as_str() {
            //     "deposit" => {
            //         // Increase the Account's Deposit token balances by the correct amounts of aUST
            //         let mut investment = BALANCES
            //             .load(deps.storage, &transaction.accounts_address.clone())
            //             .unwrap_or_else(|_| BalanceInfo::default());
            //         investment.add_tokens(Balance::Cw20(Cw20CoinVerified {
            //             amount: anchor_amount,
            //             address: env.contract.address.clone(),
            //         }));
            //         BALANCES.save(deps.storage, &transaction.accounts_address, &investment)?;

            //         // update total token supply by total aUST returned from deposit
            //         let mut token_info = TOKEN_INFO.load(deps.storage)?;
            //         token_info.total_supply += anchor_amount;
            //         TOKEN_INFO.save(deps.storage, &token_info)?;

            //         Response::new()
            //             .add_attribute("action", "anchor_reply_processing")
            //             .add_attribute("mint_amount", anchor_amount)
            //     }
            //     "redeem" => {
            //         Response::new()
            //             .add_attribute("action", "anchor_reply_processing")
            //             // Send UST back to the Account SC via VaultReciept msg
            //             .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            //                 contract_addr: transaction.accounts_address.to_string(),
            //                 msg: to_binary(
            //                     &angel_core::messages::accounts::ExecuteMsg::VaultReceipt {},
            //                 )?,
            //                 funds: vec![Coin {
            //                     amount: anchor_amount,
            //                     denom: DEPOSIT_TOKEN_DENOM.to_string(),
            //                 }],
            //             }))
            //     }
            //     "withdraw" => {
            //         Response::new()
            //             .add_attribute("action", "anchor_reply_processing")
            //             // Send UST to the Beneficiary via BankMsg::Send
            //             .add_message(BankMsg::Send {
            //                 to_address: transaction.beneficiary.unwrap().to_string(),
            //                 amount: vec![Coin {
            //                     amount: anchor_amount,
            //                     denom: DEPOSIT_TOKEN_DENOM.to_string(),
            //                 }],
            //             })
            //     }
            //     &_ => Response::new().add_attribute("action", "anchor_reply_processing"),
            // };

            // // return the response with follow up
            // // messages to beneficiary/Accounts/etc
            // Ok(res)
            Ok(Response::default())
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    in_denom: Denom,
    out_denom: Denom,
    in_denom_bal_before: Uint128,
    out_denom_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    let in_denom_bal = query_denom_balance(&deps, &in_denom, env.contract.address.to_string());
    let out_denom_bal = query_denom_balance(&deps, &out_denom, env.contract.address.to_string());

    let token1_denom: Denom;
    let token2_denom: Denom;
    let token1_amount: Uint128;
    let token2_amount: Uint128;

    if in_denom == config.input_denoms[0] {
        token1_denom = in_denom;
        token2_denom = out_denom;
        token1_amount = in_denom_bal_before - in_denom_bal;
        token2_amount = out_denom_bal - out_denom_bal_before;
    } else {
        token1_denom = out_denom;
        token2_denom = in_denom;
        token1_amount = out_denom_bal - out_denom_bal_before;
        token2_amount = in_denom_bal_before - in_denom_bal;
    }

    let mut funds = vec![];
    let mut msgs = vec![];

    match token1_denom {
        Denom::Native(denom) => funds.push(Coin {
            denom,
            amount: token1_amount,
        }),
        Denom::Cw20(contract_addr) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: config.pool_addr.to_string(),
                amount: token1_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    match token2_denom {
        Denom::Native(denom) => funds.push(Coin {
            denom,
            amount: token2_amount,
        }),
        Denom::Cw20(contract_addr) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: config.pool_addr.to_string(),
                amount: token2_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_addr.to_string(),
        msg: to_binary(&crate::wasmswap::ExecuteMsg::AddLiquidity {
            token1_amount,
            min_liquidity: Uint128::zero(),
            max_token2: token2_amount,
            expiration: None,
        })
        .unwrap(),
        funds,
    }));

    // Add the "stake" message at last
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::Stake {
            lp_token_bal_before: lp_token_bal.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "add_liquidity_to_swap_pool")]))
}

pub fn stake_lp_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let stake_amount = lp_token_bal.balance - lp_token_bal_before;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_lp_token_addr.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.staking_addr.to_string(),
            amount: stake_amount,
            msg: to_binary(&crate::cw20_stake::ReceiveMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![attr("action", "stake_lp_token")]))
}

fn query_denom_balance(deps: &DepsMut, denom: &Denom, account_addr: String) -> Uint128 {
    match denom {
        Denom::Native(denom) => {
            query_balance(&deps, account_addr, denom.to_string()).unwrap_or(Uint128::zero())
        }
        Denom::Cw20(contract_addr) => {
            query_token_balance(&deps, contract_addr.to_string(), account_addr)
                .unwrap_or(Uint128::zero())
        }
    }
}

/// Returns a native token's balance for a specific account.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **account_addr** is an object of type [`Addr`].
///
/// * **denom** is an object of type [`String`] used to specify the denomination used to return the balance (e.g uluna).
pub fn query_balance(deps: &DepsMut, account_addr: String, denom: String) -> StdResult<Uint128> {
    Ok(deps
        .querier
        .query_balance(account_addr, denom)
        .map(|c| c.amount)
        .unwrap_or(Uint128::zero()))
}

/// Returns a token balance for an account.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **contract_addr** is an object of type [`Addr`]. This is the token contract for which we return a balance.
///
/// * **account_addr** is an object of type [`Addr`] for which we query the token balance for.
pub fn query_token_balance(
    deps: &DepsMut,
    contract_addr: String,
    account_addr: String,
) -> StdResult<Uint128> {
    // load balance from the token contract
    let res: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        contract_addr,
        &cw20::Cw20QueryMsg::Balance {
            address: account_addr,
        },
    )?;
    Ok(res.balance)
}
