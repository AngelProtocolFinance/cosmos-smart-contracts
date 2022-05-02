use crate::anchor;
use crate::anchor::{deposit_stable_msg, redeem_stable_msg};
use crate::config;
use crate::config::{PendingInfo, BALANCES, PENDING, TOKEN_INFO};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{AccountTransferMsg, AccountWithdrawMsg, UpdateConfigMsg};
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentListResponse,
};
use angel_core::structs::{BalanceInfo, EndowmentEntry};
use angel_core::utils::deduct_tax;
use cosmwasm_std::{
    to_binary, Addr, Attribute, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, DepsMut, Env,
    MessageInfo, Order, QueryRequest, ReplyOn, Response, StdError, SubMsg, SubMsgExecutionResponse,
    Uint128, WasmMsg, WasmQuery,
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

    config.moneymarket = match msg.moneymarket {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.moneymarket,
    };

    let anchor_config = anchor::config(deps.as_ref(), &config.moneymarket)?;
    config.yield_token = deps.api.addr_validate(&anchor_config.aterra_contract)?;
    config.input_denom = anchor_config.stable_denom;
    config.tax_per_block = msg.tax_per_block.unwrap_or(config.tax_per_block);
    config.harvest_to_liquid = msg.harvest_to_liquid.unwrap_or(config.harvest_to_liquid);
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn deposit_stable(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AccountTransferMsg,
    _balance: Balance,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only accept max of 1 deposit coin/token per donation
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    let deposit_amount: Coin = Coin {
        denom: "uusd".to_string(),
        amount: info
            .funds
            .iter()
            .find(|c| c.denom == *"uusd")
            .map(|c| c.amount)
            .unwrap_or_else(Uint128::zero),
    };

    if deposit_amount.amount.is_zero() {
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

    let after_taxes = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.input_denom.clone(),
            amount: deposit_amount.amount,
        },
    )?;
    let mut after_taxes_locked = Uint128::zero();
    if !msg.locked.is_zero() {
        after_taxes_locked = after_taxes
            .amount
            .clone()
            .multiply_ratio(msg.locked, deposit_amount.amount);
    }

    let mut after_taxes_liquid = Uint128::zero();
    if !msg.liquid.is_zero() {
        after_taxes_liquid = after_taxes
            .amount
            .clone()
            .multiply_ratio(msg.liquid, deposit_amount.amount);
    }

    let submessage_id = config.next_pending_id;
    PENDING.save(
        deps.storage,
        &submessage_id.to_be_bytes(),
        &PendingInfo {
            typ: "deposit".to_string(),
            accounts_address: info.sender.clone(),
            beneficiary: None,
            fund: None,
            locked: after_taxes_locked,
            liquid: after_taxes_liquid,
        },
    )?;
    config.next_pending_id += 1;
    config::store(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender)
        .add_attribute("deposit_amount", deposit_amount.amount)
        .add_submessage(SubMsg {
            id: submessage_id,
            msg: deposit_stable_msg(&config.moneymarket, "uusd", after_taxes.amount)?,
            reply_on: ReplyOn::Always,
            gas_limit: None,
        }))
}

/// Redeem Stable: Take in an amount of locked/liquid deposit tokens
/// to redeem from the vault for UST to send back to the the Accounts SC
pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    account_addr: Addr,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // check that the depositor is a Registered Accounts SC
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
    let pos = endowments
        .iter()
        .position(|p| p.address == info.sender.clone());

    // reject if the sender was found not in the list of endowments
    // OR if the sender is not the Registrar SC (ie. we're closing the endowment)
    if pos == None && info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // use arg account_addr to lookup Balances
    let mut investment = BALANCES
        .load(deps.storage, &account_addr)
        .unwrap_or_else(|_| BalanceInfo::default());

    // grab total tokens for locked and liquid balances
    let locked_deposit_tokens = investment
        .locked_balance
        .get_token_amount(env.contract.address.clone());
    let liquid_deposit_tokens = investment
        .liquid_balance
        .get_token_amount(env.contract.address.clone());

    let total_redemption = locked_deposit_tokens + liquid_deposit_tokens;

    // reduce the total supply of CW20 deposit tokens by redemption amount
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply -= total_redemption;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // update investment holdings balances to zero
    let zero_tokens = Cw20CoinVerified {
        amount: Uint128::zero(),
        address: env.contract.address,
    };
    investment
        .locked_balance
        .set_token_balances(Balance::Cw20(zero_tokens.clone()));
    investment
        .liquid_balance
        .set_token_balances(Balance::Cw20(zero_tokens));

    BALANCES.save(deps.storage, &account_addr, &investment)?;

    let submessage_id = config.next_pending_id;
    PENDING.save(
        deps.storage,
        &submessage_id.to_be_bytes(),
        &PendingInfo {
            typ: "redeem".to_string(),
            accounts_address: account_addr,
            beneficiary: None,
            fund: None,
            locked: locked_deposit_tokens,
            liquid: liquid_deposit_tokens,
        },
    )?;
    config.next_pending_id += 1;
    config::store(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "redeem_from_vault")
        .add_attribute("sender", info.sender)
        .add_attribute("redeem_amount", total_redemption)
        .add_submessage(SubMsg {
            id: submessage_id,
            msg: redeem_stable_msg(&config.moneymarket, &config.yield_token, total_redemption)?,
            gas_limit: None,
            reply_on: ReplyOn::Always,
        }))
}

