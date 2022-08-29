use crate::state::{Endowment, State, CONFIG, COPYCATS, ENDOWMENTS, STATES};
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
use angel_core::messages::router::ExecuteMsg as SwapRouterExecuteMsg;
use angel_core::responses::index_fund::FundListResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse, VaultListResponse,
};
use angel_core::structs::{
    AccountStrategies, AccountType, BalanceInfo, EndowmentType, GenericBalance, RebalanceDetails,
    SocialMedialUrls, SplitDetails, StrategyComponent, SwapOperation, Tier, YieldVault,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, redeem_account_vaults, redeem_all_vaults,
    validate_deposit_fund,
};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified};
use cw4::Member;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

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
    msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    let owner = deps.api.addr_validate(&msg.owner)?;
    // try to store the endowment, fail if the ID is already in use
    ENDOWMENTS.update(
        deps.storage,
        config.next_account_id,
        |existing| match existing {
            Some(_) => Err(ContractError::AlreadyInUse {}),
            None => Ok(Endowment {
                deposit_approved: false,
                withdraw_approved: false,
                owner,                                                  // Addr
                withdraw_before_maturity: msg.withdraw_before_maturity, // bool
                maturity_time: msg.maturity_time,                       // Option<u64>
                maturity_height: msg.maturity_height,                   // Option<u64>
                strategies: AccountStrategies::default(),
                rebalance: RebalanceDetails::default(),
                kyc_donors_only: msg.kyc_donors_only,
                profile: msg.profile.clone(),
                pending_redemptions: 0 as u8,
                auto_invest: false,
                copycat_strategy: None,
            }),
        },
    )?;

    STATES.save(
        deps.storage,
        config.next_account_id,
        &State {
            donations_received: Uint128::zero(),
            balances: BalanceInfo::default(),
            closing_endowment: false,
            closing_beneficiary: None,
        },
    )?;

    // initial default Response to add submessages to
    let mut res = Response::new().add_attributes(vec![
        attr("endow_id", config.next_account_id.to_string()),
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

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;
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
                max_voting_period: msg.cw3_max_voting_period,
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
    let mut endowment = ENDOWMENTS.load(deps.storage, msg.id)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    endowment.auto_invest = msg.auto_invest;
    endowment.kyc_donors_only = msg.kyc_donors_only;
    endowment.owner = deps.api.addr_validate(&msg.owner)?;
    ENDOWMENTS.save(deps.storage, msg.id, &endowment)?;

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

    let mut endowment = ENDOWMENTS.load(deps.storage, msg.id)?;
    endowment.deposit_approved = msg.deposit_approved;
    endowment.withdraw_approved = msg.withdraw_approved;
    ENDOWMENTS.save(deps.storage, msg.id, &endowment)?;

    Ok(Response::default())
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
            endowment_type: Some(endowment.profile.endow_type.clone()),
            acct_type: Some(acct_type.clone()),
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

    endowment
        .strategies
        .set_strategy(acct_type.clone(), new_strategies.clone());
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    // If this Endowment that is changing their strategy is also being "copycatted"
    // by other endowments, the new strategy needs to be updated on those endowments.
    let copiers = COPYCATS.load(deps.storage, id).unwrap_or(vec![]);
    for i in copiers.iter() {
        let mut e = ENDOWMENTS.load(deps.storage, *i).unwrap();
        e.strategies
            .set_strategy(acct_type.clone(), new_strategies.clone());
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
    if copied_endowment
        .strategies
        .get_strategy(acct_type)
        .is_empty()
    {
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
    if endowment.copycat_strategy != None {
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

pub fn rebalance_strategies(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;

    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.pending_redemptions != 0 {
        return Err(ContractError::RedemptionInProgress {});
    }

    // redeem all existing strategies from the Endowment's existing vault sources before
    // re-investing locked funds into new sources (triggered when last redemption returns)
    let redemption_msgs: Vec<SubMsg> = redeem_account_vaults(
        deps.as_ref(),
        id,
        config.registrar_contract.to_string(),
        &endowment.strategies.get_strategy(acct_type),
    )?;
    endowment.pending_redemptions = redemption_msgs.len() as u8;
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    Ok(Response::new().add_submessages(redemption_msgs))
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
                .liquid_balance
                .get_denom_amount(denom.to_string())
                .amount
                < amount
            {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .liquid_balance
                .deduct_tokens(Balance::from(vec![Coin {
                    amount,
                    denom: denom.to_string(),
                }]));
        }
        (AssetInfo::Native(denom), AccountType::Locked) => {
            if state
                .balances
                .locked_balance
                .get_denom_amount(denom.to_string())
                .amount
                < amount
            {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .locked_balance
                .deduct_tokens(Balance::from(vec![Coin {
                    amount,
                    denom: denom.to_string(),
                }]));
        }
        (AssetInfo::Cw20(addr), AccountType::Liquid) => {
            if state
                .balances
                .liquid_balance
                .get_token_amount(addr.clone())
                .amount
                < amount
            {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .liquid_balance
                .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr.clone(),
                    amount,
                }));
        }
        (AssetInfo::Cw20(addr), AccountType::Locked) => {
            if state
                .balances
                .locked_balance
                .get_token_amount(addr.clone())
                .amount
                < amount
            {
                return Err(ContractError::BalanceTooSmall {});
            }
            state
                .balances
                .locked_balance
                .deduct_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr.clone(),
                    amount,
                }));
        }
        (AssetInfo::Cw1155(_, _), _) => unimplemented!(),
    }

    let swap_msg: CosmosMsg;
    match offer_asset {
        AssetInfo::Native(denom) => {
            swap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: registrar_config.swaps_router.unwrap().to_string(),
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
            });
        }
        AssetInfo::Cw20(addr) => {
            swap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.clone().to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                    contract: registrar_config.swaps_router.unwrap().to_string(),
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
            });
        }
        AssetInfo::Cw1155(_, _) => unimplemented!(),
    }
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
        (AssetInfo::Native(denom), AccountType::Liquid) => state
            .balances
            .liquid_balance
            .add_tokens(Balance::from(vec![Coin {
                amount: final_asset.amount,
                denom: denom.to_string(),
            }])),
        (AssetInfo::Native(denom), AccountType::Locked) => state
            .balances
            .locked_balance
            .add_tokens(Balance::from(vec![Coin {
                amount: final_asset.amount,
                denom: denom.to_string(),
            }])),
        (AssetInfo::Cw20(addr), AccountType::Liquid) => {
            state
                .balances
                .liquid_balance
                .add_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr.clone(),
                    amount: final_asset.amount,
                }))
        }
        (AssetInfo::Cw20(addr), AccountType::Locked) => {
            state
                .balances
                .locked_balance
                .add_tokens(Balance::Cw20(Cw20CoinVerified {
                    address: addr.clone(),
                    amount: final_asset.amount,
                }))
        }
        (AssetInfo::Cw1155(_, _), _) => unimplemented!(),
    }
    STATES.save(deps.storage, id, &state)?;
    Ok(Response::new())
}

