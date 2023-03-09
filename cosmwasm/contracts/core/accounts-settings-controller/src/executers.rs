use crate::state::{CONFIG, CONTROLLER, SETTINGS};
use angel_core::errors::core::ContractError;
use angel_core::msgs::accounts::EndowmentDetailsResponse;
use angel_core::msgs::accounts_settings_controller::*;
use angel_core::msgs::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::msgs::registrar::QueryMsg as RegistrarQuerier;
use angel_core::msgs::subdao::InstantiateMsg as DaoInstantiateMsg;
use angel_core::structs::{
    DaoSetup, DonationMatch, EndowmentController, EndowmentSettings, EndowmentType,
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError, SubMsg,
    SubMsgResult, WasmMsg,
};

pub fn dao_reply(deps: DepsMut, _env: Env, msg: SubMsgResult) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut id: u32 = 0;
            let mut dao: Addr = Addr::unchecked("");
            let mut dao_token: Addr = Addr::unchecked("");
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "endow_id" => id = attrb.value.parse().unwrap(),
                            "dao_addr" => dao = deps.api.addr_validate(&attrb.value)?,
                            "dao_token_addr" => dao_token = deps.api.addr_validate(&attrb.value)?,
                            _ => (),
                        }
                    }
                }
            }
            if id == 0 || dao == Addr::unchecked("") || dao_token == Addr::unchecked("") {
                return Err(ContractError::AccountNotCreated {});
            }
            let mut endowment = SETTINGS.load(deps.storage, id)?;
            endowment.dao = Some(dao);
            endowment.dao_token = Some(dao_token);
            SETTINGS.save(deps.storage, id, &endowment)?;

            // set new CW3 as endowment owner to be picked up by the Registrar (EndowmentEntry)
            Ok(Response::default()
                .add_attribute("endow_dao", endowment.dao.unwrap())
                .add_attribute("endow_dao_token", endowment.dao_token.unwrap()))
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn donation_match_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut id: u32 = 0;
            let mut donation_match_contract: Addr = Addr::unchecked("");
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "donation_match_addr" => {
                                donation_match_contract = deps.api.addr_validate(&attrb.value)?
                            }
                            "endow_id" => id = attrb.value.parse().unwrap(),
                            _ => (),
                        }
                    }
                }
            }
            if id == 0 || donation_match_contract == Addr::unchecked("") {
                return Err(ContractError::AccountNotCreated {});
            }
            let mut endowment = SETTINGS.load(deps.storage, id)?;
            endowment.donation_match_contract = Some(donation_match_contract);
            SETTINGS.save(deps.storage, id, &endowment)?;

            // set new CW3 as endowment owner to be picked up by the Registrar (EndowmentEntry)
            Ok(Response::default().add_attribute(
                "endow_donation_match_contract",
                endowment.donation_match_contract.unwrap(),
            ))
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the accounts owner can update the config
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = msg.owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(registrar) = msg.registrar_contract {
        config.registrar_contract = deps.api.addr_validate(&registrar)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

pub fn create_endowment_settings(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: CreateEndowSettingsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps
        .querier
        .query_wasm_smart(config.registrar_contract, &RegistrarQuerier::Config {})?;

    // Only the "accounts_contract" can call this entry.
    if info.sender != registrar_config.accounts_contract.unwrap() {
        return Err(ContractError::Unauthorized {});
    }

    SETTINGS.update(deps.storage, msg.id, |existing| match existing {
        Some(_) => Err(ContractError::AlreadyInUse {}),
        None => Ok(EndowmentSettings {
            dao: None,
            dao_token: None,
            donation_match_active: msg.donation_match_active,
            donation_match_contract: msg.donation_match_contract.clone(),
            beneficiaries_allowlist: msg.beneficiaries_allowlist.clone(),
            contributors_allowlist: msg.contributors_allowlist.clone(),
            maturity_allowlist: vec![],
            earnings_fee: msg.earnings_fee.clone(),
            withdraw_fee: msg.withdraw_fee.clone(),
            deposit_fee: msg.deposit_fee.clone(),
            aum_fee: msg.aum_fee.clone(),
            parent: msg.parent,
            split_to_liquid: msg.split_to_liquid.clone(),
            ignore_user_splits: msg.ignore_user_splits,
        }),
    })?;

    CONTROLLER.save(deps.storage, msg.id, &msg.endowment_controller.clone())?;

    Ok(Response::new().add_attribute("action", "create_endowment_settings"))
}

pub fn update_endowment_settings(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentSettingsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &angel_core::msgs::registrar::QueryMsg::Config {},
    )?;
    let accounts_contract = registrar_config.accounts_contract.unwrap();

    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        accounts_contract.to_string(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id: msg.id },
    )?;

    let mut settings = SETTINGS
        .load(deps.storage, msg.id)
        .unwrap_or(EndowmentSettings::default());
    let controller = CONTROLLER
        .load(deps.storage, msg.id)
        .unwrap_or(EndowmentController::default(&endow_detail.endow_type));

    let endow_state: angel_core::msgs::accounts::StateResponse = deps.querier.query_wasm_smart(
        accounts_contract,
        &angel_core::msgs::accounts::QueryMsg::State { id: msg.id },
    )?;
    if endow_state.closing_endowment {
        return Err(ContractError::UpdatesAfterClosed {});
    }

    // only normalized endowments can update certain settings (ie. Charity Endowments have more fixed settings)
    if endow_detail.endow_type != EndowmentType::Charity {
        if endow_detail.maturity_time == None
            || env.block.time.seconds() < endow_detail.maturity_time.unwrap()
        {
            if let Some(beneficiaries_allowlist) = msg.beneficiaries_allowlist {
                if controller.beneficiaries_allowlist.can_change(
                    &info.sender,
                    &endow_detail.owner,
                    settings.dao.as_ref(),
                    env.block.time,
                ) {
                    settings.beneficiaries_allowlist = beneficiaries_allowlist;
                } else {
                    return Err(ContractError::Unauthorized {});
                }
            }
            if let Some(contributors_allowlist) = msg.contributors_allowlist {
                if controller.contributors_allowlist.can_change(
                    &info.sender,
                    &endow_detail.owner,
                    settings.dao.as_ref(),
                    env.block.time,
                ) {
                    settings.contributors_allowlist = contributors_allowlist;
                } else {
                    return Err(ContractError::Unauthorized {});
                }
            }
            if let Some(maturity_allowlist) = msg.maturity_allowlist {
                if controller.maturity_allowlist.can_change(
                    &info.sender,
                    &endow_detail.owner,
                    settings.dao.as_ref(),
                    env.block.time,
                ) {
                    for addr in maturity_allowlist.add.iter() {
                        let validated_addr = deps.api.addr_validate(&addr)?;
                        settings.maturity_allowlist.push(validated_addr);
                    }
                    for addr in maturity_allowlist.remove.iter() {
                        let validated_addr = deps.api.addr_validate(&addr)?;
                        let pos = settings
                            .maturity_allowlist
                            .iter()
                            .position(|v| *v == validated_addr);
                        if pos != None {
                            settings.maturity_allowlist.swap_remove(pos.unwrap());
                        }
                    }
                } else {
                    return Err(ContractError::Unauthorized {});
                }
            }
        }
    }
    if let Some(split_to_liquid) = msg.split_to_liquid {
        if controller.split_to_liquid.can_change(
            &info.sender,
            &endow_detail.owner,
            settings.dao.as_ref(),
            env.block.time,
        ) {
            settings.split_to_liquid = Some(split_to_liquid);
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }
    if let Some(ignore_user_splits) = msg.ignore_user_splits {
        if controller.ignore_user_splits.can_change(
            &info.sender,
            &endow_detail.owner,
            settings.dao.as_ref(),
            env.block.time,
        ) {
            settings.ignore_user_splits = ignore_user_splits;
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    SETTINGS.save(deps.storage, msg.id, &settings)?;

    Ok(Response::new().add_attribute("action", "update_endowment_settings"))
}

pub fn update_endowment_controller(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentControllerMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &angel_core::msgs::registrar::QueryMsg::Config {},
    )?;
    let accounts_contract = registrar_config.accounts_contract.unwrap();

    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        accounts_contract.to_string(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id: msg.id },
    )?;

    let settings = SETTINGS
        .load(deps.storage, msg.id)
        .unwrap_or(EndowmentSettings::default());
    let mut controller = CONTROLLER
        .load(deps.storage, msg.id)
        .unwrap_or(EndowmentController::default(&endow_detail.endow_type));

    let endow_state: angel_core::msgs::accounts::StateResponse = deps.querier.query_wasm_smart(
        accounts_contract,
        &angel_core::msgs::accounts::QueryMsg::State { id: msg.id },
    )?;
    if endow_state.closing_endowment {
        return Err(ContractError::UpdatesAfterClosed {});
    }

    if !controller
        .get_permissions("endowment_controller".to_string())
        .unwrap()
        .can_change(
            &info.sender,
            &endow_detail.owner,
            settings.dao.as_ref(),
            env.block.time,
        )
    {
        return Err(ContractError::Unauthorized {});
    }

    // update the endowment controller permissions and any other passed fields
    if msg.endowment_controller != None {
        controller.set_permissions(
            "endowment_controller".to_string(),
            msg.endowment_controller.unwrap(),
        )?;
    }
    if msg.name != None {
        controller.set_permissions("name".to_string(), msg.name.unwrap())?;
    }
    if msg.image != None {
        controller.set_permissions("image".to_string(), msg.image.unwrap())?;
    }
    if msg.logo != None {
        controller.set_permissions("logo".to_string(), msg.logo.unwrap())?;
    }
    if msg.categories != None {
        controller.set_permissions("categories".to_string(), msg.categories.unwrap())?;
    }
    if msg.kyc_donors_only != None {
        controller.set_permissions("kyc_donors_only".to_string(), msg.kyc_donors_only.unwrap())?;
    }
    if msg.split_to_liquid != None {
        controller.set_permissions("split_to_liquid".to_string(), msg.split_to_liquid.unwrap())?;
    }
    if msg.ignore_user_splits != None {
        controller.set_permissions(
            "ignore_user_splits".to_string(),
            msg.ignore_user_splits.unwrap(),
        )?;
    }
    if msg.donation_match_active != None {
        controller.set_permissions(
            "donation_match_active".to_string(),
            msg.donation_match_active.unwrap(),
        )?;
    }
    if msg.beneficiaries_allowlist != None {
        controller.set_permissions(
            "beneficiaries_allowlist".to_string(),
            msg.beneficiaries_allowlist.unwrap(),
        )?;
    }
    if msg.contributors_allowlist != None {
        controller.set_permissions(
            "contributors_allowlist".to_string(),
            msg.contributors_allowlist.unwrap(),
        )?;
    }
    if msg.maturity_allowlist != None {
        controller.set_permissions(
            "maturity_allowlist".to_string(),
            msg.maturity_allowlist.unwrap(),
        )?;
    }
    if msg.earnings_fee != None {
        controller.set_permissions("earnings_fee".to_string(), msg.earnings_fee.unwrap())?;
    }
    if msg.deposit_fee != None {
        controller.set_permissions("deposit_fee".to_string(), msg.deposit_fee.unwrap())?;
    }
    if msg.withdraw_fee != None {
        controller.set_permissions("withdraw_fee".to_string(), msg.withdraw_fee.unwrap())?;
    }
    if msg.aum_fee != None {
        controller.set_permissions("aum_fee".to_string(), msg.aum_fee.unwrap())?;
    }

    CONTROLLER.save(deps.storage, msg.id, &controller)?;

    Ok(Response::new().add_attribute("action", "update_endowment_controller"))
}