/// Withdraw Stable: Takes in an amount of locked/liquid deposit tokens
/// to withdraw from the vault for UST to send back to a beneficiary
pub fn withdraw_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // check that the tx sender is an Accounts SC
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

    // reject if the sender was found not in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // reduce the total supply of CW20 deposit tokens
    let withdraw_total = msg.locked + msg.liquid;
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply -= withdraw_total;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or_else(|_| BalanceInfo::default());

    // check the account has enough balance to cover the withdraw
    let locked_balance = investment
        .locked_balance
        .get_token_amount(env.contract.address.clone());
    let liquid_balance = investment
        .liquid_balance
        .get_token_amount(env.contract.address.clone());
    if locked_balance < msg.locked || liquid_balance < msg.liquid {
        return Err(ContractError::CannotExceedCap {});
    }

    // update investment holdings balances
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
    BALANCES.save(deps.storage, &info.sender, &investment)?;

    let submessage_id = config.next_pending_id;
    PENDING.save(
        deps.storage,
        &submessage_id.to_be_bytes(),
        &PendingInfo {
            typ: "withdraw".to_string(),
            accounts_address: info.sender.clone(),
            beneficiary: Some(msg.beneficiary.clone()),
            fund: None,
            locked: msg.locked,
            liquid: msg.liquid,
        },
    )?;
    config.next_pending_id += 1;
    config::store(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "withdraw_from_vault")
        .add_attribute("sender", info.sender)
        .add_attribute("withdraw_amount", withdraw_total)
        .add_submessage(SubMsg {
            id: submessage_id,
            msg: redeem_stable_msg(&config.moneymarket, &config.yield_token, withdraw_total)?,
            gas_limit: None,
            reply_on: ReplyOn::Always,
        }))
}

