use std::str::FromStr;

use crate::state::{CONFIG, ENDOWMENT, PROFILE, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw3_multisig::{InstantiateMsg as Cw3MultisigInstantiateMsg, Threshold};
use angel_core::messages::dao_token::InstantiateMsg as DaoTokenInstantiateMsg;
use angel_core::messages::donation_match::ExecuteMsg as DonationMatchExecMsg;
use angel_core::messages::index_fund::{
    DepositMsg as IndexFundDepositMsg, ExecuteMsg as IndexFundExecuter,
    QueryMsg as IndexFundQuerier,
};
use angel_core::messages::registrar::{
    ExecuteMsg as RegistrarExecuter, QueryMsg as RegistrarQuerier, UpdateEndowmentEntryMsg,
};
use angel_core::messages::vault::{
    AccountWithdrawMsg, ExecuteMsg as VaultExecuteMsg, QueryMsg as VaultQueryMsg,
};
use angel_core::responses::index_fund::FundListResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse,
};
use angel_core::structs::{
    AcceptedTokens, BalanceResponse, EndowmentFee, EndowmentType, FundingSource, SocialMedialUrls,
    SplitDetails, StrategyComponent, Tier, TransactionRecord,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, redeem_from_vaults, validate_deposit_fund,
    withdraw_from_vaults,
};
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Decimal256, DepsMut, Env, Fraction,
    MessageInfo, QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg};
use cw_asset::{Asset, AssetInfoBase};
use cw_utils::Duration;