pub fn vault_receipt(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    sender_addr: Addr,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATES.load(deps.storage, id)?;
    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;

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

    let mut deposit_package: (Vec<SubMsg>, Uint128) = (vec![], Uint128::zero());
    match endowment.pending_redemptions {
        0 => {
            // add returned tokens back to that endowment's locked balance
            let returned_bal = match returned_token.info {
                AssetInfoBase::Native(ref denom) => Balance::from(vec![Coin {
                    denom: denom.to_string(),
                    amount: returned_token.amount,
                }]),
                AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                    address: contract_addr.clone(),
                    amount: returned_token.amount,
                }),
                AssetInfoBase::Cw1155(_, _) => unimplemented!(),
            };
            state.balances.locked_balance.add_tokens(returned_bal);
        }
        // last redemption, remove pending u64, and build deposit submsgs
        1 => {
            endowment.pending_redemptions = 0;
            // normal vault receipt if closing_endowment has NOT been set to TRUE
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
                let leftovers: Asset;
                deposit_package = deposit_to_vaults(
                    deps.as_ref(),
                    config.registrar_contract.to_string(),
                    id.clone(),
                    asset.clone(),
                    &endowment.strategies.get_strategy(acct_type),
                )?;
                leftovers = Asset {
                    info: asset.info,
                    amount: deposit_package.1,
                };

                // set any remaining tokens to the locked balance "Tokens on Hand"
                state
                    .balances
                    .locked_balance
                    .set_token_balances(match leftovers.info {
                        AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
                            denom: denom.to_string(),
                            amount: leftovers.amount,
                        }]),
                        AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                            address: contract_addr.clone(),
                            amount: leftovers.amount,
                        }),
                        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
                    });
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
                        Balance::Native(v) => deposit_package.0.push(SubMsg::new(BankMsg::Send {
                            to_address: deps.api.addr_validate(addr)?.to_string(),
                            amount: v.0,
                        })),
                        Balance::Cw20(v) => {
                            deposit_package.0.push(SubMsg::new(CosmosMsg::Wasm(
                                WasmMsg::Execute {
                                    contract_addr: v.address.to_string(),
                                    msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                                        recipient: addr.to_string(),
                                        amount: v.amount,
                                    })
                                    .unwrap(),
                                    funds: vec![],
                                },
                            )));
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
                                    endowment_id: id.clone(),
                                })?,
                            }))?;
                        if !fund_list.funds.is_empty() {
                            // send funds to the first index fund in list
                            match asset {
                                Balance::Native(v) => deposit_package.0.push(SubMsg::new(
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
                                Balance::Cw20(v) => deposit_package.0.push(SubMsg::new(
                                    CosmosMsg::Wasm(WasmMsg::Execute {
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
                                    }),
                                )),
                            }
                        } else {
                            // Orphaned Endowment (ie. no parent index fund)
                            // send funds to the DANO treasury
                            match asset {
                                Balance::Native(v) => {
                                    deposit_package.0.push(SubMsg::new(BankMsg::Send {
                                        to_address: registrar_config.treasury,
                                        amount: v.0,
                                    }))
                                }
                                Balance::Cw20(v) => deposit_package.0.push(SubMsg::new(
                                    CosmosMsg::Wasm(WasmMsg::Execute {
                                        contract_addr: v.address.to_string(),
                                        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                                            recipient: registrar_config.treasury,
                                            amount: v.amount,
                                        })
                                        .unwrap(),
                                        funds: vec![],
                                    }),
                                )),
                            }
                        }
                    }
                }
            }
        }
        _ => {
            // add returned tokens back to that endowment's locked balance
            let returned_bal = match returned_token.info {
                AssetInfoBase::Native(ref denom) => Balance::from(vec![Coin {
                    denom: denom.to_string(),
                    amount: returned_token.amount,
                }]),
                AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                    address: contract_addr.clone(),
                    amount: returned_token.amount,
                }),
                AssetInfoBase::Cw1155(_, _) => unimplemented!(),
            };
            state.balances.locked_balance.add_tokens(returned_bal);
            // subtract one redemption and hold off on doing deposits until down to 1
            endowment.pending_redemptions -= 1;
        }
    };

    STATES.save(deps.storage, id, &state)?;
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    Ok(Response::new()
        .add_submessages(deposit_package.0)
        .add_attribute("action", "vault_receipt"))
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
    let msg: SubMsg = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: vault_addr.clone(),
        msg: to_binary(&angel_core::messages::vault::ExecuteMsg::ReinvestToLocked {
            endowment_id: id.clone(),
            amount,
        })
        .unwrap(),
        funds: vec![],
    }));
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
    let mut state = STATES.load(deps.storage, msg.id)?;
    state.donations_received += deposit_amount;

    let mut deposit_messages: Vec<SubMsg> = vec![];
    // check endowment strategies are setup
    // hold locked funds until (auto_invest == true && strategy has vaults set)
    if endowment.auto_invest == false {
        // increase the locked & liquid balance by respective donation amounts
        match locked_amount.info {
            AssetInfoBase::Native(ref denom) => {
                state
                    .balances
                    .locked_balance
                    .add_tokens(Balance::from(vec![Coin {
                        denom: denom.to_string(),
                        amount: locked_amount.amount,
                    }]));
                state
                    .balances
                    .liquid_balance
                    .add_tokens(Balance::from(vec![Coin {
                        denom: denom.to_string(),
                        amount: liquid_amount.amount,
                    }]));
            }
            AssetInfoBase::Cw20(ref contract_addr) => {
                state
                    .balances
                    .locked_balance
                    .add_tokens(Balance::Cw20(Cw20CoinVerified {
                        address: contract_addr.clone(),
                        amount: locked_amount.amount,
                    }));

                state
                    .balances
                    .liquid_balance
                    .add_tokens(Balance::Cw20(Cw20CoinVerified {
                        address: contract_addr.clone(),
                        amount: liquid_amount.amount,
                    }));
            }
            AssetInfoBase::Cw1155(_, _) => unimplemented!(),
        };
    } else {
        // Process Locked Strategy Deposits
        let locked_strategies = endowment.strategies.get_strategy(AccountType::Locked);
        // build deposit messages for each of the sources/amounts
        let (messages, leftover_amt) = deposit_to_vaults(
            deps.as_ref(),
            config.registrar_contract.to_string(),
            msg.id.clone(),
            locked_amount.clone(),
            &locked_strategies,
        )?;
        for m in messages.iter() {
            deposit_messages.push(m.clone());
        }
        // If invested portion of strategies < 100% there will be leftover deposits
        // Add any remaining deposited tokens to the locked balance "Tokens on Hand"
        state
            .balances
            .locked_balance
            .add_tokens(match locked_amount.info {
                AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
                    denom: denom.to_string(),
                    amount: leftover_amt,
                }]),
                AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                    address: contract_addr.clone(),
                    amount: leftover_amt,
                }),
                AssetInfoBase::Cw1155(_, _) => unimplemented!(),
            });
    }

    // Process Liquid Strategy Deposits
    let liquid_strategies = endowment.strategies.get_strategy(AccountType::Liquid);
    // build deposit messages for each of the sources/amounts
    let (messages, leftover_amt) = deposit_to_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        msg.id.clone(),
        liquid_amount.clone(),
        &liquid_strategies,
    )?;
    for m in messages.iter() {
        deposit_messages.push(m.clone());
    }
    // If invested portion of strategies < 100% there will be leftover deposits
    // Add any remaining deposited tokens to the liquid balance "Tokens on Hand"
    state
        .balances
        .liquid_balance
        .add_tokens(match liquid_amount.info {
            AssetInfoBase::Native(denom) => Balance::from(vec![Coin {
                denom: denom.to_string(),
                amount: leftover_amt,
            }]),
            AssetInfoBase::Cw20(contract_addr) => Balance::Cw20(Cw20CoinVerified {
                address: contract_addr.clone(),
                amount: leftover_amt,
            }),
            AssetInfoBase::Cw1155(_, _) => unimplemented!(),
        });

    STATES.save(deps.storage, msg.id, &state)?;
    Ok(Response::new()
        .add_submessages(deposit_messages)
        .add_attribute("action", "account_deposit"))
}

