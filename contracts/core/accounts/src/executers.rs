use crate::state::{Endowment, State, CONFIG, ENDOWMENTS, REDEMPTIONS, STATES};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw3_multisig::EndowmentInstantiateMsg as Cw3InstantiateMsg;
use angel_core::messages::index_fund::{
    DepositMsg as IndexFundDepositMsg, ExecuteMsg as IndexFundExecuter,
    QueryMsg as IndexFundQuerier,
};
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::messages::registrar::{
    ExecuteMsg as RegistrarExecuter, QueryMsg as RegistrarQuerier, UpdateEndowmentEntryMsg,
};
use angel_core::responses::index_fund::FundListResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse, VaultListResponse,
};
use angel_core::structs::{
    BalanceInfo, EndowmentType, FundingSource, GenericBalance, RebalanceDetails, SocialMedialUrls,
    SplitDetails, StrategyComponent, Tier,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, redeem_from_vaults, validate_deposit_fund,
    withdraw_from_vaults,
};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified};
use cw4::Member;
use cw_asset::{Asset, AssetInfoBase};

pub fn cw3_reply(deps: DepsMut, _env: Env, msg: SubMsgResult) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut id: String = "".to_string();
            let mut owner: Addr = Addr::unchecked("");
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "multisig_addr" => owner = deps.api.addr_validate(&attrb.value)?,
                            "endow_id" => id = attrb.value,
                            _ => (),
                        }
                    }
                }
            }
            if id == "".to_string() || owner == Addr::unchecked("") {
                return Err(ContractError::AccountNotCreated {});
            }
            let mut endowment = ENDOWMENTS.load(deps.storage, &id)?;
            endowment.owner = owner;
            ENDOWMENTS.save(deps.storage, &id, &endowment)?;

            // set new CW3 as endowment owner to be picked up by the Registrar (EndowmentEntry)
            Ok(Response::default().add_attribute("endow_owner", endowment.owner.to_string()))
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn create_endowment(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // check that the Endowment ID is of resonable length (lte 25 chars)
    if &msg.id.chars().count() >= &registrar_config.account_id_char_limit {
        return Err(ContractError::InvalidInputs {});
    }

    let owner = deps.api.addr_validate(&msg.owner)?;
    let beneficiary = deps.api.addr_validate(&msg.beneficiary)?;
    // try to store the endowment, fail if the ID is already in use
    ENDOWMENTS.update(deps.storage, &msg.id, |existing| match existing {
        Some(_) => Err(ContractError::AlreadyInUse {}),
        None => Ok(Endowment {
            deposit_approved: false,
            withdraw_approved: false,
            owner,                                                  // Addr
            beneficiary,                                            // Addr
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,                       // Option<u64>
            maturity_height: msg.maturity_height,                   // Option<u64>
            strategies: vec![],
            rebalance: RebalanceDetails::default(),
            kyc_donors_only: msg.kyc_donors_only,
            profile: msg.profile.clone(),
        }),
    })?;
    REDEMPTIONS.save(deps.storage, &msg.id, &None)?;
    STATES.save(
        deps.storage,
        &msg.id,
        &State {
            donations_received: Uint128::zero(),
            balances: BalanceInfo::default(),
            closing_endowment: false,
            closing_beneficiary: None,
        },
    )?;

    // initial default Response to add submessages to
    let mut res = Response::new().add_attributes(vec![
        attr("endow_id", msg.id.clone()),
        attr("endow_name", msg.profile.name),
        attr("endow_type", msg.profile.endow_type.to_string()),
        attr(
            "endow_logo",
            msg.profile.logo.unwrap_or_else(|| "".to_string()),
        ),
        attr(
            "endow_image",
            msg.profile.image.unwrap_or_else(|| "".to_string()),
        ),
        attr(
            "endow_tier",
            msg.profile.tier.unwrap_or_else(|| 0).to_string(),
        ),
        attr(
            "endow_un_sdg",
            msg.profile.un_sdg.unwrap_or_else(|| 0).to_string(),
        ),
    ]);

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
            label: "new endowment cw3 multisig".to_string(),
            msg: to_binary(&Cw3InstantiateMsg {
                // endowment ID
                id: msg.id,
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
                max_voting_period: msg.cw3_max_voting_period,
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    Ok(res)
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
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentSettingsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, &msg.id)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address strings passed
    endowment.kyc_donors_only = msg.kyc_donors_only;
    endowment.owner = deps.api.addr_validate(&msg.owner)?;
    ENDOWMENTS.save(deps.storage, &msg.id, &endowment)?;

    // send the new owner informtion back to the registrar
    Ok(
        Response::new().add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarExecuter::UpdateEndowmentEntry(
                UpdateEndowmentEntryMsg {
                    endowment_id: msg.id,
                    owner: Some(msg.owner),
                    name: Some(endowment.profile.name),
                    logo: endowment.profile.logo,
                    image: endowment.profile.image,
                    endow_type: Some(endowment.profile.endow_type),
                    tier: match endowment.profile.tier {
                        Some(1) => Some(Some(Tier::Level1)),
                        Some(2) => Some(Some(Tier::Level2)),
                        Some(3) => Some(Some(Tier::Level3)),
                        None => Some(None),
                        _ => return Err(ContractError::InvalidInputs {}),
                    },
                    un_sdg: match endowment.profile.un_sdg {
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

    let mut endowment = ENDOWMENTS.load(deps.storage, &msg.id)?;
    endowment.deposit_approved = msg.deposit_approved;
    endowment.withdraw_approved = msg.withdraw_approved;
    ENDOWMENTS.save(deps.storage, &msg.id, &endowment)?;

    Ok(Response::default())
}

pub fn update_strategies(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    strategies: Vec<Strategy>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, &id)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut redemptions = REDEMPTIONS.load(deps.storage, &id)?;
    if redemptions != None {
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
            endowment_type: Some(endowment.profile.endow_type.clone()),
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

    redemptions = Some(redeem_messages.len() as u64);
    REDEMPTIONS.save(deps.storage, &id, &redemptions)?;

    // update endowment strategies attribute with all newly passed strategies
    let mut new_strategies = vec![];
    for strategy in strategies {
        new_strategies.push(StrategyComponent {
            vault: deps.api.addr_validate(&strategy.vault.clone())?.to_string(),
            percentage: strategy.percentage,
        });
    }
    endowment.strategies = new_strategies;
    ENDOWMENTS.save(deps.storage, &id, &endowment)?;

    Ok(Response::new()
        .add_attribute("action", "update_strategies")
        .add_submessages(redeem_messages))
}

pub fn vault_receipt(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    sender_addr: Addr,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATES.load(deps.storage, &id)?;
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;
    let mut redemptions = REDEMPTIONS.load(deps.storage, &id)?;

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
    match redemptions {
        // last redemption, remove pending u64, and build deposit submsgs
        Some(1) => {
            redemptions = None;
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
                    id.clone(),
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
        Some(_) => match redemptions.unwrap().checked_sub(1) {
            Some(n) => redemptions = Some(n),
            None => redemptions = None,
        },
        None => (),
    };

    STATES.save(deps.storage, &id, &state)?;
    REDEMPTIONS.save(deps.storage, &id, &redemptions)?;

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
    let endowment = ENDOWMENTS.load(deps.storage, &msg.id)?;

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
    let mut state = STATES.load(deps.storage, &msg.id)?;
    state.donations_received += deposit_amount;

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
    // check endowment strategies set.
    // if empty: hold locked funds until a vault is set
    if endowment.strategies.is_empty() {
        deposit_messages = vec![];
        // increase the locked balance by locked donation amount
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
            msg.id.clone(),
            locked_amount,
            &endowment.strategies,
        )?;
    }

    STATES.save(deps.storage, &msg.id, &state)?;
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
    id: String,
    beneficiary: String,
    sources: Vec<FundingSource>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;

    // check that sender is the owner or the beneficiary
    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // check that the Endowment has been approved to withdraw deposits
    if !endowment.withdraw_approved {
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
        id.clone(),
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
    id: String,
    beneficiary: String,
    assets: GenericBalance,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;

    // check that sender is the owner or the beneficiary
    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    // check that the Endowment has been approved to withdraw deposits
    if !endowment.withdraw_approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Withdraws are not approved for this endowment".to_string(),
        }));
    }

    let mut state = STATES.load(deps.storage, &id)?;
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

    STATES.save(deps.storage, &id, &state)?;

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
    id: String,
    beneficiary: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    let mut redemptions = REDEMPTIONS.load(deps.storage, &id)?;
    if redemptions != None {
        return Err(ContractError::RedemptionInProgress {});
    }

    // set the STATE with relevent status and closing beneficiary
    let mut state = STATES.load(deps.storage, &id)?;
    state.closing_endowment = true;
    state.closing_beneficiary = beneficiary;
    STATES.save(deps.storage, &id, &state)?;

    // Redeem all UST back from strategies invested in
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;
    let redeem_messages = redeem_from_vaults(
        deps.as_ref(),
        env.contract.address,
        config.registrar_contract.to_string(),
        endowment.strategies,
    )?;

    redemptions = Some(redeem_messages.len() as u64);
    REDEMPTIONS.save(deps.storage, &id, &redemptions)?;

    Ok(Response::new()
        .add_attribute("action", "close_endowment")
        .add_attribute("sender", info.sender.to_string())
        .add_submessages(redeem_messages))
}

pub fn update_profile(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateProfileMsg,
) -> Result<Response, ContractError> {
    // Validation 1. Only "Endowment.owner" or "Config.owner" is able to execute
    let mut endowment = ENDOWMENTS.load(deps.storage, &msg.id)?;
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
    // Only config.owner can update "un_sdg" & "tier" fields
    if info.sender == config.owner {
        endowment.profile.un_sdg = msg.un_sdg;
        endowment.profile.tier = msg.tier;
        if let Some(endow_type) = msg.endow_type {
            endowment.profile.endow_type = match endow_type.as_str() {
                "charity" => EndowmentType::Charity,
                "normal" => EndowmentType::Normal,
                _ => return Err(ContractError::InvalidInputs {}),
            };
        }
    }

    // Only endowment.owner can update all other fields
    if info.sender == endowment.owner {
        if let Some(name) = msg.name.clone() {
            endowment.profile.name = name;
        }
        if let Some(overview) = msg.overview {
            endowment.profile.overview = overview;
        }
        endowment.profile.logo = msg.logo.clone();
        endowment.profile.image = msg.image.clone();
        endowment.profile.url = msg.url;
        endowment.profile.registration_number = msg.registration_number;
        endowment.profile.country_of_origin = msg.country_of_origin;
        endowment.profile.street_address = msg.street_address;
        endowment.profile.contact_email = msg.contact_email;
        endowment.profile.number_of_employees = msg.number_of_employees;
        endowment.profile.average_annual_budget = msg.average_annual_budget;
        endowment.profile.annual_revenue = msg.annual_revenue;
        endowment.profile.charity_navigator_rating = msg.charity_navigator_rating;

        let social_media_urls = SocialMedialUrls {
            facebook: msg.facebook,
            twitter: msg.twitter,
            linkedin: msg.linkedin,
        };
        endowment.profile.social_media_urls = social_media_urls;
    }

    ENDOWMENTS.save(deps.storage, &msg.id, &endowment)?;

    let sub_msgs: Vec<SubMsg> = vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.registrar_contract.to_string(),
        msg: to_binary(&RegistrarExecuter::UpdateEndowmentEntry(
            UpdateEndowmentEntryMsg {
                endowment_id: msg.id,
                name: msg.name,
                logo: msg.logo,
                image: msg.image,
                owner: Some(endowment.owner.to_string()),
                tier,
                un_sdg,
                endow_type: Some(endowment.profile.endow_type),
            },
        ))?,
        funds: vec![],
    }))];

    Ok(Response::new()
        .add_submessages(sub_msgs)
        .add_attribute("action", "update_profile")
        .add_attribute("sender", info.sender.to_string()))
}