pub fn update_delegate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    setting: String,
    action: String,
    delegate_address: String,
    delegate_expiry: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &angel_core::msgs::registrar::QueryMsg::Config {},
    )?;
    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        registrar_config.accounts_contract.unwrap(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id },
    )?;

    // grab the Settings & Controller for an Endowment
    let settings = SETTINGS
        .load(deps.storage, id)
        .unwrap_or(EndowmentSettings::default());
    let mut controller = CONTROLLER
        .load(deps.storage, id)
        .unwrap_or(EndowmentController::default(&endow_detail.endow_type));

    // grab the current permissions for the setting of interest
    let mut permissions = controller.get_permissions(setting.clone())?;

    // update the delegate field appropraitely based on action
    match action.as_str() {
        "set" => {
            permissions.set_delegate(
                &info.sender,
                &endow_detail.owner,
                settings.dao.as_ref(),
                deps.api.addr_validate(&delegate_address)?,
                delegate_expiry,
            );
        }
        "revoke" => {
            permissions.revoke_delegate(
                &info.sender,
                &endow_detail.owner,
                settings.dao.as_ref(),
                env.block.time,
            );
        }
        _ => return Err(ContractError::InvalidInputs {}),
    }

    // save mutated permissions back to Endowment Controller
    controller.set_permissions(setting, permissions)?;
    CONTROLLER.save(deps.storage, id, &controller)?;

    Ok(Response::default().add_attribute("action", "update_delegate"))
}

