use crate::state::PROFILE;
use crate::state::{CONFIG, ENDOWMENT, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw3_multisig::InstantiateMsg as Cw3MultisigInstantiateMsg;
use angel_core::messages::cw3_multisig::Threshold;
use angel_core::messages::donation_match::ExecuteMsg as DonationMatchExecMsg;
use angel_core::messages::index_fund::DepositMsg as IndexFundDepositMsg;
use angel_core::messages::index_fund::ExecuteMsg as IndexFundExecuter;
use angel_core::messages::index_fund::QueryMsg as IndexFundQuerier;
use angel_core::messages::registrar::{
    ExecuteMsg as RegistrarExecuter, QueryMsg as RegistrarQuerier, UpdateEndowmentTypeMsg,
};
use angel_core::messages::vault::AccountTransferMsg;
use angel_core::responses::index_fund::FundListResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse,
};
use angel_core::structs::{
    AcceptedTokens, EndowmentType, FundingSource, SocialMedialUrls, SplitDetails,
    StrategyComponent, Tier, TransactionRecord,
};
use angel_core::utils::{
    check_splits, deduct_tax, deposit_to_vaults, ratio_adjusted_balance, redeem_from_vaults,
    withdraw_from_vaults,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgExecutionResponse, Uint128,
    WasmMsg, WasmQuery,
};
use cw0::Duration;
use cw20::Balance;

