use crate::state::{CONFIG, ENDOWMENT, PROFILE, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::index_fund::{
    DepositMsg as IndexFundDepositMsg, ExecuteMsg as IndexFundExecuter,
    QueryMsg as IndexFundQuerier,
};
use angel_core::messages::registrar::{
    ExecuteMsg as RegistrarExecuter, QueryMsg as RegistrarQuerier, UpdateEndowmentEntryMsg,
};
use angel_core::responses::index_fund::FundListResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse,
};
use angel_core::structs::{
    EndowmentType, FundingSource, SocialMedialUrls, SplitDetails, StrategyComponent, Tier,
    TransactionRecord,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, redeem_from_vaults, validate_deposit_fund,
    withdraw_from_vaults,
};
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified};
use cw_asset::{Asset, AssetInfoBase};

pub fn new_cw3_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut cw3_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "multisig_addr" => cw3_addr = attrb.value,
                            _ => (),
                        }
                    }
                }
            }

            let mut endowment = ENDOWMENT.load(deps.storage)?;
            endowment.owner = deps.api.addr_validate(&cw3_addr)?;
            ENDOWMENT.save(deps.storage, &endowment)?;

            Ok(Response::default())
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

pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the registrar contract can update it's address in the config
    if info.sender != config.registrar_contract {
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

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address strings passed
    endowment.kyc_donors_only = msg.kyc_donors_only;
    endowment.owner = deps.api.addr_validate(&msg.owner)?;
    ENDOWMENT.save(deps.storage, &endowment)?;

    let profile = PROFILE.load(deps.storage)?;

    // send the new owner informtion back to the registrar
    Ok(
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarExecuter::UpdateEndowmentEntry(
                UpdateEndowmentEntryMsg {
                    endowment_addr: env.contract.address.to_string(),
                    owner: Some(msg.owner),
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
    if sender_addr.to_string() != index_fund {
        let new_splits = check_splits(registrar_split_configs, locked_split, liquid_split);
        locked_split = new_splits.0;
        liquid_split = new_splits.1;
    }

    let locked_amount = Asset {
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
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
    };
    state.balances.liquid_balance.add_tokens(liquid_balance);

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
    Ok(Response::new()
        .add_submessages(deposit_messages)
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
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