pub fn update_endowment_fees(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentFeesMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &angel_core::msgs::registrar::QueryMsg::Config {},
    )?;
    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        registrar_config.accounts_contract.unwrap(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id: msg.id },
    )?;

    let controller = CONTROLLER
        .load(deps.storage, msg.id)
        .unwrap_or(EndowmentController::default(&endow_detail.endow_type));
    let mut settings = SETTINGS
        .load(deps.storage, msg.id)
        .unwrap_or(EndowmentSettings::default());

    // only normalized endowments can update the additional fees
    if endow_detail.endow_type == EndowmentType::Charity {
        return Err(ContractError::Std(StdError::generic_err(
            "Charity Endowments may not change endowment fees",
        )));
    }

    // Update the "EndowmentFee"s
    if controller.earnings_fee.can_change(
        &info.sender,
        &endow_detail.owner,
        settings.dao.as_ref(),
        env.block.time,
    ) {
        settings.earnings_fee = msg.earnings_fee;
    }

    if controller.deposit_fee.can_change(
        &info.sender,
        &endow_detail.owner,
        settings.dao.as_ref(),
        env.block.time,
    ) {
        settings.deposit_fee = msg.deposit_fee;
    }

    if controller.withdraw_fee.can_change(
        &info.sender,
        &endow_detail.owner,
        settings.dao.as_ref(),
        env.block.time,
    ) {
        settings.withdraw_fee = msg.withdraw_fee;
    }

    if controller.aum_fee.can_change(
        &info.sender,
        &endow_detail.owner,
        settings.dao.as_ref(),
        env.block.time,
    ) {
        settings.aum_fee = msg.aum_fee;
    }

    SETTINGS.save(deps.storage, msg.id, &settings)?;

    Ok(Response::new()
        .add_attribute("action", "update_endowment_fees")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn setup_dao(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    endowment_id: u32,
    msg: DaoSetup,
) -> Result<Response, ContractError> {
    let settings = SETTINGS
        .load(deps.storage, endowment_id)
        .unwrap_or(EndowmentSettings::default());
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &angel_core::msgs::registrar::QueryMsg::Config {},
    )?;
    let accounts_contract = registrar_config
        .accounts_contract
        .expect("Cannot get the accounts contract address");
    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        accounts_contract.to_string(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id: endowment_id },
    )?;

    if !(info.sender == endow_detail.owner || info.sender == accounts_contract) {
        return Err(ContractError::Unauthorized {});
    }

    if settings.dao.is_some() {
        return Err(ContractError::Std(StdError::generic_err(
            "A DAO already exists for this Endowment",
        )));
    }

    Ok(Response::new().add_submessage(SubMsg {
        id: 1,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: registrar_config.subdao_gov_code.unwrap(),
            admin: None,
            label: "new endowment dao contract".to_string(),
            msg: to_binary(&DaoInstantiateMsg {
                id: endowment_id,
                quorum: msg.quorum,
                threshold: msg.threshold,
                voting_period: msg.voting_period,
                timelock_period: msg.timelock_period,
                expiration_period: msg.expiration_period,
                proposal_deposit: msg.proposal_deposit,
                snapshot_period: msg.snapshot_period,
                token: msg.token,
                endow_type: endow_detail.endow_type,
                endow_owner: endow_detail.owner.to_string(),
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
    endowment_id: u32,
    setup: DonationMatch,
) -> Result<Response, ContractError> {
    let settings = SETTINGS
        .load(deps.storage, endowment_id)
        .unwrap_or(EndowmentSettings::default());
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &angel_core::msgs::registrar::QueryMsg::Config {},
    )?;
    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        registrar_config.accounts_contract.unwrap(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id: endowment_id },
    )?;

    if info.sender != endow_detail.owner {
        return Err(ContractError::Unauthorized {});
    }

    if settings.dao.is_some() {
        return Err(ContractError::Std(StdError::generic_err(
            "A DAO does not exist yet for this Endowment. Please set that up first.",
        )));
    }

    if settings.donation_match_contract.is_some() {
        return Err(ContractError::Std(StdError::generic_err(
            "A Donation Match contract already exists for this Endowment",
        )));
    }

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
                        id: 2,
                        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                            code_id: match_code,
                            admin: None,
                            label: "new donation match contract".to_string(),
                            msg: to_binary(&angel_core::msgs::donation_match::InstantiateMsg {
                                id: endowment_id,
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
                id: 2,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: match_code,
                    admin: None,
                    label: "new donation match contract".to_string(),
                    msg: to_binary(&angel_core::msgs::donation_match::InstantiateMsg {
                        id: endowment_id,
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