pub fn new_cw4_group_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut group_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        // set in "cw4_group" instantiation response.
                        if attrb.key == "group_addr" {
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
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_cw3_multisig_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut multisig_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate" {
                    for attrb in event.attributes {
                        if attrb.key == "_contract_address" {
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
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_dao_token_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut dao_token_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate" {
                    for attrb in event.attributes {
                        if attrb.key == "_contract_address" {
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
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_dao_cw20_token_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut dao_cw20_token_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate" {
                    for attrb in event.attributes {
                        if attrb.key == "_contract_address" {
                            dao_cw20_token_addr = attrb.value;
                        }
                    }
                }
            }

            // update the endowment "dao_token" to be the new contract
            let mut endowment = ENDOWMENT.load(deps.storage)?;
            endowment.dao_token = Some(deps.api.addr_validate(&dao_cw20_token_addr)?);
            ENDOWMENT.save(deps.storage, &endowment)?;

            // NOTE: After some discussion, it should add the logic of instantiating
            //       new "'dao_cw20_token' - axlUSDC" pair contract right HERE.

            Ok(Response::default())
        }
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn new_donation_match_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut donation_match_contract_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate" {
                    for attrb in event.attributes {
                        if attrb.key == "_contract_address" {
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
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
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

    config.settings_controller = match msg.settings_controller {
        Some(controller) => controller,
        None => config.settings_controller,
    };
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
    env: Env,
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

    // validate address strings passed
    endowment.kyc_donors_only = msg.kyc_donors_only;

    if let Some(whitelist) = msg.maturity_whitelist {
        let endow_mature_time = endowment.maturity_time.expect("Cannot get maturity time");
        if endow_mature_time < env.block.time.seconds() {
            let UpdateMaturityWhitelist { add, remove } = whitelist;
            for addr in add {
                let validated_addr = deps.api.addr_validate(&addr)?;
                endowment.maturity_whitelist.push(validated_addr);
            }
            for addr in remove {
                let validated_addr = deps.api.addr_validate(&addr)?;
                let id = endowment
                    .maturity_whitelist
                    .iter()
                    .position(|v| *v == validated_addr);
                if let Some(id) = id {
                    endowment.maturity_whitelist.swap_remove(id);
                }
            }
        }
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

    if config.settings_controller.strategies.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
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

    let mut percentages_sum = Decimal::zero();
    for strategy in strategies.iter() {
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: strategy.vault.to_string(),
                })?,
            }))?;
        if !vault_config.vault.approved {
            return Err(ContractError::InvalidInputs {});
        }
        percentages_sum += strategy.percentage;
    }

    if percentages_sum != Decimal::one() {
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
            percentage: strategy.percentage,
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
    fund: Asset,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;

    let returned_token =
        validate_deposit_fund(deps.as_ref(), config.registrar_contract.as_str(), fund)?;
    let returned_amount = returned_token.amount;

    // check that the deposit token came from an approved Vault SC
    let _vaults_rsp: VaultDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Vault {
                vault_addr: sender_addr.to_string(),
            })?,
        }))?;

    let mut submessages: Vec<SubMsg> = vec![];
    match config.pending_redemptions {
        // last redemption, remove pending u64, and build deposit submsgs
        Some(1) => {
            config.pending_redemptions = None;
            // normal vault receipt if closing_endowment has not been set to TRUE
            if !state.closing_endowment {
                let asset = match returned_token.info {
                    AssetInfoBase::Native(ref denom) => state
                        .balances
                        .locked_balance
                        .get_denom_amount(denom.to_string()),
                    AssetInfoBase::Cw20(ref contract_addr) => state
                        .balances
                        .locked_balance
                        .get_token_amount(deps.api.addr_validate(&contract_addr.to_string())?),
                };
                submessages = deposit_to_vaults(
                    deps.as_ref(),
                    config.registrar_contract.to_string(),
                    asset,
                    &endowment.strategies,
                )?;

                // set token balances available to zero for locked
                let balance = match returned_token.info {
                    AssetInfoBase::Native(ref denom) => Balance::from(vec![Coin {
                        amount: Uint128::zero(),
                        denom: denom.to_string(),
                    }]),
                    AssetInfoBase::Cw20(ref contract_addr) => Balance::Cw20(Cw20CoinVerified {
                        address: contract_addr.clone(),
                        amount: Uint128::zero(),
                    }),
                };
                state.balances.locked_balance.set_token_balances(balance);
            } else {
                // this is a vault receipt triggered by closing an Endowment
                // need to handle beneficiary vs index fund submsg actions taken
                let asset = match returned_token.info {
                    AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
                        amount: state
                            .balances
                            .locked_balance
                            .get_denom_amount(denom.to_string())
                            .amount
                            + state
                                .balances
                                .liquid_balance
                                .get_denom_amount(denom.to_string())
                                .amount,
                        denom: denom.to_string(),
                    }]),
                    AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                        address: contract_addr.clone(),
                        amount: state
                            .balances
                            .locked_balance
                            .get_token_amount(contract_addr.clone())
                            .amount
                            + state
                                .balances
                                .liquid_balance
                                .get_token_amount(contract_addr)
                                .amount,
                    }),
                };
                match state.closing_beneficiary {
                    Some(ref addr) => match asset {
                        Balance::Native(v) => submessages.push(SubMsg::new(BankMsg::Send {
                            to_address: deps.api.addr_validate(addr)?.to_string(),
                            amount: v.0,
                        })),
                        Balance::Cw20(v) => {
                            submessages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                                contract_addr: v.address.to_string(),
                                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                                    recipient: addr.to_string(),
                                    amount: v.amount,
                                })
                                .unwrap(),
                                funds: vec![],
                            })));
                        }
                    },
                    None => {
                        // Get the Index Fund SC address from the Registrar SC
                        let registrar_config: RegistrarConfigResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: config.registrar_contract.to_string(),
                                msg: to_binary(&RegistrarQuerier::Config {})?,
                            }))?;
                        let index_fund: String = match registrar_config.index_fund {
                            Some(addr) => addr,
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
                            match asset {
                                Balance::Native(v) => submessages.push(SubMsg::new(
                                    CosmosMsg::Wasm(WasmMsg::Execute {
                                        contract_addr: index_fund,
                                        msg: to_binary(&IndexFundExecuter::Deposit(
                                            IndexFundDepositMsg {
                                                fund_id: Some(fund_list.funds[0].id),
                                                split: None,
                                            },
                                        ))?,
                                        funds: v.0,
                                    }),
                                )),
                                Balance::Cw20(v) => submessages.push(SubMsg::new(CosmosMsg::Wasm(
                                    WasmMsg::Execute {
                                        contract_addr: v.address.to_string(),
                                        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                                            contract: index_fund,
                                            amount: v.amount,
                                            msg: to_binary(&IndexFundExecuter::Deposit(
                                                IndexFundDepositMsg {
                                                    fund_id: Some(fund_list.funds[0].id),
                                                    split: None,
                                                },
                                            ))
                                            .unwrap(),
                                        })
                                        .unwrap(),
                                        funds: vec![],
                                    },
                                ))),
                            }
                        } else {
                            // Orphaned Endowment (ie. no parent index fund)
                            // send funds to the DANO treasury
                            match asset {
                                Balance::Native(v) => {
                                    submessages.push(SubMsg::new(BankMsg::Send {
                                        to_address: registrar_config.treasury,
                                        amount: v.0,
                                    }))
                                }
                                Balance::Cw20(v) => submessages.push(SubMsg::new(CosmosMsg::Wasm(
                                    WasmMsg::Execute {
                                        contract_addr: v.address.to_string(),
                                        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                                            recipient: registrar_config.treasury,
                                            amount: v.amount,
                                        })
                                        .unwrap(),
                                        funds: vec![],
                                    },
                                ))),
                            }
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
    _info: MessageInfo,
    sender_addr: Addr,
    msg: DepositMsg,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;
    let mut res = Response::new();
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

    // Check the token with "accepted_tokens"
    let mut deposit_token =
        validate_deposit_fund(deps.as_ref(), config.registrar_contract.as_str(), fund)?;
    let mut deposit_amount = deposit_token.amount;

    // Deduct the `deposit_fee` from `deposit_amount` if configured.
    // Send the `deposit_fee` to `payout_address` if any.
    if endowment.deposit_fee.is_some() {
        let EndowmentFee {
            payout_address,
            fee_percentage,
            active,
        } = endowment.deposit_fee.unwrap();
        if active {
            let deposit_fee_amount = deposit_amount
                .multiply_ratio(fee_percentage.numerator(), fee_percentage.denominator());

            deposit_amount -= deposit_fee_amount;
            deposit_token.amount -= deposit_fee_amount;

            match deposit_token.info {
                AssetInfoBase::Native(ref token) => {
                    let deposit_fee: Coin = Coin {
                        denom: token.to_string(),
                        amount: deposit_fee_amount,
                    };
                    res = res.add_message(CosmosMsg::Bank(BankMsg::Send {
                        to_address: payout_address.to_string(),
                        amount: vec![deposit_fee],
                    }));
                }
                AssetInfoBase::Cw20(ref contract_addr) => {
                    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: contract_addr.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: payout_address.to_string(),
                            amount: deposit_fee_amount,
                        })
                        .unwrap(),
                        funds: vec![],
                    }));
                }
            }
        }
    }

    // Get the split to liquid parameters set in the Registrar SC
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    let mut locked_split = msg.locked_percentage;
    let mut liquid_split = msg.liquid_percentage;

    let registrar_split_configs: SplitDetails = registrar_config.split_to_liquid;
    // check split passed by the donor against the Registrar SC split params
    let index_fund = match registrar_config.index_fund {
        Some(addr) => addr,
        None => return Err(ContractError::ContractNotConfigured {}),
    };
    if sender_addr.to_string() != index_fund {
        let new_splits = check_splits(registrar_split_configs, locked_split, liquid_split);
        locked_split = new_splits.0;
        liquid_split = new_splits.1;
    }

    let mut locked_amount = Asset {
        info: deposit_token.info.clone(),
        amount: deposit_amount * locked_split,
    };
    let liquid_amount = Asset {
        info: deposit_token.info.clone(),
        amount: deposit_amount * liquid_split,
    };

    // update total donations recieved for a charity
    let mut state = STATE.load(deps.storage)?;
    state.donations_received += deposit_amount;
    // note the tx in records
    let tx_record = TransactionRecord {
        block: env.block.height,
        sender: sender_addr.clone(),
        recipient: None,
        amount: deposit_amount,
        asset_info: deposit_token.info,
    };
    state.transactions.push(tx_record);
    // increase the liquid balance by donation (liquid) amount
    let liquid_balance = match liquid_amount.info {
        AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: liquid_amount.amount,
        }]),
        AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
            address: contract_addr.clone(),
            amount: liquid_amount.amount,
        }),
    };
    state.balances.liquid_balance.add_tokens(liquid_balance);

    // check if the donation matching is possible
    let mut donor_match_messages: Vec<SubMsg> = vec![];
    if !locked_amount.amount.is_zero() && endowment.donation_match && endowment.dao_token.is_some()
    {
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
        // 10% of "locked_amount" amount
        let donation_match_amount = locked_amount.amount.multiply_ratio(100_u128, 1000_u128);
        locked_amount.amount -= donation_match_amount;

        // build "donor_match" message for donation matching
        match locked_amount.info {
            AssetInfoBase::Native(ref token) => {
                donor_match_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: donation_match_contract.to_string(),
                    msg: to_binary(&DonationMatchExecMsg::DonorMatch {
                        amount: donation_match_amount,
                        donor: sender_addr.clone(),
                        token: endowment.dao_token.unwrap(),
                    })?,
                    funds: vec![Coin {
                        amount: donation_match_amount,
                        denom: token.to_string(),
                    }],
                })));
            }
            AssetInfoBase::Cw20(ref contract_addr) => {
                // IMPORTANT: This part should be done after the
                //            "donation-match" contract implements the `receive_cw20`
                //            The reason is that we should use the `Send/Receive` entry
                //            of CW20 token to send the token & trigger the action in target contract

                // donor_match_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                //     contract_addr: donation_match_contract.to_string(),
                //     msg: to_binary(&DonationMatchExecMsg::DonorMatch {
                //         amount: donation_match_amount,
                //         donor: sender_addr,
                //         token: endowment.dao_token.unwrap(),
                //     })?,
                //     funds: vec![Coin {
                //         amount: donation_match_amount,
                //         denom: token.to_string(),
                //     }],
                // })));
            }
        }
    };

    let deposit_messages;
    let endowment = ENDOWMENT.load(deps.storage)?;
    // check endowment strategies set.
    // if empty: hold locked funds until a vault is set
    if endowment.strategies.is_empty() {
        deposit_messages = vec![];
        // increase the liquid balance by donation (liquid) amount
        let locked_balance = match locked_amount.info {
            AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
                denom: denom.to_string(),
                amount: locked_amount.amount,
            }]),
            AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                address: contract_addr.clone(),
                amount: locked_amount.amount,
            }),
        };
        state.balances.locked_balance.add_tokens(locked_balance);
    } else {
        // if not empty: build deposit messages for each of the sources/amounts
        deposit_messages = deposit_to_vaults(
            deps.as_ref(),
            config.registrar_contract.to_string(),
            locked_amount,
            &endowment.strategies,
        )?;
    }

    STATE.save(deps.storage, &state)?;

    Ok(res
        .add_submessages(deposit_messages)
        .add_submessages(donor_match_messages)
        .add_attribute("action", "account_deposit")
        .add_attribute("sender", sender_addr)
        .add_attribute("deposit_amount", deposit_amount.to_string()))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sources: Vec<FundingSource>,
    beneficiary: String,
    asset_info: AssetInfoBase<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;

    // Check that sender is able to "withdraw"
    let endow_mature_time = endowment.maturity_time.expect("Cannot get maturity time");
    if endow_mature_time < env.block.time.seconds() {
        // check that sender is the owner or the beneficiary
        if info.sender != endowment.owner {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        // check that sender is one of "maturity_whitelist" (if exist)
        if endowment.maturity_whitelist.len() > 0
            && !endowment.maturity_whitelist.contains(&info.sender)
        {
            return Err(ContractError::Unauthorized {});
        }
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
        if source.amount > Uint128::zero()
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
        asset_info.clone(),
    )?;

    // Save the tx record in STATE
    let mut state = STATE.load(deps.storage)?;
    let tx_record = TransactionRecord {
        block: env.block.height,
        sender: env.contract.address.clone(),
        recipient: Some(Addr::unchecked(beneficiary.clone())),
        amount: tx_amounts,
        asset_info,
    };
    state.transactions.push(tx_record);
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_submessages(withdraw_messages)
        .add_attribute("action", "withdrawal")
        .add_attribute("sender", env.contract.address.to_string())
        .add_attribute("beneficiary", beneficiary))
}

