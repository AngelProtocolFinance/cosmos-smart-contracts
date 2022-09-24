use crate::state::{Endowment, State, CONFIG, COPYCATS, ENDOWMENTS, PROFILES, STATES};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw3_multisig::EndowmentInstantiateMsg as Cw3InstantiateMsg;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::messages::router::ExecuteMsg as SwapRouterExecuteMsg;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse, VaultListResponse,
};
use angel_core::structs::{
    AccountStrategies, AccountType, BalanceInfo, Beneficiary, DonationsReceived, EndowmentStatus,
    EndowmentType, GenericBalance, OneOffVaults, RebalanceDetails, SocialMedialUrls, SplitDetails,
    StrategyComponent, SwapOperation, VaultType, YieldVault,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, validate_deposit_fund, vault_endowment_balance,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, QueryRequest,
    ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult, Timestamp, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw4::Member;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetUnchecked};
use cw_utils::Duration;

pub fn cw3_reply(deps: DepsMut, _env: Env, msg: SubMsgResult) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut id: u32 = 0;
            let mut owner: Addr = Addr::unchecked("");
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "multisig_addr" => owner = deps.api.addr_validate(&attrb.value)?,
                            "endow_id" => id = attrb.value.parse().unwrap(),
                            _ => (),
                        }
                    }
                }
            }
            if id == 0 || owner == Addr::unchecked("") {
                return Err(ContractError::AccountNotCreated {});
            }
            let mut endowment = ENDOWMENTS.load(deps.storage, id)?;
            endowment.owner = owner;
            ENDOWMENTS.save(deps.storage, id, &endowment)?;

            // set new CW3 as endowment owner to be picked up by the Registrar (EndowmentEntry)
            Ok(Response::default().add_attribute("endow_owner", endowment.owner.to_string()))
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn create_endowment(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // Charity endowments must be created through the CW3 Review Applications
    if msg.endow_type == EndowmentType::Charity
        && info.sender.to_string() != registrar_config.applications_review
    {
        return Err(ContractError::Unauthorized {});
    }

    if !msg.categories.general.is_empty() {
        msg.categories.general.sort();
        if msg.categories.general.last().unwrap() > &config.max_general_category_id {
            return Err(ContractError::InvalidInputs {});
        }
    }

    let owner = deps.api.addr_validate(&msg.owner)?;
    // try to store the endowment, fail if the ID is already in use
    ENDOWMENTS.update(
        deps.storage,
        config.next_account_id,
        |existing| match existing {
            Some(_) => Err(ContractError::AlreadyInUse {}),
            None => Ok(Endowment {
                owner,
                name: msg.name.clone(),
                categories: msg.categories.clone(),
                endow_type: msg.endow_type.clone(),
                status: EndowmentStatus::Approved,
                deposit_approved: true,
                withdraw_approved: true,
                withdraw_before_maturity: msg.withdraw_before_maturity,
                maturity_height: msg.maturity_height,
                maturity_time: msg.maturity_time,
                strategies: AccountStrategies::default(),
                oneoff_vaults: OneOffVaults::default(),
                rebalance: RebalanceDetails::default(),
                kyc_donors_only: msg.kyc_donors_only,
                pending_redemptions: 0_u8,
                copycat_strategy: None,
                tier: msg.tier.clone(),
                logo: msg.logo.clone(),
                image: msg.image.clone(),
                proposal_link: msg.proposal_link,
            }),
        },
    )?;

    // store the profile data
    PROFILES.save(deps.storage, config.next_account_id, &msg.profile)?;

    STATES.save(
        deps.storage,
        config.next_account_id,
        &State {
            donations_received: DonationsReceived {
                locked: Uint128::zero(),
                liquid: Uint128::zero(),
            },
            balances: BalanceInfo::default(),
            closing_endowment: false,
            closing_beneficiary: None,
        },
    )?;

    COPYCATS.save(deps.storage, config.next_account_id, &vec![])?;

    // initial default Response to add submessages to
    let mut res = Response::new();
    if registrar_config.cw3_code.eq(&None) || registrar_config.cw4_code.eq(&None) {
        return Err(ContractError::Std(StdError::generic_err(
            "cw3_code & cw4_code must exist",
        )));
    }

    // Add submessage to create new CW3 multisig for the endowment
    res = res.add_submessage(SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: registrar_config.cw3_code.unwrap(),
            admin: None,
            label: format!("new endowment cw3 multisig - {}", config.next_account_id),
            msg: to_binary(&Cw3InstantiateMsg {
                // endowment ID
                id: config.next_account_id,
                // check if CW3/CW4 codes were passed to setup a multisig/group
                cw4_members: match msg.cw4_members.is_empty() {
                    true => vec![Member {
                        addr: msg.owner.to_string(),
                        weight: 1,
                    }],
                    false => msg.cw4_members,
                },
                cw4_code: registrar_config.cw4_code.unwrap(),
                threshold: msg.cw3_threshold,
                max_voting_period: Duration::Time(msg.cw3_max_voting_period),
                registrar_contract: config.registrar_contract.to_string(),
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    // bump the next account ID and save
    config.next_account_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(res)
}

pub fn update_endowment_status(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentStatusMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // look up the endowment in the Registry. Will fail if doesn't exist
    let endowment_id = msg.endowment_id;
    let mut endowment = ENDOWMENTS.load(deps.storage, endowment_id)?;

    // check that the endowment has not been closed (liquidated or terminated) as this is not reversable
    if endowment.status == EndowmentStatus::Closed {
        return Err(ContractError::AccountClosed {});
    }

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    if !(info.sender == config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    if msg.status > 3 {
        return Err(ContractError::Std(StdError::generic_err(
            "Status not found",
        )));
    }

    let msg_endowment_status = match msg.status {
        0 => EndowmentStatus::Inactive,
        1 => EndowmentStatus::Approved,
        2 => EndowmentStatus::Frozen,
        3 => EndowmentStatus::Closed,
        _ => EndowmentStatus::Inactive, // should never be reached due to status check earlier
    };
    // check first that the current status is different from the new status sent
    if endowment.status.to_string() == msg_endowment_status.to_string() {
        return Ok(Response::default());
    }

    // Take different actions on the affected Accounts SC, based on the status passed
    // Build out list of SubMsgs to send to the Account SC and/or Index Fund SC
    // 1. INDEX FUND - Update fund members list removing a member if the member can no longer accept deposits
    // 2. ACCOUNTS - Update the Endowment deposit/withdraw approval config settings based on the new status
    let index_fund_contract = match registrar_config.index_fund {
        Some(addr) => addr,
        None => return Err(ContractError::ContractNotConfigured {}),
    };
    let sub_messages: Vec<SubMsg> = match msg_endowment_status {
        // Allowed to receive donations and process withdrawals
        EndowmentStatus::Approved => {
            endowment.deposit_approved = true;
            endowment.withdraw_approved = true;
            vec![]
        }
        // Can accept inbound deposits, but cannot withdraw funds out
        EndowmentStatus::Frozen => {
            endowment.deposit_approved = true;
            endowment.withdraw_approved = false;
            vec![]
        }
        // Has been liquidated or terminated. Remove from Funds and lockdown money flows
        EndowmentStatus::Closed => {
            // set a Beneficiary for the newly closed Endowment to send all funds to
            let beneficiary: Beneficiary;
            if msg.beneficiary.is_some() {
                beneficiary = msg.beneficiary.unwrap();
            } else {
                // query the Index Fund SC to find the Fund that this Endowment is a member of
                let fund_list: angel_core::responses::index_fund::FundListResponse =
                    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: index_fund_contract.clone(),
                        msg: to_binary(
                            &angel_core::messages::index_fund::QueryMsg::InvolvedFunds {
                                endowment_id: msg.endowment_id,
                            },
                        )?,
                    }))?;
                // send funds to the first index fund in list if found
                if !fund_list.funds.is_empty() {
                    beneficiary = Beneficiary::IndexFund {
                        id: fund_list.funds[0].id,
                    };
                } else {
                    // Orphaned Endowment (ie. no index fund)
                    // send funds to the AP treasury
                    beneficiary = Beneficiary::Wallet {
                        address: registrar_config.treasury.to_string(),
                    };
                }
            }
            endowment.deposit_approved = false;
            endowment.withdraw_approved = false;
            vec![
                // trigger the removal of this endowment from all Index Funds
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: index_fund_contract,
                    msg: to_binary(&angel_core::messages::index_fund::ExecuteMsg::RemoveMember(
                        angel_core::messages::index_fund::RemoveMemberMsg {
                            member: msg.endowment_id,
                        },
                    ))
                    .unwrap(),
                    funds: vec![],
                })),
                // start redemption of Account SC's Vault holdings to final beneficiary/index fund
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(
                        &angel_core::messages::accounts::ExecuteMsg::CloseEndowment {
                            id: endowment_id,
                            beneficiary,
                        },
                    )
                    .unwrap(),
                    funds: vec![],
                })),
            ]
        }
        _ => vec![],
    };

    // update entry status & save to the Registry
    endowment.status = msg_endowment_status;
    ENDOWMENTS.save(deps.storage, endowment_id, &endowment)?;

    Ok(Response::new()
        .add_submessages(sub_messages)
        .add_attribute("action", "update_endowment_status"))
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
    new_registrar: String,
    max_general_category_id: u8,
    ibc_controller: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the accounts owner can update the config
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_registrar = deps.api.addr_validate(&new_registrar)?;
    let mut controller = config.ibc_controller;
    if ibc_controller != None {
        controller = deps.api.addr_validate(&ibc_controller.unwrap())?;
    }
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.registrar_contract = new_registrar;
        config.max_general_category_id = max_general_category_id;
        config.ibc_controller = controller;
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
    let mut endowment = ENDOWMENTS.load(deps.storage, msg.id)?;
    let config = CONFIG.load(deps.storage)?;
    let state = STATES.load(deps.storage, msg.id)?;
    if state.closing_endowment {
        return Err(ContractError::UpdatesAfterClosed {});
    }

    if !(info.sender == config.owner || info.sender == endowment.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // Only config.owner can update owner, tier and endowment_type fields
    if info.sender == config.owner {
        endowment.tier = msg.tier;
        if let Some(owner) = msg.owner {
            endowment.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(endow_type) = msg.endow_type {
            endowment.endow_type = match endow_type.as_str() {
                "charity" => EndowmentType::Charity,
                "normal" => EndowmentType::Normal,
                _ => return Err(ContractError::InvalidInputs {}),
            };
        }
    }

    // Only endowment.owner can update all other fields
    if info.sender == endowment.owner {
        if let Some(name) = msg.name.clone() {
            endowment.name = name;
        }
        if let Some(categories) = msg.categories {
            // check that at least 1 SDG category is set for charity endowments
            if endowment.endow_type == EndowmentType::Charity {
                if endowment.categories.sdgs.is_empty() {
                    return Err(ContractError::InvalidInputs {});
                }
                endowment.categories.sdgs.sort();
                for item in endowment.categories.sdgs.clone().into_iter() {
                    if item > 17 || item == 0 {
                        return Err(ContractError::InvalidInputs {});
                    }
                }
            }
            if !endowment.categories.general.is_empty() {
                endowment.categories.general.sort();
                if endowment.categories.general.last().unwrap() > &config.max_general_category_id {
                    return Err(ContractError::InvalidInputs {});
                }
            }
            endowment.categories = categories;
        }
        if let Some(kyc_donors_only) = msg.kyc_donors_only.clone() {
            endowment.kyc_donors_only = kyc_donors_only;
        }
        if let Some(logo) = msg.logo.clone() {
            endowment.logo = Some(logo);
        }
        if let Some(image) = msg.image.clone() {
            endowment.image = Some(image);
        }
    }

    ENDOWMENTS.save(deps.storage, msg.id, &endowment)?;

    Ok(Response::new().add_attribute("action", "update_endowment_settings"))
}

pub fn update_strategies(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    strategies: Vec<Strategy>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    let state = STATES.load(deps.storage, id)?;
    if state.closing_endowment {
        return Err(ContractError::UpdatesAfterClosed {});
    }

    if endowment.pending_redemptions != 0 {
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

    // Check that all strategies supplied can be invested in by this type of Endowment
    // ie. There are no restricted or non-approved vaults in the proposed Strategies setup
    let allowed: VaultListResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.registrar_contract.to_string(),
        msg: to_binary(&RegistrarQuerier::VaultList {
            approved: Some(true),
            endowment_type: Some(endowment.endow_type.clone()),
            acct_type: Some(acct_type.clone()),
            vault_type: None,
            network: None,
            start_after: None,
            limit: None,
        })?,
    }))?;

    let mut percentages_sum = Decimal::zero();

    for strategy in strategies.iter() {
        match allowed
            .vaults
            .iter()
            .position(|v| v.address == strategy.vault)
        {
            None => return Err(ContractError::InvalidInputs {}),
            Some(_) => percentages_sum += strategy.percentage,
        }
    }

    // An endowment cannot have over 100% of strategy allocations
    // Sub-100%: leftover goes into "Tokens on Hand"
    if percentages_sum > Decimal::one() {
        return Err(ContractError::InvalidStrategyAllocation {});
    }

    // update endowment strategies attribute with all newly passed strategies
    let mut new_strategies = vec![];
    for strategy in strategies {
        new_strategies.push(StrategyComponent {
            vault: deps.api.addr_validate(&strategy.vault.clone())?.to_string(),
            percentage: strategy.percentage,
        });
    }

    endowment.copycat_strategy = None;
    endowment
        .strategies
        .set(acct_type.clone(), new_strategies.clone());
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    // If this Endowment that is changing their strategy is also being "copycatted"
    // by other endowments, the new strategy needs to be updated on those endowments.
    let copiers = COPYCATS.load(deps.storage, id).unwrap_or_default();
    for i in copiers.iter() {
        let mut e = ENDOWMENTS.load(deps.storage, *i).unwrap();
        e.strategies.set(acct_type.clone(), new_strategies.clone());
        ENDOWMENTS.save(deps.storage, *i, &e).unwrap();
    }
    Ok(Response::new().add_attribute("action", "update_strategies"))
}