pub fn harvest(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collector_address: String,
    collector_share: Decimal,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    if info.sender != config.registrar_contract && info.sender.to_string() != *CRON_WALLET {
        return Err(ContractError::Unauthorized {});
    }

    let curr_epoch = anchor::epoch_state(deps.as_ref(), &config.moneymarket)?;

    let harvest_earn_rate = Decimal::from(
        (curr_epoch.exchange_rate - config.last_harvest_fx.unwrap_or(curr_epoch.exchange_rate))
            / config.last_harvest_fx.unwrap_or(curr_epoch.exchange_rate),
    );

    config.last_harvest = env.block.height;
    config.last_harvest_fx = Some(curr_epoch.exchange_rate);
    config::store(deps.storage, &config)?;

    // pull registrar SC config to fetch the Treasury Tax Rate
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::Config {})?,
        }))?;
    let treasury_addr = deps.api.addr_validate(&registrar_config.treasury)?;
    let collector_addr = deps.api.addr_validate(&collector_address)?;
    let mut harvest_account = BalanceInfo::default();

    // iterate over all accounts and shuffle DP tokens from Locked to Liquid
    // set aside a small amount for treasury
    let accounts: Result<Vec<_>, _> = BALANCES
        .keys(deps.storage, None, None, Order::Ascending)
        .map(String::from_utf8)
        .collect();
    for account in accounts.unwrap().iter() {
        let account_address = deps.api.addr_validate(account)?;
        let mut balances = BALANCES
            .load(deps.storage, &account_address)
            .unwrap_or_else(|_| BalanceInfo::default());

        // CALCULATE ALL AMOUNTS TO BE COLLECTED FOR TAXES AND TO BE TRANSFERED
        // UPFRONT BEFORE PERFORMING ANY ACTUAL BALANCE SHUFFLES
        // calculate harvest taxes owed on liquid balance earnings
        let liquid_taxes_owed = balances
            .liquid_balance
            .get_token_amount(env.contract.address.clone())
            * harvest_earn_rate
            * registrar_config.tax_rate;

        // calculate harvest taxes owed on locked balance earnings
        let locked_taxes_owed = balances
            .locked_balance
            .get_token_amount(env.contract.address.clone())
            * harvest_earn_rate
            * registrar_config.tax_rate;

        // calulate amount of earnings to be harvested from locked >> liquid balance
        // reduce harvest amount by any locked taxes owed on those earnings
        let transfer_amt = balances
            .locked_balance
            .get_token_amount(env.contract.address.clone())
            * harvest_earn_rate
            * config.harvest_to_liquid
            - locked_taxes_owed;

        // deduct liquid taxes if we have a non-zero amount
        if liquid_taxes_owed > Uint128::zero() {
            let deposit_token = Cw20CoinVerified {
                address: env.contract.address.clone(),
                amount: liquid_taxes_owed,
            };
            // lower liquid balance
            balances
                .liquid_balance
                .deduct_tokens(Balance::Cw20(deposit_token.clone()));

            // add taxes collected to the liquid balance of the Collector
            harvest_account
                .liquid_balance
                .add_tokens(Balance::Cw20(deposit_token.clone()));
        }

        // deduct locked taxes if we have a non-zero amount
        if locked_taxes_owed > Uint128::zero() {
            let deposit_token = Cw20CoinVerified {
                address: env.contract.address.clone(),
                amount: locked_taxes_owed,
            };
            // lower locked balance
            balances
                .locked_balance
                .deduct_tokens(Balance::Cw20(deposit_token.clone()));

            // add taxes collected to the liquid balance of the Collector
            harvest_account
                .liquid_balance
                .add_tokens(Balance::Cw20(deposit_token.clone()));
        }

        // proceed to shuffle balances if we have a non-zero amount
        if transfer_amt > Uint128::zero() {
            let deposit_token = Cw20CoinVerified {
                address: env.contract.address.clone(),
                amount: transfer_amt,
            };

            // lower locked balance
            balances
                .locked_balance
                .deduct_tokens(Balance::Cw20(deposit_token.clone()));

            // add to liquid balance
            balances
                .liquid_balance
                .add_tokens(Balance::Cw20(deposit_token.clone()));
        }

        BALANCES.save(deps.storage, &account_address, &balances)?;
    }

    if harvest_account
        .liquid_balance
        .get_token_amount(env.contract.address.clone())
        > Uint128::zero()
    {
        // Withdraw all DP Tokens from Treasury and send to Collector Contract and/or the AP Treasury Wallet
        let withdraw_total = harvest_account
            .liquid_balance
            .get_token_amount(env.contract.address);
        let mut withdraw_leftover = withdraw_total;

        let mut res = Response::new()
            .add_attribute("action", "harvest_redeem_from_vault")
            .add_attribute("sender", info.sender)
            .add_attribute("withdraw_amount", withdraw_total);

        // Harvested Amount is split by collector split input percentage
        if !collector_share.is_zero() && collector_share <= Decimal::one() {
            let submessage_id = config.next_pending_id;
            PENDING.save(
                deps.storage,
                &submessage_id.to_be_bytes(),
                &PendingInfo {
                    typ: "withdraw".to_string(),
                    accounts_address: collector_addr.clone(),
                    beneficiary: Some(collector_addr.clone()),
                    fund: None,
                    locked: Uint128::zero(),
                    liquid: withdraw_total * collector_share,
                },
            )?;
            withdraw_leftover = withdraw_total - (withdraw_total * collector_share);
            config.next_pending_id += 1;
            res = res
                .add_attribute("collector_addr", collector_addr)
                .add_submessage(SubMsg {
                    id: submessage_id,
                    msg: redeem_stable_msg(
                        &config.moneymarket,
                        &config.yield_token,
                        withdraw_total * collector_share,
                    )?,
                    gas_limit: None,
                    reply_on: ReplyOn::Always,
                });
        }

        // Remainder (if any) is sent to AP Treasury Address
        if withdraw_leftover > Uint128::zero() {
            let submessage_id = config.next_pending_id;
            PENDING.save(
                deps.storage,
                &submessage_id.to_be_bytes(),
                &PendingInfo {
                    typ: "withdraw".to_string(),
                    accounts_address: treasury_addr.clone(),
                    beneficiary: Some(treasury_addr.clone()),
                    fund: None,
                    locked: Uint128::zero(),
                    liquid: withdraw_leftover,
                },
            )?;
            config.next_pending_id += 1;
            config::store(deps.storage, &config)?;
            res = res
                .add_attribute("treasury_addr", treasury_addr)
                .add_submessage(SubMsg {
                    id: submessage_id,
                    msg: redeem_stable_msg(
                        &config.moneymarket,
                        &config.yield_token,
                        withdraw_leftover,
                    )?,
                    gas_limit: None,
                    reply_on: ReplyOn::Always,
                });
        }
        Ok(res)
    } else {
        Ok(Response::new()
            .add_attribute("action", "harvest_redeem_from_vault")
            .add_attribute("sender", info.sender))
    }
}