pub fn withdraw_liquid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    liquid_amount: Uint128,
    beneficiary: String,
    asset_info: AssetInfoBase<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;

    // Check that sender is able to "withdraw"
    let endow_mature_time = endowment.maturity_time.expect("Cannot get maturity time");
    if endow_mature_time < env.block.time.seconds() {
        // check that sender is the owner or the beneficiary
        if info.sender != endowment.owner {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        // check that sender is one of "maturity_whitelist" (if exist)
        if endowment.maturity_whitelist.len() > 0
            && !endowment.maturity_whitelist.contains(&info.sender)
        {
            return Err(ContractError::Unauthorized {});
        }
    }

    // check that the Endowment has been approved to withdraw deposits
    if !config.withdraw_approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Withdraws are not approved for this endowment".to_string(),
        }));
    }

    let mut state = STATE.load(deps.storage)?;
    // check that the amount in liquid balance is sufficient to cover request
    let amount = match asset_info {
        AssetInfoBase::Native(ref denom) => {
            state
                .balances
                .liquid_balance
                .get_denom_amount(denom.to_string())
                .amount
        }
        AssetInfoBase::Cw20(ref contract_addr) => {
            state
                .balances
                .liquid_balance
                .get_token_amount(contract_addr.clone())
                .amount
        }
    };
    if amount < liquid_amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // Update the Liquid Balance in STATE
    let balance = match asset_info {
        AssetInfoBase::Native(ref denom) => Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: liquid_amount,
        }]),
        AssetInfoBase::Cw20(ref contract_addr) => Balance::Cw20(Cw20CoinVerified {
            address: deps.api.addr_validate(&contract_addr.to_string())?,
            amount: liquid_amount,
        }),
    };
    state.balances.liquid_balance.deduct_tokens(balance);
    STATE.save(deps.storage, &state)?;

    // Send "asset" to the Beneficiary via BankMsg::Send
    let mut messages: Vec<SubMsg> = vec![];
    match asset_info {
        AssetInfoBase::Native(ref denom) => {
            messages.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: beneficiary.to_string(),
                amount: coins(liquid_amount.u128(), denom.to_string()),
            })))
        }
        AssetInfoBase::Cw20(ref contract_addr) => {
            messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: beneficiary.to_string(),
                    amount: liquid_amount,
                })
                .unwrap(),
                funds: vec![],
            })))
        }
    };
    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "withdrawal")
        .add_attribute("sender", env.contract.address.to_string())
        .add_attribute("beneficiary", beneficiary))
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

    let un_sdg = if info.sender == config.owner {
        match msg.un_sdg {
            Some(i) => Some(Some(i)),
            None => Some(None),
        }
    } else {
        None
    };

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
        profile.logo = msg.logo.clone();
        profile.image = msg.image.clone();
        profile.url = msg.url;
        profile.registration_number = msg.registration_number;
        profile.country_of_origin = msg.country_of_origin;
        profile.street_address = msg.street_address;
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
        msg: to_binary(&RegistrarExecuter::UpdateEndowmentEntry(
            UpdateEndowmentEntryMsg {
                endowment_addr: env.contract.address.to_string(),
                name: msg.name,
                logo: msg.logo,
                image: msg.image,
                owner: None,
                tier,
                un_sdg,
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

pub fn update_endowment_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentFeesMsg,
) -> Result<Response, ContractError> {
    // Validation 1. Only "Endowment.owner" or "Config.owner" is able to execute
    let mut endowment = ENDOWMENT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != endowment.owner && info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Update the "EndowmentFee"s
    endowment.earnings_fee = msg.earnings_fee;
    endowment.deposit_fee = msg.deposit_fee;
    endowment.withdraw_fee = msg.withdraw_fee;
    endowment.aum_fee = msg.aum_fee;

    ENDOWMENT.save(deps.storage, &endowment)?;

    Ok(Response::new()
        .add_attribute("action", "update_endowment_fees")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn harvest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;
    // harvest can only be valid if it comes from the  (AP Team/DANO) SC Owner
    if info.sender.ne(&endowment.owner) {
        return Err(ContractError::Unauthorized {});
    }

    let vault_addr = deps.api.addr_validate(&vault_addr)?;

    let sub_messages: Vec<SubMsg> = vec![harvest_msg(
        vault_addr.to_string(),
        config.last_earnings_harvest,
        config.last_harvest_fx,
    )];

    Ok(Response::new()
        .add_submessages(sub_messages)
        .add_attribute("action", "harvest"))
}

fn harvest_msg(
    account: String,
    last_earnings_harvest: u64,
    last_harvest_fx: Option<Decimal256>,
) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account,
        msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Harvest {
            last_earnings_harvest,
            last_harvest_fx,
        })
        .unwrap(),
        funds: vec![],
    });

    SubMsg {
        id: 5,
        msg: wasm_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }
}