pub fn copycat_strategies(
    deps: DepsMut,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    id_to_copy: u32,
) -> Result<Response, ContractError> {
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;
    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let copied_endowment = ENDOWMENTS.load(deps.storage, id_to_copy)?;
    if copied_endowment.strategies.get(acct_type).is_empty() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Attempting to copy an endowment with no set strategy for that account type"
                .to_string(),
        }));
    }

    if endowment.copycat_strategy == Some(id_to_copy) {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Attempting re-set the same copycat endowment ID".to_string(),
        }));
    }
    // if this endowment was already copying another prior to this new one,
    // first remove it from the old list and add to the new copycat list
    if endowment.copycat_strategy.is_some() {
        let old_id = endowment.copycat_strategy.unwrap();
        let mut old_copiers = COPYCATS.load(deps.storage, old_id)?;
        if let Some(pos) = old_copiers.iter().position(|i| *i == id) {
            old_copiers.swap_remove(pos);
        }
        COPYCATS.save(deps.storage, old_id, &old_copiers)?;
    }

    // add this endowment to the new Copycat list
    let mut copiers = COPYCATS.load(deps.storage, id_to_copy)?;
    copiers.push(id);
    COPYCATS.save(deps.storage, id_to_copy, &copiers)?;

    // set new copycat id
    endowment.copycat_strategy = Some(id_to_copy);
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    Ok(Response::new())
}

