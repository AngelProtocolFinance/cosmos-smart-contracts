use crate::state::{CONFIG, ENDOWMENT, PROFILE, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
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
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse, VaultListResponse,
};
use angel_core::structs::{
    AcceptedTokens, DaoSetup, DonationMatch, EndowmentFee, EndowmentType, FundingSource,
    GenericBalance, SocialMedialUrls, SplitDetails, StrategyComponent, Tier,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, redeem_from_vaults, validate_deposit_fund,
    withdraw_from_vaults,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Decimal256, DepsMut, Env, Fraction,
    MessageInfo, QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified, Cw20ExecuteMsg};
use cw_asset::{Asset, AssetInfoBase};
use std::str::FromStr;

pub fn contract_setup_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut endowment = ENDOWMENT.load(deps.storage)?;
            let mut res = Response::default();
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "multisig_addr" => {
                                endowment.owner = deps.api.addr_validate(&attrb.value)?;
                                res = res.add_attribute("endow_owner", endowment.owner.to_string())
                            }
                            "dao_addr" => {
                                endowment.dao = Some(deps.api.addr_validate(&attrb.value)?)
                            }
                            "dao_token_addr" => {
                                endowment.dao_token = Some(deps.api.addr_validate(&attrb.value)?)
                            }
                            "donation_match_addr" => {
                                endowment.donation_match_contract =
                                    Some(deps.api.addr_validate(&attrb.value)?)
                            }
                            &_ => (),
                        }
                    }
                }
            }
            ENDOWMENT.save(deps.storage, &endowment)?;
            Ok(res)
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
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

    // only the SC admin can update the registrar in the config
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
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENT.load(deps.storage)?;
    let profile = PROFILE.load(deps.storage)?;

    if info.sender.ne(&endowment.owner)
        && (endowment.dao != None && info.sender != *endowment.dao.as_ref().unwrap())
    {
        return Err(ContractError::Unauthorized {});
    }

    endowment.owner = match msg.owner {
        Some(i) => {
            // only the endowment owner can update these configs
            if info.sender != endowment.owner {
                return Err(ContractError::Unauthorized {});
            }
            deps.api.addr_validate(&i)?
        }
        None => endowment.owner,
    };
    // only normalized endowments can update certain settings (ie. Charity Endowments have more fixed settings)
    if profile.endow_type != EndowmentType::Charity {
        endowment.whitelisted_beneficiaries = match msg.whitelisted_beneficiaries {
            Some(i) => {
                if config
                    .settings_controller
                    .whitelisted_beneficiaries
                    .can_change(&info.sender, &endowment.owner, endowment.dao.as_ref())
                {
                    i
                } else {
                    endowment.whitelisted_beneficiaries
                }
            }
            None => endowment.whitelisted_beneficiaries,
        };
        endowment.whitelisted_contributors = match msg.whitelisted_contributors {
            Some(i) => {
                if config
                    .settings_controller
                    .whitelisted_contributors
                    .can_change(&info.sender, &endowment.owner, endowment.dao.as_ref())
                {
                    i
                } else {
                    endowment.whitelisted_contributors
                }
            }
            None => endowment.whitelisted_contributors,
        };
        endowment.withdraw_before_maturity = match msg.withdraw_before_maturity {
            Some(i) => {
                if config
                    .settings_controller
                    .whitelisted_contributors
                    .can_change(&info.sender, &endowment.owner, endowment.dao.as_ref())
                {
                    i
                } else {
                    endowment.withdraw_before_maturity
                }
            }
            None => endowment.withdraw_before_maturity,
        };
        endowment.maturity_time = match msg.maturity_time {
            Some(i) => {
                if config.settings_controller.maturity_time.can_change(
                    &info.sender,
                    &endowment.owner,
                    endowment.dao.as_ref(),
                ) {
                    i
                } else {
                    endowment.maturity_time
                }
            }
            None => endowment.maturity_time,
        };
        endowment.rebalance = match msg.rebalance {
            Some(i) => i,
            None => endowment.rebalance,
        };
    }

    // validate address strings passed
    if config.settings_controller.kyc_donors_only.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
        endowment.kyc_donors_only = msg.kyc_donors_only;
    }

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

    let profile = PROFILE.load(deps.storage)?;

    // send the new owner informtion back to the registrar
    Ok(
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarExecuter::UpdateEndowmentEntry(
                UpdateEndowmentEntryMsg {
                    endowment_addr: env.contract.address.to_string(),
                    owner: Some(endowment.owner.to_string()),
                    name: Some(profile.name),
                    logo: profile.logo,
                    image: profile.image,
                    endow_type: Some(profile.endow_type),
                    tier: match profile.tier {
                        Some(1) => Some(Some(Tier::Level1)),
                        Some(2) => Some(Some(Tier::Level2)),
                        Some(3) => Some(Some(Tier::Level3)),
                        None => Some(None),
                        _ => return Err(ContractError::InvalidInputs {}),
                    },
                    un_sdg: match profile.un_sdg {
                        Some(i) => Some(Some(i)),
                        None => Some(None),
                    },
                },
            ))?,
            funds: vec![],
        }))),
    )
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
    let profile = PROFILE.load(deps.storage)?;

    if !config.settings_controller.strategies.can_change(
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

    // Check that all strategies supplied can be invested in by this type of Endowment
    // ie. There are no restricted or non-approved vaults in the proposed Strategies setup
    let allowed: VaultListResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.registrar_contract.to_string(),
        msg: to_binary(&RegistrarQuerier::VaultList {
            approved: Some(true),
            endowment_type: Some(profile.endow_type),
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
            .position(|v| v.address == strategy.vault.to_string())
        {
            None => return Err(ContractError::InvalidInputs {}),
            Some(_) => percentages_sum += strategy.percentage,
        }
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
            vault: deps.api.addr_validate(&strategy.vault.clone())?.to_string(),
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
                    AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
                    AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
                        denom,
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
                    AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
        Some(_) => match config.pending_redemptions.unwrap().checked_sub(1) {
            Some(n) => config.pending_redemptions = Some(n),
            None => config.pending_redemptions = None,
        },
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
    _env: Env,
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
                AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
    if sender_addr != index_fund {
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

    // increase the liquid balance by donation (liquid) amount
    let liquid_balance = match liquid_amount.info {
        AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
            denom,
            amount: liquid_amount.amount,
        }]),
        AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
            address: contract_addr,
            amount: liquid_amount.amount,
        }),
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
    };
    state.balances.liquid_balance.add_tokens(liquid_balance);

    // get the correct donation match contract to use
    let donation_match_contract = match (
        profile.endow_type,
        endowment.donation_match_contract,
        registrar_config.donation_match_charites_contract,
    ) {
        (EndowmentType::Normal, Some(match_contract), _) => Some(match_contract),
        (EndowmentType::Charity, _, Some(charity_match_contract)) => {
            Some(deps.api.addr_validate(&charity_match_contract)?)
        }
        (_, _, _) => None,
    };

    // check if the donation matching is possible
    let mut donor_match_messages: Vec<SubMsg> = vec![];
    if donation_match_contract != None
        && !locked_amount.amount.is_zero()
        && endowment.dao_token.is_some()
    {
        // 10% of "locked_amount" amount
        let donation_match_amount = locked_amount.amount.multiply_ratio(100_u128, 1000_u128);
        locked_amount.amount -= donation_match_amount;

        // build "donor_match" message for donation matching
        match locked_amount.info {
            AssetInfoBase::Native(ref token) => {
                donor_match_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: donation_match_contract.unwrap().to_string(),
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
                donor_match_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                        contract: donation_match_contract.unwrap().to_string(),
                        amount: donation_match_amount,
                        msg: to_binary(&DonationMatchExecMsg::DonorMatch {
                            amount: donation_match_amount,
                            donor: sender_addr.clone(),
                            token: endowment.dao_token.unwrap(),
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                })))
            }
            AssetInfoBase::Cw1155(_, _) => unimplemented!(),
        }
    };

    let deposit_messages;
    let endowment = ENDOWMENT.load(deps.storage)?;
    // check endowment strategies set.
    // if empty: hold locked funds until a vault is set
    if endowment.strategies.is_empty() {
        deposit_messages = vec![];
        // increase the locked balance by locked donation amount
        let locked_balance = match locked_amount.info {
            AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
                denom,
                amount: locked_amount.amount,
            }]),
            AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                address: contract_addr,
                amount: locked_amount.amount,
            }),
            AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
    beneficiary: String,
    sources: Vec<FundingSource>,
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
        if !endowment.maturity_whitelist.is_empty()
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
    let withdraw_messages = withdraw_from_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        &deps.api.addr_validate(&beneficiary)?,
        sources,
    )?;

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
    beneficiary: String,
    assets: GenericBalance,
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

    let mut state = STATE.load(deps.storage)?;
    let mut messages: Vec<SubMsg> = vec![];

    for asset in assets.native.iter() {
        let liquid_balance = state
            .balances
            .liquid_balance
            .get_denom_amount(asset.denom.clone())
            .amount;
        // check that the amount in liquid balance is sufficient to cover request
        if asset.amount > liquid_balance {
            return Err(ContractError::InsufficientFunds {});
        }
    }
    // Build message to send all native tokens to the Beneficiary via BankMsg::Send
    messages.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: beneficiary.to_string(),
        amount: assets.native.clone(),
    })));
    // Update the native tokens Liquid Balance in STATE
    state
        .balances
        .liquid_balance
        .deduct_tokens(Balance::from(assets.native));

    for asset in assets.cw20.into_iter() {
        let liquid_balance = state
            .balances
            .liquid_balance
            .get_token_amount(asset.address.clone())
            .amount;
        // check that the amount in liquid balance is sufficient to cover request
        if asset.amount > liquid_balance {
            return Err(ContractError::InsufficientFunds {});
        }
        // Build message to send a CW20 tokens to the Beneficiary via CW20::Transfer
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: asset.address.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                recipient: beneficiary.to_string(),
                amount: asset.amount,
            })
            .unwrap(),
            funds: vec![],
        })));
        // Update a CW20 token's Liquid Balance in STATE
        state
            .balances
            .liquid_balance
            .deduct_tokens(Balance::Cw20(asset));
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "withdraw_liquid")
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

    // Check that the info.sender is not one of the usual suspects who are allowed to poke around here.
    if info.sender.ne(&config.owner) && info.sender.ne(&endowment.owner) {
        if endowment.dao == None || info.sender.ne(endowment.dao.as_ref().unwrap()) {
            return Err(ContractError::Unauthorized {});
        }
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
    if config.settings_controller.profile.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
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
                owner: Some(endowment.owner.to_string()),
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
    let mut endowment = ENDOWMENT.load(deps.storage)?;
    let profile = PROFILE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // only normalized endowments can update the additional fees
    if profile.endow_type != EndowmentType::Charity {
        return Err(ContractError::Std(StdError::generic_err(
            "Charity Endowments may not change endowment fees",
        )));
    }

    // Update the "EndowmentFee"s
    if config.settings_controller.earnings_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
        endowment.earnings_fee = msg.earnings_fee;
    }

    if config.settings_controller.deposit_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
        endowment.deposit_fee = msg.deposit_fee;
    }

    if config.settings_controller.withdraw_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
        endowment.withdraw_fee = msg.withdraw_fee;
    }

    if config.settings_controller.aum_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
    ) {
        endowment.aum_fee = msg.aum_fee;
    }

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
    // harvest can only be valid if it comes from the (AP Team/DANO) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    let vault_addr = deps.api.addr_validate(&vault_addr)?;
    Ok(Response::new()
        .add_submessage(SubMsg {
            id: 1,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: vault_addr.to_string(),
                msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Harvest {
                    last_earnings_harvest: config.last_earnings_harvest,
                    last_harvest_fx: config.last_harvest_fx,
                })
                .unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        })
        .add_attribute("action", "harvest"))
}

