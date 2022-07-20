use crate::config::{PendingInfo, BALANCES, PENDING, TOKEN_INFO};
use crate::wasmswap::InfoResponse;
use crate::{config, wasmswap};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{AccountWithdrawMsg, UpdateConfigMsg};
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentListResponse,
};
use angel_core::structs::{BalanceInfo, EndowmentEntry};
use cosmwasm_std::{
    to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    Order, QueryRequest, ReplyOn, Response, StdError, SubMsg, SubMsgResult, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified};

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

    config.target = match msg.swap_pool_addr {
        Some(ref addr) => deps.api.addr_validate(&addr)?,
        None => config.target,
    };

    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(&config.target, &wasmswap::QueryMsg::Info {})?;
    config.yield_token = deps.api.addr_validate(&swap_pool_info.lp_token_address)?;
    config.input_denoms = vec![swap_pool_info.token1_denom, swap_pool_info.token2_denom];

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

pub fn deposit_stable(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _balance: Balance,
) -> Result<Response, ContractError> {
    // let mut config = config::read(deps.storage)?;

    // // only accept max of 1 deposit coin/token per donation
    // if info.funds.len() != 1 {
    //     return Err(ContractError::InvalidCoinsDeposited {});
    // }

    // let deposit_amount: Coin = Coin {
    //     denom: DEPOSIT_TOKEN_DENOM.to_string(),
    //     amount: info
    //         .funds
    //         .iter()
    //         .find(|c| {
    //             c.denom == *DEPOSIT_TOKEN_DENOM
    //         })
    //         .map(|c| c.amount)
    //         .unwrap_or_else(Uint128::zero),
    // };

    // if deposit_amount.amount.is_zero() {
    //     return Err(ContractError::EmptyBalance {});
    // }

    // // check that the depositor is an Accounts SC
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
    // let pos = endowments.iter().position(|p| p.address == info.sender);
    // // reject if the sender was found in the list of endowments
    // if pos == None {
    //     return Err(ContractError::Unauthorized {});
    // }

    // let submessage_id = config.next_pending_id;
    // PENDING.save(
    //     deps.storage,
    //     &submessage_id.to_be_bytes(),
    //     &PendingInfo {
    //         typ: "deposit".to_string(),
    //         accounts_address: info.sender.clone(),
    //         beneficiary: None,
    //         fund: None,
    //         amount: deposit_amount.amount,
    //     },
    // )?;
    // config.next_pending_id += 1;
    // config::store(deps.storage, &config)?;

    // Ok(Response::new()
    //     .add_attribute("action", "deposit")
    //     .add_attribute("sender", info.sender)
    //     .add_attribute("deposit_amount", deposit_amount.amount)
    //     .add_submessage(SubMsg {
    //         id: submessage_id,
    //         msg: deposit_stable_msg(
    //             &config.moneymarket,
    //             DEPOSIT_TOKEN_DENOM,
    //             deposit_amount.amount,
    //         )?,
    //         reply_on: ReplyOn::Always,
    //         gas_limit: None,
    //     }))
    Ok(Response::default())
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