pub fn swap_token(
    deps: DepsMut,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    amount: Uint128,
    operations: Vec<SwapOperation>,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() || operations.is_empty() {
        return Err(ContractError::InvalidInputs {});
    }

    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    let mut state = STATES.load(deps.storage, id)?;
    let offer_asset = match operations.first().unwrap() {
        SwapOperation::JunoSwap {
            offer_asset_info, ..
        } => offer_asset_info,
        SwapOperation::Loop {
            offer_asset_info, ..
        } => offer_asset_info,
    };

    match (offer_asset, acct_type.clone()) {
        (AssetInfo::Native(denom), AccountType::Liquid) => {
            if state
                .balances
                .liquid
                .get_denom_amount(denom.to_string())
                .amount
                < amount
            {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .liquid
                .deduct_tokens(Balance::from(vec![Coin {
                    amount,
                    denom: denom.to_string(),
                }]));
        }
        (AssetInfo::Native(denom), AccountType::Locked) => {
            if state
                .balances
                .locked
                .get_denom_amount(denom.to_string())
                .amount
                < amount
            {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .locked
                .deduct_tokens(Balance::from(vec![Coin {
                    amount,
                    denom: denom.to_string(),
                }]));
        }
        (AssetInfo::Cw20(addr), AccountType::Liquid) => {
            if state.balances.liquid.get_token_amount(addr.clone()).amount < amount {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .liquid
                .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr.clone(),
                    amount,
                }));
        }
        (AssetInfo::Cw20(addr), AccountType::Locked) => {
            if state.balances.locked.get_token_amount(addr.clone()).amount < amount {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .locked
                .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr.clone(),
                    amount,
                }));
        }
        _ => unreachable!(),
    }

    let swap_msg: CosmosMsg = match offer_asset {
        AssetInfo::Native(denom) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registrar_config.swaps_router.unwrap(),
            msg: to_binary(&SwapRouterExecuteMsg::ExecuteSwapOperations {
                endowment_id: id,
                acct_type,
                operations: operations.clone(),
                minimum_receive: None,
            })
            .unwrap(),
            funds: vec![Coin {
                amount,
                denom: denom.to_string(),
            }],
        }),
        AssetInfo::Cw20(addr) => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addr.clone().to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: registrar_config.swaps_router.unwrap(),
                amount,
                msg: to_binary(&SwapRouterExecuteMsg::ExecuteSwapOperations {
                    endowment_id: id,
                    acct_type,
                    operations,
                    minimum_receive: None,
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }),
        _ => unreachable!(),
    };
    STATES.save(deps.storage, id, &state)?;
    Ok(Response::new().add_message(swap_msg))
}