pub fn new_cw4_group_reply(
    deps: DepsMut,
    _env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        ContractResult::Ok(subcall) => {
            let mut group_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate_contract" {
                    for attrb in event.attributes {
                        if attrb.key == "contract_address" {
                            group_addr = attrb.value;
                        }
                    }
                }
            }

            // Register the new Endowment on success Reply
            let _addr = deps.api.addr_validate(&group_addr)?;

            let registrar_config: RegistrarConfigResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.registrar_contract.to_string(),
                    msg: to_binary(&RegistrarQuerier::Config {})?,
                }))?;

            // Fire the creation of new multisig linked to new group
            Ok(Response::new().add_submessage(SubMsg {
                id: 2,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: registrar_config.cw3_code.unwrap(),
                    admin: None,
                    label: "new endowment guardians multisig".to_string(),
                    msg: to_binary(&Cw3MultisigInstantiateMsg {
                        group_addr,
                        threshold: Threshold::ThresholdQuorum {
                            threshold: Decimal::percent(30),
                            quorum: Decimal::percent(50),
                        },
                        max_voting_period: Duration::Time(600),
                    })?,
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            }))
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_cw3_multisig_reply(
    deps: DepsMut,
    _env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    match msg {
        ContractResult::Ok(subcall) => {
            let mut multisig_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate_contract" {
                    for attrb in event.attributes {
                        if attrb.key == "contract_address" {
                            multisig_addr = attrb.value;
                        }
                    }
                }
            }

            // update the endowment owner to be the new multisig contract
            let mut endowment = ENDOWMENT.load(deps.storage)?;
            endowment.owner = deps.api.addr_validate(&multisig_addr)?;
            ENDOWMENT.save(deps.storage, &endowment)?;

            Ok(Response::default())
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_dao_token_reply(
    deps: DepsMut,
    _env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    match msg {
        ContractResult::Ok(subcall) => {
            let mut dao_token_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate_contract" {
                    for attrb in event.attributes {
                        if attrb.key == "contract_address" {
                            dao_token_addr = attrb.value;
                        }
                    }
                }
            }

            // update the endowment owner to be the new multisig contract
            let mut endowment = ENDOWMENT.load(deps.storage)?;
            endowment.dao_token = Some(deps.api.addr_validate(&dao_token_addr)?);
            ENDOWMENT.save(deps.storage, &endowment)?;

            Ok(Response::default())
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_donation_match_reply(
    deps: DepsMut,
    _env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    match msg {
        ContractResult::Ok(subcall) => {
            let mut donation_match_contract_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate_contract" {
                    for attrb in event.attributes {
                        if attrb.key == "contract_address" {
                            donation_match_contract_addr = attrb.value;
                        }
                    }
                }
            }

            // update the endowment "donation_matching_contract" to be new donation_match_contract_addr
            let mut endowment = ENDOWMENT.load(deps.storage)?;
            endowment.donation_matching_contract =
                Some(deps.api.addr_validate(&donation_match_contract_addr)?);
            ENDOWMENT.save(deps.storage, &endowment)?;

            Ok(Response::default())
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = deps.api.addr_validate(&new_owner)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.accepted_tokens = AcceptedTokens {
        native: msg.accepted_tokens_native,
        cw20: msg.accepted_tokens_cw20,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the SC admin can update it's address in the config
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_registrar = deps.api.addr_validate(&new_registrar)?;
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.registrar_contract = new_registrar;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_endowment_settings(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentSettingsMsg,
) -> Result<Response, ContractError> {
    let mut endowment = ENDOWMENT.load(deps.storage)?;

    // only the endowment owner can update these configs
    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"endowment_owner".to_string())
    {
        endowment.owner = match msg.owner {
            Some(i) => deps.api.addr_validate(&i)?,
            None => endowment.owner,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"whitelisted_beneficiaries".to_string())
    {
        endowment.whitelisted_beneficiaries = match msg.whitelisted_beneficiaries {
            Some(i) => i,
            None => endowment.whitelisted_beneficiaries,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"whitelisted_contributors".to_string())
    {
        endowment.whitelisted_contributors = match msg.whitelisted_contributors {
            Some(i) => i,
            None => endowment.whitelisted_contributors,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"name".to_string())
    {
        endowment.name = match msg.name {
            Some(i) => i,
            None => endowment.name,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"description".to_string())
    {
        endowment.description = match msg.description {
            Some(i) => i,
            None => endowment.description,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"withdraw_before_maturity".to_string())
    {
        endowment.withdraw_before_maturity = match msg.withdraw_before_maturity {
            Some(i) => i,
            None => endowment.withdraw_before_maturity,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"maturity_time".to_string())
    {
        endowment.maturity_time = match msg.maturity_time {
            Some(i) => i,
            None => endowment.maturity_time,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"maturity_height".to_string())
    {
        endowment.maturity_height = match msg.maturity_height {
            Some(i) => i,
            None => endowment.maturity_height,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"strategies".to_string())
    {
        endowment.strategies = match msg.strategies {
            Some(i) => i,
            None => endowment.strategies,
        };
    }

    if !endowment
        .locked_endowment_configs
        .contains(&"rebalance".to_string())
    {
        endowment.rebalance = match msg.rebalance {
            Some(i) => i,
            None => endowment.rebalance,
        };
    }
    ENDOWMENT.save(deps.storage, &endowment)?;

    Ok(Response::default())
}

pub fn update_endowment_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentStatusMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the Registrar SC can update these status configs
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.deposit_approved = msg.deposit_approved;
        config.withdraw_approved = msg.withdraw_approved;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_strategies(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    strategies: Vec<Strategy>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENT.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if config.pending_redemptions != None {
        return Err(ContractError::RedemptionInProgress {});
    }

    let mut addresses: Vec<Addr> = strategies
        .iter()
        .map(|strategy| deps.api.addr_validate(&strategy.vault).unwrap())
        .collect();
    addresses.sort();
    addresses.dedup();

    if addresses.len() < strategies.len() {
        return Err(ContractError::StrategyComponentsNotUnique {});
    };

    let mut locked_percentages_sum = Decimal::zero();
    let mut liquid_percentages_sum = Decimal::zero();

    for strategy in strategies.iter() {
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: strategy.vault.to_string(),
                })?,
            }))?;
        if vault_config.vault.approved != true {
            return Err(ContractError::InvalidInputs {});
        }

        locked_percentages_sum = locked_percentages_sum + strategy.locked_percentage;
        liquid_percentages_sum = liquid_percentages_sum + strategy.liquid_percentage;
    }

    if locked_percentages_sum != Decimal::one() || liquid_percentages_sum != Decimal::one() {
        return Err(ContractError::InvalidStrategyAllocation {});
    }

    // redeem all existing strategies from the Endowment's old sources
    // before updating endowment with new sources
    let redeem_messages = redeem_from_vaults(
        deps.as_ref(),
        env.contract.address,
        config.registrar_contract.to_string(),
        endowment.strategies,
    )?;

    config.pending_redemptions = Some(redeem_messages.len() as u64);
    CONFIG.save(deps.storage, &config)?;

    // update endowment strategies attribute with all newly passed strategies
    let mut new_strategies = vec![];
    for strategy in strategies {
        new_strategies.push(StrategyComponent {
            vault: deps.api.addr_validate(&strategy.vault.clone())?,
            locked_percentage: strategy.locked_percentage,
            liquid_percentage: strategy.liquid_percentage,
        });
    }
    endowment.strategies = new_strategies;
    ENDOWMENT.save(deps.storage, &endowment)?;

    Ok(Response::new()
        .add_attribute("action", "update_strategies")
        .add_submessages(redeem_messages))
}

pub fn vault_receipt(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_addr: Addr,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;

    // only accept max of 1 deposit coin/token per donation
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    let returned_amount: Coin = Coin {
        denom: "uusd".to_string(),
        amount: info
            .funds
            .iter()
            .find(|c| c.denom == *"uusd")
            .map(|c| c.amount)
            .unwrap_or_else(Uint128::zero),
    };

    if returned_amount.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // check that the deposit token came from an approved Vault SC
    let _vaults_rsp: VaultDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Vault {
                vault_addr: sender_addr.to_string(),
            })?,
        }))?;

    // funds go into state balances (locked/liquid)
    let total = msg.locked + msg.liquid;
    if !msg.locked.is_zero() {
        state
            .balances
            .locked_balance
            .add_tokens(ratio_adjusted_balance(
                Balance::from(vec![returned_amount.clone()]),
                msg.locked,
                total,
            ));
    }
    if !msg.liquid.is_zero() {
        state
            .balances
            .liquid_balance
            .add_tokens(ratio_adjusted_balance(
                Balance::from(vec![returned_amount]),
                msg.liquid,
                total,
            ));
    }

    let mut submessages: Vec<SubMsg> = vec![];
    match config.pending_redemptions {
        // last redemption, remove pending u64, and build deposit submsgs
        Some(1) => {
            config.pending_redemptions = None;
            // normal vault receipt if closing_endowment has not been set to TRUE
            if !state.closing_endowment {
                submessages = deposit_to_vaults(
                    deps.as_ref(),
                    config.registrar_contract.to_string(),
                    state.balances.locked_balance.get_ust(),
                    state.balances.liquid_balance.get_ust(),
                    &endowment.strategies,
                )?;
                // set UST balances available to zero
                state
                    .balances
                    .locked_balance
                    .set_token_balances(Balance::from(vec![Coin {
                        amount: Uint128::zero(),
                        denom: "uusd".to_string(),
                    }]));
                state
                    .balances
                    .liquid_balance
                    .set_token_balances(Balance::from(vec![Coin {
                        amount: Uint128::zero(),
                        denom: "uusd".to_string(),
                    }]));
            } else {
                // this is a vault receipt triggered by closing an Endowment
                // need to handle beneficiary vs index fund submsg actions taken
                let balance_after_tax = deduct_tax(
                    deps.as_ref(),
                    Coin {
                        amount: state.balances.locked_balance.get_ust().amount
                            + state.balances.liquid_balance.get_ust().amount,
                        denom: "uusd".to_string(),
                    },
                )?;
                match state.closing_beneficiary {
                    Some(ref addr) => submessages.push(SubMsg::new(BankMsg::Send {
                        to_address: deps.api.addr_validate(addr)?.to_string(),
                        amount: vec![balance_after_tax],
                    })),
                    None => {
                        // Get the Index Fund SC address from the Registrar SC
                        let registrar_config: RegistrarConfigResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: config.registrar_contract.to_string(),
                                msg: to_binary(&RegistrarQuerier::Config {})?,
                            }))?;
                        let index_fund: String = match registrar_config.index_fund {
                            Some(addr) => addr.to_string(),
                            None => return Err(ContractError::ContractNotConfigured {}),
                        };

                        // query the Index Fund SC to find the Fund that this Endowment is a member of
                        let fund_list: FundListResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: index_fund.to_string(),
                                msg: to_binary(&IndexFundQuerier::InvolvedFunds {
                                    address: env.contract.address.to_string(),
                                })?,
                            }))?;
                        if !fund_list.funds.is_empty() {
                            // send funds to the first index fund in list
                            submessages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                                contract_addr: index_fund,
                                msg: to_binary(&IndexFundExecuter::Deposit(IndexFundDepositMsg {
                                    fund_id: Some(fund_list.funds[0].id),
                                    split: None,
                                }))?,
                                funds: vec![balance_after_tax],
                            })))
                        } else {
                            // Orphaned Endowment (ie. no parent index fund)
                            // send funds to the DANO treasury
                            submessages.push(SubMsg::new(BankMsg::Send {
                                to_address: registrar_config.treasury,
                                amount: vec![balance_after_tax],
                            }))
                        }
                    }
                }
            }
        }
        // subtract one redemption and hold off on doing deposits
        Some(_) => config.pending_redemptions = Some(config.pending_redemptions.unwrap() - 1),
        None => (),
    };

    STATE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_submessages(submessages)
        .add_attribute("action", "vault_receipt")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_addr: Addr,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let profile = PROFILE.load(deps.storage)?;

    // check that the Endowment has been approved to receive deposits
    if !config.deposit_approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Deposits are not approved for this endowment".to_string(),
        }));
    }

    // check that the split %s sum to 1
    if msg.locked_percentage + msg.liquid_percentage != Decimal::one() {
        return Err(ContractError::InvalidSplit {});
    }

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

    let mut locked_split = msg.locked_percentage;
    let mut liquid_split = msg.liquid_percentage;

    // Get the split to liquid parameters set in the Registrar SC
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let registrar_split_configs: SplitDetails = registrar_config.split_to_liquid;

    // check split passed by the donor against the Registrar SC split params
    let index_fund = match registrar_config.index_fund {
        Some(addr) => addr,
        None => return Err(ContractError::ContractNotConfigured {}),
    };
    if sender_addr != index_fund {
        let new_splits = check_splits(registrar_split_configs, locked_split, liquid_split);
        locked_split = new_splits.0;
        liquid_split = new_splits.1;
    }

    let mut ust_locked = Coin {
        amount: deposit_amount.amount * locked_split,
        denom: "uusd".to_string(),
    };
    let ust_liquid = Coin {
        amount: deposit_amount.amount * liquid_split,
        denom: "uusd".to_string(),
    };

    // update total donations recieved for a charity
    let mut state = STATE.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;
    state.donations_received += deposit_amount.amount;

    let tx_record = TransactionRecord {
        block: env.block.height,
        sender: sender_addr.clone(),
        recipient: None,
        amount: deposit_amount.amount,
        denom: deposit_amount.denom,
    };
    state.transactions.push(tx_record);
    STATE.save(deps.storage, &state)?;

    // check if the donation matching is possible
    let mut doner_match_messages: Vec<SubMsg> = vec![];
    if !ust_locked.amount.is_zero() && endowment.donation_match && endowment.dao_token.is_some() {
        // get the correct donation match contract to use
        let donation_match_contract = match profile.endow_type {
            EndowmentType::Normal => match endowment.donation_matching_contract {
                Some(addr) => addr,
                None => return Err(ContractError::AccountNotCreated {}),
            },
            EndowmentType::Charity => match registrar_config.donation_match_charites_contract {
                Some(addr) => deps.api.addr_validate(&addr)?,
                None => return Err(ContractError::AccountDoesNotExist {}),
            },
        };
        // 10% of "ust_locked" amount
        let donation_match_amount = ust_locked.amount.multiply_ratio(100_u128, 1000_u128);
        ust_locked = Coin {
            amount: ust_locked.amount - donation_match_amount,
            denom: "uusd".to_string(),
        };

        // build "doner_match" message for donation matching
        doner_match_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: donation_match_contract.to_string(),
            msg: to_binary(&DonationMatchExecMsg::DonorMatch {
                amount: donation_match_amount,
                donor: sender_addr,
                token: endowment.dao_token.unwrap(),
            })?,
            funds: vec![Coin {
                amount: donation_match_amount,
                denom: "uusd".to_string(),
            }],
        })));
    };

    // build deposit messages for each of the sources/amounts
    let deposit_messages = deposit_to_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        ust_locked,
        ust_liquid,
        &endowment.strategies,
    )?;

    Ok(Response::new()
        .add_submessages(deposit_messages)
        .add_submessages(doner_match_messages)
        .add_attribute("action", "account_deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", deposit_amount.amount.to_string()))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sources: Vec<FundingSource>,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;

    // check that sender is the owner or the beneficiary
    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // check that the Endowment has been approved to withdraw deposits
    if !config.withdraw_approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Withdraws are not approved for this endowment".to_string(),
        }));
    }

    // check if locked tokens are requested and
    // reject if endowment cannot withdraw from locked before maturity
    for source in sources.iter() {
        if source.locked > Uint128::zero()
            && (!endowment.withdraw_before_maturity || !endowment.is_expired(&env))
        {
            return Err(ContractError::InaccessableLockedBalance {});
        }
    }

    // build redeem messages for each of the sources/amounts
    let (withdraw_messages, tx_amounts) = withdraw_from_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        &deps.api.addr_validate(&beneficiary)?,
        sources,
    )?;

    // Save the tx record in STATE
    let mut state = STATE.load(deps.storage)?;
    let tx_record = TransactionRecord {
        block: env.block.height,
        sender: env.contract.address.clone(),
        recipient: Some(Addr::unchecked(beneficiary.clone())),
        amount: tx_amounts,
        denom: "uusd".to_string(),
    };
    state.transactions.push(tx_record);
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_submessages(withdraw_messages)
        .add_attribute("action", "withdrawal")
        .add_attribute("sender", env.contract.address.to_string())
        .add_attribute("beneficiary", beneficiary.to_string()))
}