/// Allow Endowment owners to invest some amount of their free balance
/// "Tokens on Hand" holdings into a Vault. Does not have to be a Vault
/// that exists in their Strategy. One-time/one-off investment.
pub fn vault_invest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    asset: AssetInfo,
    amount: Uint128,
    vault: String,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // check vault addr passed is valid
    let vault_addr = deps.api.addr_validate(&vault)?.to_string();

    // check vault is in registrar vaults list and is approved
    let config = CONFIG.load(deps.storage)?;
    let vault_config: VaultDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Vault {
                vault_addr: vault_addr.clone(),
            })?,
        }))?;

    let yield_vault: YieldVault = vault_config.vault;
    if !yield_vault.approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Vault is not approved to accept deposits".to_string(),
        }));
    }

    if yield_vault.acct_type != acct_type {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!("Vault's account type is not: {}", acct_type),
        }));
    }

    // create a deposit message for the vault
    // funds payload can contain CW20 | Native token amounts
    let deposit_msg = match asset {
        AssetInfoBase::Native(ref denom) => SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: vault_addr.clone(),
            msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Deposit {
                endowment_id: id.clone(),
            })
            .unwrap(),
            funds: vec![Coin {
                denom: denom.clone(),
                amount,
            }],
        })),
        AssetInfoBase::Cw20(ref contract_addr) => SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: vault_addr,
                amount,
                msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Deposit {
                    endowment_id: id,
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        })),
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
    };

    Ok(Response::new()
        .add_attribute("action", "vault_invest")
        .add_submessage(deposit_msg))
}