pub fn swap_receipt(
    deps: DepsMut,
    id: u32,
    sender_addr: Addr,
    final_asset: Asset,
    acct_type: AccountType,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    if sender_addr != registrar_config.swaps_router.unwrap() {
        return Err(ContractError::Unauthorized {});
    }

    let mut state = STATES.load(deps.storage, id)?;
    match (final_asset.info, acct_type) {
        (AssetInfo::Native(denom), AccountType::Liquid) => {
            state.balances.liquid.add_tokens(Balance::from(vec![Coin {
                amount: final_asset.amount,
                denom,
            }]))
        }
        (AssetInfo::Native(denom), AccountType::Locked) => {
            state.balances.locked.add_tokens(Balance::from(vec![Coin {
                amount: final_asset.amount,
                denom,
            }]))
        }
        (AssetInfo::Cw20(addr), AccountType::Liquid) => {
            state
                .balances
                .liquid
                .add_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr,
                    amount: final_asset.amount,
                }))
        }
        (AssetInfo::Cw20(addr), AccountType::Locked) => {
            state
                .balances
                .locked
                .add_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr,
                    amount: final_asset.amount,
                }))
        }
        _ => unreachable!(),
    }
    STATES.save(deps.storage, id, &state)?;
    Ok(Response::new())
}

pub fn distribute_to_beneficiary(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATES.load(deps.storage, id)?;

    // Consolidate all locked & liquid assets for the closing endowment if going to a wallet,
    // otherwise keep the locked & liquid division preserved.
    let mut msgs: Vec<SubMsg> = vec![];
    match state.closing_beneficiary {
        None => (),
        Some(Beneficiary::Wallet { ref address }) => {
            // build msg for all native coins
            let native_coins: Vec<Coin> = [
                state.balances.liquid.native.clone(),
                state.balances.locked.native.clone(),
            ]
            .concat();
            if !native_coins.is_empty() {
                msgs.push(SubMsg::new(BankMsg::Send {
                    to_address: address.to_string(),
                    amount: native_coins,
                }));
            }

            // build list of all CW20 coins
            let cw20_coins: Vec<Cw20Coin> = [
                state.balances.liquid.cw20_list(),
                state.balances.locked.cw20_list(),
            ]
            .concat();
            // create a transfer msg for each CW20 coin
            for coin in cw20_coins.iter() {
                if !coin.amount.is_zero() {
                    msgs.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: coin.address.to_string(),
                        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                            recipient: address.to_string(),
                            amount: coin.amount,
                        })
                        .unwrap(),
                        funds: vec![],
                    })));
                }
            }
        }
        Some(Beneficiary::Endowment { id }) => {
            let mut rcv_endow = STATES.load(deps.storage, id)?;
            rcv_endow
                .balances
                .locked
                .receive_generic_balance(state.balances.locked);
            rcv_endow
                .balances
                .liquid
                .receive_generic_balance(state.balances.liquid);
            STATES.save(deps.storage, id, &rcv_endow)?;
        }
        Some(Beneficiary::IndexFund { id }) => {
            // get index fund addr from registrar
            let registrar_config: RegistrarConfigResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.registrar_contract.to_string(),
                    msg: to_binary(&RegistrarQuerier::Config {})?,
                }))?;
            // get index fund members list & count
            let index_fund: angel_core::responses::index_fund::FundDetailsResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: registrar_config.index_fund.unwrap(),
                    msg: to_binary(&angel_core::messages::index_fund::QueryMsg::FundDetails {
                        fund_id: id,
                    })?,
                }))?;
            let members = index_fund.fund.unwrap().members;
            let members_count = Uint128::from(members.len() as u8);
            // split up endoment locked/liquid balances based on member count
            let split_liquid: GenericBalance = state.balances.liquid.split_balance(members_count);
            let split_locked: GenericBalance = state.balances.locked.split_balance(members_count);
            // transfer split funds portons to each member
            for member in members.into_iter() {
                let mut rcv_endow = STATES.load(deps.storage, member)?;
                rcv_endow
                    .balances
                    .locked
                    .receive_generic_balance(split_locked.clone());
                rcv_endow
                    .balances
                    .liquid
                    .receive_generic_balance(split_liquid.clone());
                STATES.save(deps.storage, member, &rcv_endow)?;
            }
        }
    }

    // zero out the closing endowment's balances
    state.balances = BalanceInfo::default();
    STATES.save(deps.storage, id, &state)?;

    Ok(Response::default().add_submessages(msgs))
}