pub fn process_anchor_reply(
    deps: DepsMut,
    env: Env,
    id: u64,
    result: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    // pull up the pending transaction details from storage
    let transaction = PENDING.load(deps.storage, &id.to_be_bytes())?;
    let transaction_total = transaction.locked + transaction.liquid;

    // remove this pending transaction
    PENDING.remove(deps.storage, &id.to_be_bytes());

    match result {
        ContractResult::Ok(subcall) => {
            // Grab the Amount returned from Anchor (UST/aUST)
            let mut anchor_amount = Uint128::zero();
            for event in subcall.events {
                if event.ty == "wasm" {
                    let deposit_attr: Attribute = Attribute::new("action", "deposit_stable");
                    if event.attributes.clone().contains(&deposit_attr) {
                        for attr in event.attributes.clone() {
                            if attr.key == "mint_amount" {
                                anchor_amount = Uint128::from(attr.value.parse::<u128>().unwrap());
                                break;
                            }
                        }
                    }

                    let redeem_attr: Attribute = Attribute::new("action", "redeem_stable");
                    if event.attributes.contains(&redeem_attr) {
                        for attr in event.attributes {
                            if attr.key == "redeem_amount" {
                                anchor_amount = Uint128::from(attr.value.parse::<u128>().unwrap());
                                break;
                            }
                        }
                    }
                }
            }

            // Get the correct Anchor returned amount split by Locked/Liquid ratio in the transaction
            let anchor_locked = anchor_amount
                .clone()
                .multiply_ratio(transaction.locked, transaction_total);
            let anchor_liquid = anchor_amount
                .clone()
                .multiply_ratio(transaction.liquid, transaction_total);

            let res = match transaction.typ.as_str() {
                "deposit" => {
                    // Increase the Account's Deposit token balances by the correct amounts of aUST
                    let mut investment = BALANCES
                        .load(deps.storage, &transaction.accounts_address.clone())
                        .unwrap_or_else(|_| BalanceInfo::default());
                    investment
                        .locked_balance
                        .add_tokens(Balance::Cw20(Cw20CoinVerified {
                            amount: anchor_locked,
                            address: env.contract.address.clone(),
                        }));
                    investment
                        .liquid_balance
                        .add_tokens(Balance::Cw20(Cw20CoinVerified {
                            amount: anchor_liquid,
                            address: env.contract.address,
                        }));
                    BALANCES.save(deps.storage, &transaction.accounts_address, &investment)?;

                    // update total token supply by total aUST returned from deposit
                    let mut token_info = TOKEN_INFO.load(deps.storage)?;
                    token_info.total_supply += anchor_amount;
                    TOKEN_INFO.save(deps.storage, &token_info)?;

                    Response::new()
                        .add_attribute("action", "anchor_reply_processing")
                        .add_attribute("mint_amount", anchor_amount)
                }
                "redeem" => {
                    let after_tax_locked = deduct_tax(
                        deps.as_ref(),
                        deduct_tax(
                            deps.as_ref(),
                            Coin {
                                amount: anchor_locked,
                                denom: "uusd".to_string(),
                            },
                        )?,
                    )?;
                    let after_tax_liquid = deduct_tax(
                        deps.as_ref(),
                        deduct_tax(
                            deps.as_ref(),
                            Coin {
                                amount: anchor_liquid,
                                denom: "uusd".to_string(),
                            },
                        )?,
                    )?;

                    Response::new()
                        .add_attribute("action", "anchor_reply_processing")
                        // Send UST back to the Account SC via VaultReciept msg
                        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: transaction.accounts_address.to_string(),
                            msg: to_binary(
                                &angel_core::messages::accounts::ExecuteMsg::VaultReceipt(
                                    AccountTransferMsg {
                                        locked: after_tax_locked.amount,
                                        liquid: after_tax_liquid.amount,
                                    },
                                ),
                            )?,
                            funds: vec![Coin {
                                amount: after_tax_locked.amount + after_tax_liquid.amount,
                                denom: "uusd".to_string(),
                            }],
                        }))
                }
                "withdraw" => {
                    Response::new()
                        .add_attribute("action", "anchor_reply_processing")
                        // Send UST to the Beneficiary via BankMsg::Send
                        .add_message(BankMsg::Send {
                            to_address: transaction.beneficiary.unwrap().to_string(),
                            amount: vec![deduct_tax(
                                deps.as_ref(),
                                deduct_tax(
                                    deps.as_ref(),
                                    Coin {
                                        amount: anchor_amount,
                                        denom: "uusd".to_string(),
                                    },
                                )?,
                            )?],
                        })
                }
                &_ => Response::new().add_attribute("action", "anchor_reply_processing"),
            };

            // return the response with follow up
            // messages to beneficiary/Accounts/etc
            Ok(res)
        }
        ContractResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}