/// Allow Endowment owners to redeem some amount of Vault tokens back to their
/// Locked Balance "Tokens on Hand" holdings
pub fn vault_redeem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    amount: Uint128,
    vault: String,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    if endowment.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.pending_redemptions != 0 {
        return Err(ContractError::RedemptionInProgress {});
    }
    // check vault addr passed is valid
    let vault_addr = deps.api.addr_validate(&vault)?.to_string();

    // check vault is in registrar vaults list and is approved
    let config = CONFIG.load(deps.storage)?;
    let _vault_config: VaultDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Vault {
                vault_addr: vault_addr.clone(),
            })?,
        }))?;

    let redeem_msg = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: vault_addr,
        msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Redeem {
            endowment_id: id,
            amount,
        })
        .unwrap(),
        funds: vec![],
    }));

    ENDOWMENTS.save(deps.storage, id, &endowment)?;

    Ok(Response::new()
        .add_attribute("action", "redeem_vault")
        .add_submessage(redeem_msg))
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    id: u32,
    acct_type: AccountType,
    beneficiary: String,
    assets: GenericBalance,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    let config = CONFIG.load(deps.storage)?;

    // check that sender is correct based on account types attempting to access
    if acct_type == AccountType::Locked && info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if acct_type == AccountType::Liquid {
        if info.sender != endowment.owner {
            return Err(ContractError::Unauthorized {});
        }
        // check that the Endowment has been approved to withdraw deposits
        if !endowment.withdraw_approved {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Withdraws are not approved for this endowment".to_string(),
            }));
        }
    }

    let mut state = STATES.load(deps.storage, id)?;
    let mut state_bal: GenericBalance = state.balances.get(&acct_type).clone();
    let mut messages: Vec<SubMsg> = vec![];

    for asset in assets.native.iter() {
        let liquid_balance = state_bal.get_denom_amount(asset.denom.clone()).amount;
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
    state_bal.deduct_tokens(Balance::from(assets.native));

    for asset in assets.cw20.into_iter() {
        let liquid_balance = state_bal.get_token_amount(asset.address.clone()).amount;
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
        state_bal.deduct_tokens(Balance::Cw20(asset));
    }

    // set the updated balance for the account type
    match acct_type {
        AccountType::Locked => state.balances.locked_balance = state_bal,
        AccountType::Liquid => state.balances.liquid_balance = state_bal,
    }
    STATES.save(deps.storage, id, &state)?;

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "withdraw_liquid")
        .add_attribute("beneficiary", beneficiary))
}

pub fn close_endowment(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
    beneficiary: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    let mut endowment = ENDOWMENTS.load(deps.storage, id)?;
    if endowment.pending_redemptions != 0 {
        return Err(ContractError::RedemptionInProgress {});
    }

    // set the STATE with relevent status and closing beneficiary
    let mut state = STATES.load(deps.storage, id)?;
    state.closing_endowment = true;
    state.closing_beneficiary = beneficiary;
    STATES.save(deps.storage, id, &state)?;

    // Redeem all UST back from strategies invested in
    let redeem_messages = redeem_all_vaults(
        deps.as_ref(),
        id,
        config.registrar_contract.to_string(),
        &endowment.strategies,
    )?;

    endowment.pending_redemptions = redeem_messages.len() as u8;
    ENDOWMENTS.save(deps.storage, id, &endowment)?;

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
    let mut endowment = ENDOWMENTS.load(deps.storage, msg.id)?;
    let config = CONFIG.load(deps.storage)?;

    if !(info.sender == endowment.owner || info.sender == config.owner) {
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

    ENDOWMENTS.save(deps.storage, msg.id, &endowment)?;

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