pub fn vault_receipt(
    deps: DepsMut,
    env: Env,
    id: u32,
    acct_type: AccountType,
    sender_addr: Addr,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATES.load(deps.storage, id)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;

    // check that the returned token came from an Vault contract in our Registrar
    let _vault: VaultDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Vault {
                vault_addr: sender_addr.to_string(),
            })?,
        }))?;

    let returned_token =
        validate_deposit_fund(deps.as_ref(), config.registrar_contract.as_str(), fund)?;

    // add returned tokens back to that endowment's balance
    let returned_bal = match returned_token.info {
        AssetInfoBase::Native(ref denom) => Balance::from(vec![Coin {
            denom: denom.to_string(),
            amount: returned_token.amount,
        }]),
        AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: returned_token.amount,
        }),
        _ => unreachable!(),
    };
    match acct_type {
        AccountType::Locked => state.balances.locked.add_tokens(returned_bal),
        AccountType::Liquid => state.balances.liquid.add_tokens(returned_bal),
    }

    STATES.save(deps.storage, id, &state)?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    match endowment.pending_redemptions {
        // nothing pending, no action needed
        0 => (),
        1 => {
            // reset pending redemptions
            endowment.pending_redemptions = 0;
            // if the endowment is also closing, distribute all funds to beneficiary
            if state.closing_endowment {
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::DistributeToBeneficiary { id })?,
                    funds: vec![],
                }));
            }
        }
        // deduct pending redemptions as they come in
        _ => endowment.pending_redemptions -= 1,
    }
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    Ok(Response::new().add_attribute("action", "vault_receipt"))
}

pub fn reinvest_to_locked(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    amount: Uint128,
    vault_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENTS.load(deps.storage, id)?;

    // check that sender is the owner or the beneficiary
    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // ensure we have a non-zero amount and a valid vault target
    let vault_config: VaultDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Vault {
                vault_addr: vault_addr.clone(),
            })?,
        }))?;
    let yield_vault: YieldVault = vault_config.vault;
    if amount.is_zero() || !yield_vault.approved || yield_vault.acct_type.ne(&AccountType::Liquid) {
        return Err(ContractError::InvalidInputs {});
    }

    let msg: SubMsg;
    match yield_vault.vault_type {
        VaultType::Native => {
            msg = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: vault_addr,
                msg: to_binary(&angel_core::messages::vault::ExecuteMsg::ReinvestToLocked {
                    endowment_id: id,
                    amount,
                })
                .unwrap(),
                funds: vec![],
            }));
        }
        VaultType::Ibc { ica: _ } => unimplemented!(),
        VaultType::Evm => unimplemented!(),
    }
    Ok(Response::new().add_submessage(msg))
}