pub fn harvest_aum(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let endowment = ENDOWMENT.load(deps.storage)?;

    // Validations
    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Get the `aum_fee` info
    if endowment.aum_fee.is_none() {
        return Err(ContractError::Std(StdError::generic_err(
            "AUM_FEE info is not set",
        )));
    }
    let EndowmentFee {
        fee_percentage,
        payout_address,
        active,
    } = endowment.aum_fee.unwrap();
    if !active {
        return Err(ContractError::Std(StdError::generic_err(
            "AUM_FEE info is not activated",
        )));
    }

    // Calc the total AUM & aum_harvest_withdraw from vaults balances
    let mut msgs: Vec<CosmosMsg> = vec![];
    let vaults: Vec<Addr> = endowment
        .strategies
        .iter()
        .map(|s| s.vault.clone())
        .collect();
    for vault in vaults {
        let vault_balances: BalanceResponse = deps.querier.query_wasm_smart(
            vault.to_string(),
            &VaultQueryMsg::Balance {
                address: vault.to_string(),
            },
        )?;
        // Here, we assume that only one native coin -
        // `UST` is used for deposit/withdraw in vault
        let mut total_aum: Uint128 = Uint128::zero();
        total_aum += vault_balances.locked_native[0].amount;
        total_aum += vault_balances.liquid_native[0].amount;

        // Calc the `aum_harvest_withdraw` amount
        if !total_aum.is_zero() {
            let aum_harvest_withdraw =
                total_aum.multiply_ratio(fee_percentage.numerator(), fee_percentage.denominator());
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: vault.to_string(),
                msg: to_binary(&VaultExecuteMsg::Withdraw(AccountWithdrawMsg {
                    beneficiary: payout_address.clone(),
                    amount: aum_harvest_withdraw,
                }))
                .unwrap(),
                funds: vec![],
            }))
        }
    }

    if msgs.is_empty() {
        return Err(ContractError::Std(StdError::generic_err(
            "Total AUM is zero",
        )));
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "harvest_aum_fee"))
}