pub fn close_endowment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    if config.pending_redemptions != None {
        return Err(ContractError::RedemptionInProgress {});
    }

    // set the STATE with relevent status and closing beneficiary
    let mut state = STATE.load(deps.storage)?;
    state.closing_endowment = true;
    state.closing_beneficiary = beneficiary;
    STATE.save(deps.storage, &state)?;

    // Redeem all UST back from strategies invested in
    let endowment = ENDOWMENT.load(deps.storage)?;
    let redeem_messages = redeem_from_vaults(
        deps.as_ref(),
        env.contract.address,
        config.registrar_contract.to_string(),
        endowment.strategies,
    )?;

    config.pending_redemptions = Some(redeem_messages.len() as u64);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "close_endowment")
        .add_attribute("sender", info.sender.to_string())
        .add_submessages(redeem_messages))
}

pub fn update_profile(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateProfileMsg,
) -> Result<Response, ContractError> {
    // Validation 1. Only "Endowment.owner" or "Config.owner" is able to execute
    let endowment = ENDOWMENT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != endowment.owner && info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let tier = if info.sender == config.owner {
        match msg.tier {
            Some(1) => Some(Some(Tier::Level1)),
            Some(2) => Some(Some(Tier::Level2)),
            Some(3) => Some(Some(Tier::Level3)),
            None => Some(None),
            _ => return Err(ContractError::InvalidInputs {}),
        }
    } else {
        None
    };

    // Update the Endowment profile
    let mut profile = PROFILE.load(deps.storage)?;

    // Only config.owner can update "un_sdg" & "tier" fields
    if info.sender == config.owner {
        profile.un_sdg = msg.un_sdg;
        profile.tier = msg.tier;
        if let Some(endow_type) = msg.endow_type {
            profile.endow_type = match endow_type.as_str() {
                "charity" => EndowmentType::Charity,
                "normal" => EndowmentType::Normal,
                _ => return Err(ContractError::InvalidInputs {}),
            };
        }
    }

    // Only endowment.owner can update all other fields
    if info.sender == endowment.owner {
        if let Some(name) = msg.name.clone() {
            profile.name = name;
        }
        if let Some(overview) = msg.overview {
            profile.overview = overview;
        }
        profile.logo = msg.logo;
        profile.image = msg.image;
        profile.url = msg.url;
        profile.registration_number = msg.registration_number;
        profile.country_city_origin = msg.country_city_origin;
        profile.contact_email = msg.contact_email;
        profile.number_of_employees = msg.number_of_employees;
        profile.average_annual_budget = msg.average_annual_budget;
        profile.annual_revenue = msg.annual_revenue;
        profile.charity_navigator_rating = msg.charity_navigator_rating;

        let social_media_urls = SocialMedialUrls {
            facebook: msg.facebook,
            twitter: msg.twitter,
            linkedin: msg.linkedin,
        };
        profile.social_media_urls = social_media_urls;
    }

    PROFILE.save(deps.storage, &profile)?;

    let sub_msgs: Vec<SubMsg> = vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.registrar_contract.to_string(),
        msg: to_binary(&RegistrarExecuter::UpdateEndowmentType(
            UpdateEndowmentTypeMsg {
                endowment_addr: env.contract.address.to_string(),
                name: msg.name.and_then(|v| Some(v)),
                owner: None,
                tier,
                endow_type: Some(profile.endow_type),
            },
        ))?,
        funds: vec![],
    }))];

    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_attribute("action", "update_profile")
        .add_attribute("sender", info.sender.to_string()))
}