pub fn deposit(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    sender_addr: Addr,
    msg: DepositMsg,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENTS.load(deps.storage, msg.id)?;

    // check that the Endowment has been approved to receive deposits
    if !endowment.deposit_approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Deposits are not approved for this endowment".to_string(),
        }));
    }

    // check that the split %s sum to 1
    if msg.locked_percentage + msg.liquid_percentage != Decimal::one() {
        return Err(ContractError::InvalidSplit {});
    }

    // Check the token with "accepted_tokens"
    let deposit_token =
        validate_deposit_fund(deps.as_ref(), config.registrar_contract.as_str(), fund)?;
    let deposit_amount = deposit_token.amount;

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
    if sender_addr != index_fund {
        let new_splits = check_splits(registrar_split_configs, locked_split, liquid_split);
        locked_split = new_splits.0;
        liquid_split = new_splits.1;
    }

    let locked_amount = Asset {
        info: deposit_token.info.clone(),
        amount: deposit_amount * locked_split,
    };
    let liquid_amount = Asset {
        info: deposit_token.info,
        amount: deposit_amount * liquid_split,
    };

    // update total donations recieved for a charity
    let mut state: State = STATES.load(deps.storage, msg.id)?;
    state.donations_received.locked += locked_amount.amount;
    state.donations_received.liquid += liquid_amount.amount;

    let mut deposit_messages: Vec<SubMsg> = vec![];

    // Process Locked Strategy Deposits
    let locked_strategies = endowment.strategies.get(AccountType::Locked);
    // build deposit messages for each of the sources/amounts
    let (messages, leftover_amt) = deposit_to_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        msg.id,
        locked_amount.clone(),
        &locked_strategies,
    )?;
    for m in messages.iter() {
        deposit_messages.push(m.clone());
    }
    // If invested portion of strategies < 100% there will be leftover deposits
    // Add any remaining deposited tokens to the locked balance "Tokens on Hand"
    state.balances.locked.add_tokens(match locked_amount.info {
        AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
            denom,
            amount: leftover_amt,
        }]),
        AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: leftover_amt,
        }),
        _ => unreachable!(),
    });

    // Process Liquid Strategy Deposits
    let liquid_strategies = endowment.strategies.get(AccountType::Liquid);
    // build deposit messages for each of the sources/amounts
    let (messages, leftover_amt) = deposit_to_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        msg.id,
        liquid_amount.clone(),
        &liquid_strategies,
    )?;
    for m in messages.iter() {
        deposit_messages.push(m.clone());
    }
    // If invested portion of strategies < 100% there will be leftover deposits
    // Add any remaining deposited tokens to the liquid balance "Tokens on Hand"
    state.balances.liquid.add_tokens(match liquid_amount.info {
        AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
            denom,
            amount: leftover_amt,
        }]),
        AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: leftover_amt,
        }),
        _ => unreachable!(),
    });

    STATES.save(deps.storage, msg.id, &state)?;
    Ok(Response::new()
        .add_submessages(deposit_messages)
        .add_attribute("action", "account_deposit"))
}

/// Allow Endowment owners to invest some amount of their free balance
/// "Tokens on Hand" holdings into Vault(s). Does not have to be a Vault
/// that exists in their donation Strategy. One-time/one-off investment.
pub fn vaults_invest(
    deps: DepsMut,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    vaults: Vec<(String, Asset)>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;
    let mut state = STATES.load(deps.storage, id)?;
    let mut current_bal: GenericBalance = state.balances.get(&acct_type);

    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if vaults.is_empty() {
        return Err(ContractError::InvalidInputs {});
    }

    // iterate over each vault and asset passed in
    // 1. Validate that Vault addr and input Asset are valid
    // 2. Check that TOH for AcctType has enough tokens to cover deposit amt
    // 3. Create deposit message to Vault
    let mut deposit_msgs: Vec<SubMsg> = vec![];
    for (vault, asset) in vaults.iter() {
        // check vault addr passed is valid
        let vault_addr = deps.api.addr_validate(vault)?;

        // check vault is in registrar vaults list and is approved
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: vault_addr.clone().to_string(),
                })?,
            }))?;

        if !vault_config.vault.approved {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Vault is not approved to accept deposits".to_string(),
            }));
        }
        let token_denom: String = match &asset.info {
            AssetInfo::Native(denom) => denom.clone(),
            AssetInfo::Cw20(addr) => addr.to_string(),
            _ => unimplemented!(),
        };

        // check that the vault input token matches Asset to deposit
        if vault_config.vault.input_denom != token_denom {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Asset given is not a valid for Vault input".to_string(),
            }));
        }

        if vault_config.vault.acct_type != acct_type {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Vault and Endowment AccountTypes do not match".to_string(),
            }));
        }

        // add vault to the one-off-vaults list if a new vault
        match acct_type {
            AccountType::Locked => {
                let pos = endowment
                    .oneoff_vaults
                    .locked
                    .iter()
                    .position(|v| v == vault);
                if pos.is_some() {
                    endowment.oneoff_vaults.locked.push(vault_addr.clone());
                }
            }
            AccountType::Liquid => {
                let pos = endowment
                    .oneoff_vaults
                    .liquid
                    .iter()
                    .position(|v| v == vault);
                if pos.is_some() {
                    endowment.oneoff_vaults.liquid.push(vault_addr.clone());
                }
            }
        }

        // check that the token balance on hand is enough to cover the deposit amount
        // fetch the amount of an asset held in the state balance
        let token_balance: Uint128 = match asset.info.clone() {
            AssetInfo::Native(denom) => current_bal.get_denom_amount(denom).amount,
            AssetInfo::Cw20(addr) => current_bal.get_token_amount(addr).amount,
            _ => unreachable!(),
        };
        // check that the amount in state balance is sufficient to cover withdraw request
        if asset.amount > token_balance {
            return Err(ContractError::InsufficientFunds {});
        }

        // deduct the tokens from the state's current balance
        match asset.info.clone() {
            AssetInfo::Native(denom) => current_bal.deduct_tokens(Balance::from(vec![Coin {
                denom: denom.clone(),
                amount: asset.amount,
            }])),
            AssetInfo::Cw20(addr) => current_bal.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
                amount: asset.amount,
                address: addr,
            })),
            _ => unreachable!(),
        }

        // create a deposit message for the vault
        // funds payload can contain CW20 | Native token amounts
        match vault_config.vault.vault_type {
            VaultType::Native => {
                deposit_msgs.push(match &asset.info {
                    AssetInfoBase::Native(ref denom) => {
                        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: vault_addr.clone().to_string(),
                            msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Deposit {
                                endowment_id: id,
                            })
                            .unwrap(),
                            funds: vec![Coin {
                                denom: denom.clone(),
                                amount: asset.amount,
                            }],
                        }))
                    }
                    AssetInfo::Cw20(ref contract_addr) => {
                        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: contract_addr.to_string(),
                            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                                contract: vault_addr.clone().to_string(),
                                amount: asset.amount,
                                msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Deposit {
                                    endowment_id: id,
                                })
                                .unwrap(),
                            })
                            .unwrap(),
                            funds: vec![],
                        }))
                    }
                    _ => unreachable!(),
                });
            }
            VaultType::Ibc { ica: _ } => unimplemented!(),
            VaultType::Evm => unimplemented!(),
        }
    }

    // set the final state balance after all assets have been deducted and save
    match &acct_type {
        AccountType::Locked => state.balances.locked = current_bal.clone(),
        AccountType::Liquid => state.balances.liquid = current_bal.clone(),
    }
    STATES.save(deps.storage, id, &state)?;

    Ok(Response::new()
        .add_attribute("action", "vault_invest")
        .add_submessages(deposit_msgs))
}