pub fn harvest_aum(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // only normalized endowments can update certain settings (ie. Charity Endowments have more fixed settings)
    let profile = PROFILE.load(deps.storage)?;
    if profile.endow_type != EndowmentType::Charity {
        return Err(ContractError::Std(StdError::generic_err(
            "Charity Endowments do not have AUM fees to harvest",
        )));
    }

    // Validations
    let endowment = ENDOWMENT.load(deps.storage)?;
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
    let vaults: Vec<String> = endowment
        .strategies
        .iter()
        .map(|s| s.vault.clone())
        .collect();
    for vault in vaults {
        let vault_balances: Uint128 = deps.querier.query_wasm_smart(
            vault.clone(),
            &VaultQueryMsg::Balance {
                address: vault.clone(),
            },
        )?;
        // Here, we assume that only one native coin -
        // `UST` is used for deposit/withdraw in vault
        let mut total_aum: Uint128 = Uint128::zero();
        total_aum += vault_balances;

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
            let mut config = CONFIG.load(deps.storage)?;
            for event in subcall.events {
                if event.ty == "wasm" {
                    for attrb in event.attributes {
                        if attrb.key == "last_earnings_harvest" {
                            config.last_earnings_harvest = attrb.value.parse::<u64>().unwrap();
                        }
                        if attrb.key == "last_harvest_fx" {
                            config.last_harvest_fx =
                                Some(Decimal256::from_str(&attrb.value).unwrap());
                        }
                    }
                }
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default())
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn setup_dao(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: DaoSetup,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let profile = PROFILE.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.dao != None {
        return Err(ContractError::Std(StdError::generic_err(
            "A DAO already exists for this Endowment",
        )));
    }

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    Ok(Response::new().add_submessage(SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: registrar_config.subdao_gov_code.unwrap(),
            admin: None,
            label: "new endowment dao contract".to_string(),
            msg: to_binary(&angel_core::messages::subdao::InstantiateMsg {
                quorum: msg.quorum,
                threshold: msg.threshold,
                voting_period: msg.voting_period,
                timelock_period: msg.timelock_period,
                expiration_period: msg.expiration_period,
                proposal_deposit: msg.proposal_deposit,
                snapshot_period: msg.snapshot_period,
                token: msg.token,
                endow_type: profile.endow_type,
                endow_owner: endowment.owner.to_string(),
                registrar_contract: config.registrar_contract.to_string(),
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }))
}

pub fn setup_donation_match(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    setup: DonationMatch,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENT.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.dao == None {
        return Err(ContractError::Std(StdError::generic_err(
            "A DAO does not exist yet for this Endowment. Please set that up first.",
        )));
    }

    if endowment.donation_match_contract != None {
        return Err(ContractError::Std(StdError::generic_err(
            "A Donation Match contract already exists for this Endowment",
        )));
    }

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    let mut res = Response::default();
    let match_code = match registrar_config.donation_match_code {
        Some(match_code) => match_code,
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "No code id for donation matching contract".to_string(),
            }))
        }
    };
    match setup {
        DonationMatch::HaloTokenReserve {} => {
            match (
                registrar_config.halo_token,
                registrar_config.halo_token_lp_contract,
            ) {
                (Some(reserve_addr), Some(lp_addr)) => {
                    res = res.add_submessage(SubMsg {
                        id: 0,
                        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                            code_id: match_code,
                            admin: None,
                            label: "new donation match contract".to_string(),
                            msg: to_binary(
                                &angel_core::messages::donation_match::InstantiateMsg {
                                    reserve_token: reserve_addr,
                                    lp_pair: lp_addr,
                                    registrar_contract: config.registrar_contract.to_string(),
                                },
                            )?,
                            funds: vec![],
                        }),
                        gas_limit: None,
                        reply_on: ReplyOn::Success,
                    });
                }
                _ => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "HALO Token is not setup to be a reserve token".to_string(),
                    }))
                }
            }
        }
        DonationMatch::Cw20TokenReserve {
            reserve_addr,
            lp_addr,
        } => {
            res = res.add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: match_code,
                    admin: None,
                    label: "new donation match contract".to_string(),
                    msg: to_binary(&angel_core::messages::donation_match::InstantiateMsg {
                        reserve_token: reserve_addr,
                        lp_pair: lp_addr,
                        registrar_contract: config.registrar_contract.to_string(),
                    })?,
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            });
        }
    }

    Ok(res)
}