pub fn harvest_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut last_earnings_harvest: u64 = 0;
            let mut last_harvest_fx: Option<Decimal256> = None;
            for event in subcall.events {
                if event.ty == "wasm" {
                    for attrb in event.attributes {
                        if attrb.key == "last_earnings_harvest" {
                            last_earnings_harvest = attrb.value.parse::<u64>().unwrap();
                        }
                        if attrb.key == "last_harvest_fx" {
                            last_harvest_fx = Some(Decimal256::from_str(&attrb.value).unwrap());
                        }
                    }
                }
            }

            let mut config = CONFIG.load(deps.storage)?;
            config.last_earnings_harvest = last_earnings_harvest;
            config.last_harvest_fx = last_harvest_fx;
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::default())
        }
        SubMsgResult::Err(_) => Err(ContractError::Std(StdError::generic_err("Harvest failed"))),
    }
}

pub fn setup_dao_token(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    option: DaoSetupOption,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    let submsgs =
        setup_dao_token_messages(deps.branch(), option, &registrar_config, endowment.owner)?;

    Ok(Response::new().add_submessages(submsgs))
}

pub fn setup_dao_token_messages(
    deps: DepsMut,
    option: DaoSetupOption,
    registrar_config: &RegistrarConfigResponse,
    endowment_owner: Addr,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut submsgs: Vec<SubMsg> = vec![];
    match option {
        // Option #1. User can set an existing CW20 token as the DAO's Token
        DaoSetupOption::ExistingCw20Token(contract_addr) => {
            // Validation
            if !registrar_config
                .accepted_tokens
                .cw20_valid(contract_addr.to_string())
            {
                return Err(ContractError::NotInApprovedCoins {});
            }

            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            ENDOWMENT.update(deps.storage, |mut endow| -> StdResult<_> {
                endow.dao_token = Some(contract_addr);
                Ok(endow)
            })?;
        }

        // Option #2. Create a basic CW20 token contract with a fixed supply
        DaoSetupOption::SetupCw20Token(config) => {
            // setup DAO token contract
            submsgs.push(SubMsg {
                id: 6,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: config.code_id,
                    admin: None,
                    label: "new endowment dao token(cw20) contract".to_string(),
                    msg: to_binary(&cw20_base::msg::InstantiateMsg {
                        name: "AP Endowment Dao Token".to_string(),
                        symbol: "APEDT".to_string(),
                        decimals: 6,
                        initial_balances: vec![Cw20Coin {
                            address: endowment_owner.to_string(),
                            amount: config.initial_supply,
                        }],
                        mint: None,
                        marketing: None,
                    })?,
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
        }
        // Option #3. Create a CW20 token with supply controlled by a bonding curve
        DaoSetupOption::SetupBondCurveToken(curve_type) => {
            // setup DAO token contract
            let halo_token = match registrar_config.halo_token.clone() {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "HALO token address is empty".to_string(),
                    }))
                }
            };
            submsgs.push(SubMsg {
                id: 3,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: registrar_config.subdao_token_code.unwrap(),
                    admin: None,
                    label: "new endowment dao token contract".to_string(),
                    msg: to_binary(&DaoTokenInstantiateMsg {
                        name: "AP Endowment Dao Token".to_string(), // need dynamic name
                        symbol: "APEDT".to_string(),                // need dynamic symbol
                        decimals: 6,
                        reserve_denom: halo_token.to_string(),
                        reserve_decimals: 6,
                        curve_type,
                        halo_token,
                        unbonding_period: 7,
                    })?,
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
        }
    }

    Ok(submsgs)
}