/// Allow Endowment owners to redeem some amount of Vault tokens back to their
/// Locked Balance "Tokens on Hand" holdings
pub fn vaults_redeem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    vaults: Vec<(String, Uint128)>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;

    if vaults.is_empty() {
        return Err(ContractError::InvalidInputs {});
    }

    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.pending_redemptions != 0 {
        return Err(ContractError::RedemptionInProgress {});
    }

    // iterate over each vault and amount passed in
    // 1. Validate that Vault addr and input Asset are valid
    // 2. Create redeem message to Vault
    let mut redeem_msgs: Vec<SubMsg> = vec![];
    for (vault, amount) in vaults.iter() {
        // check vault addr passed is valid
        let vault_addr = deps.api.addr_validate(vault)?.to_string();

        // check vault is in registrar vaults list and is approved
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: vault_addr.clone(),
                })?,
            }))?;

        if vault_config.vault.acct_type != acct_type {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Vault and Endowment AccountTypes do not match".to_string(),
            }));
        }

        // check if the vault tokens have been depleted and remove one-off-vault from list if so
        let vault_balance = vault_endowment_balance(deps.as_ref(), vault.to_string(), id);
        match acct_type {
            AccountType::Locked => {
                let pos = endowment
                    .oneoff_vaults
                    .locked
                    .iter()
                    .position(|v| v == vault);
                if pos.is_some() && vault_balance == *amount {
                    endowment.oneoff_vaults.locked.swap_remove(pos.unwrap());
                }
            }
            AccountType::Liquid => {
                let pos = endowment
                    .oneoff_vaults
                    .liquid
                    .iter()
                    .position(|v| v == vault);
                if pos.is_some() && vault_balance == *amount {
                    endowment.oneoff_vaults.liquid.swap_remove(pos.unwrap());
                }
            }
        }

        match vault_config.vault.vault_type {
            VaultType::Native => {
                // Check the vault token(VT) balance
                let available_vt: Uint128 = deps.querier.query_wasm_smart(
                    vault_addr.to_string(),
                    &angel_core::messages::vault::QueryMsg::Balance { endowment_id: id },
                )?;
                if *amount > available_vt {
                    return Err(ContractError::BalanceTooSmall {});
                }
                redeem_msgs.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: vault_addr,
                    msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Redeem {
                        endowment_id: id,
                        amount: *amount,
                    })
                    .unwrap(),
                    funds: vec![],
                })));
            }
            VaultType::Ibc { ica: _ } => unimplemented!(),
            VaultType::Evm => unimplemented!(),
        }
    }

    ENDOWMENTS.save(deps.storage, id, &endowment)?;
    Ok(Response::new()
        .add_attribute("action", "vault_redeem")
        .add_submessages(redeem_msgs))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    beneficiary: String,
    assets: Vec<AssetUnchecked>,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATES.load(deps.storage, id)?;
    let mut state_bal: GenericBalance = state.balances.get(&acct_type);
    let mut messages: Vec<SubMsg> = vec![];
    let mut native_coins: Vec<Coin> = vec![];

    if !endowment.withdraw_approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Withdraws are not approved for this endowment".to_string(),
        }));
    }

    // check that sender is correct based on account type attempting to access
    // Only config owner can authorize a locked balance withdraw when locks are in place or maturity is not reached
    // Only the endowment owner can authorize a locked balance withdraw once maturity is reached or if early withdraws are allowed
    if acct_type == AccountType::Locked {
        if let EndowmentType::Charity = endowment.endow_type {
            if info.sender != config.owner {
                return Err(ContractError::Unauthorized {});
            }
        } else {
            if info.sender != endowment.owner {
                return Err(ContractError::Unauthorized {});
            }
            if !endowment.withdraw_before_maturity
                || (endowment.maturity_time != None
                    && Timestamp::from_seconds(endowment.maturity_time.unwrap()) <= env.block.time)
            {
                return Err(ContractError::Std(StdError::generic_err(
                    "Endowment is not mature. Cannot withdraw before maturity time is reached.",
                )));
            }
        }
    }
    // Only the owner of an endowment w/ withdraws approved can remove liquid balances
    if acct_type == AccountType::Liquid {
        if info.sender != endowment.owner {
            return Err(ContractError::Unauthorized {});
        }
    }

    for asset in assets.iter() {
        asset.check(deps.api, None)?;
        // check for assets with zero amounts and raise error if found
        if asset.amount.is_zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        // fetch the amount of an asset held in the state balance
        let balance: Uint128 = match asset.info.clone() {
            AssetInfoBase::Native(denom) => state_bal.get_denom_amount(denom).amount,
            AssetInfoBase::Cw20(addr) => {
                state_bal
                    .get_token_amount(deps.api.addr_validate(&addr).unwrap())
                    .amount
            }
            _ => unreachable!(),
        };
        // check that the amount in state balance is sufficient to cover withdraw request
        if asset.amount > balance {
            return Err(ContractError::InsufficientFunds {});
        }

        // build message based on asset type and update state balance with deduction
        match asset.info.clone() {
            AssetInfoBase::Native(denom) => {
                // add Coin to the native coins vector to have a message built
                // and all deductions against the state balance done at the end
                native_coins.push(Coin {
                    denom: denom.clone(),
                    amount: asset.amount,
                });
            }
            AssetInfoBase::Cw20(addr) => {
                // Build message to transfer CW20 tokens to the Beneficiary
                messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                        recipient: beneficiary.to_string(),
                        amount: asset.amount,
                    })
                    .unwrap(),
                    funds: vec![],
                })));
                // Update a CW20 token's Balance in STATE
                state_bal.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
                    amount: asset.amount,
                    address: deps.api.addr_validate(&addr).unwrap(),
                }));
            }
            _ => unreachable!(),
        }
    }

    // build the native Coin BankMsg if needed
    if !native_coins.is_empty() {
        // deduct the native coins withdrawn against balances held in state
        state_bal.deduct_tokens(Balance::from(native_coins.clone()));
        // Build message to send all native tokens to the Beneficiary via BankMsg::Send
        messages.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: beneficiary,
            amount: native_coins,
        })));
    }

    // set the updated balance for the account type
    match acct_type {
        AccountType::Locked => state.balances.locked = state_bal,
        AccountType::Liquid => state.balances.liquid = state_bal,
    }
    STATES.save(deps.storage, id, &state)?;

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "withdraw"))
}

pub fn close_endowment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    beneficiary: Beneficiary,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;
    if endowment.pending_redemptions != 0 {
        return Err(ContractError::RedemptionInProgress {});
    }

    // set the STATE with relevent status and closing beneficiary
    let mut state = STATES.load(deps.storage, id)?;
    state.closing_endowment = true;
    state.closing_beneficiary = Some(beneficiary);
    STATES.save(deps.storage, id, &state)?;

    // Redeem all funds back from vaults that an Endowment is invested in
    let mut all_vaults: Vec<String> = [
        [
            endowment.oneoff_vaults.get(AccountType::Liquid),
            endowment.oneoff_vaults.get(AccountType::Locked),
        ]
        .concat()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>(),
        [
            endowment.strategies.get(AccountType::Liquid),
            endowment.strategies.get(AccountType::Locked),
        ]
        .concat()
        .iter()
        .map(|s| s.vault.clone())
        .collect(),
    ]
    .concat();

    all_vaults.sort();
    all_vaults.dedup();

    let mut redeem_messages = vec![];
    for vault in all_vaults.iter() {
        // create a redeem message for Vault, noting amount of tokens
        let vault_balance = vault_endowment_balance(deps.as_ref(), vault.clone(), id);
        redeem_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: vault.to_string(),
            msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Redeem {
                endowment_id: id,
                amount: vault_balance,
            })
            .unwrap(),
            funds: vec![],
        })));
    }

    endowment.pending_redemptions = redeem_messages.len() as u8;
    endowment.deposit_approved = false;
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    Ok(Response::new()
        .add_attribute("action", "close_endowment")
        .add_submessages(redeem_messages))
}

pub fn update_profile(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateProfileMsg,
) -> Result<Response, ContractError> {
    // Validation 1. Only "Endowment.owner" or "Config.owner" is able to execute
    let endowment = ENDOWMENTS.load(deps.storage, msg.id)?;
    let mut profile = PROFILES.load(deps.storage, msg.id)?;

    // Only endowment.owner can update these fields
    if !(info.sender == endowment.owner) {
        return Err(ContractError::Unauthorized {});
    }

    let state = STATES.load(deps.storage, msg.id)?;
    if state.closing_endowment {
        return Err(ContractError::UpdatesAfterClosed {});
    }

    if let Some(overview) = msg.overview {
        profile.overview = overview;
    }
    let social_media_urls = SocialMedialUrls {
        facebook: msg.facebook,
        twitter: msg.twitter,
        linkedin: msg.linkedin,
    };
    profile.social_media_urls = social_media_urls;
    profile.url = msg.url;
    profile.registration_number = msg.registration_number;
    profile.country_of_origin = msg.country_of_origin;
    profile.street_address = msg.street_address;
    profile.contact_email = msg.contact_email;
    profile.number_of_employees = msg.number_of_employees;
    profile.average_annual_budget = msg.average_annual_budget;
    profile.annual_revenue = msg.annual_revenue;
    profile.charity_navigator_rating = msg.charity_navigator_rating;

    PROFILES.save(deps.storage, msg.id, &profile)?;

    Ok(Response::new().add_attribute("action", "update_profile"))
}
