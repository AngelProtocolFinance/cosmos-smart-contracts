use crate::state::{CONFIG, ENDOWMENTSETTINGS};
use angel_core::errors::core::ContractError;
use angel_core::messages::cw3_multisig::EndowmentInstantiateMsg as Cw3InstantiateMsg;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::messages::router::ExecuteMsg as SwapRouterExecuteMsg;
use angel_core::messages::settings_controller::*;
use angel_core::messages::subdao::InstantiateMsg as DaoInstantiateMsg;
use angel_core::responses::accounts::EndowmentDetailsResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, NetworkConnectionResponse, VaultDetailResponse,
    VaultListResponse,
};
use angel_core::structs::{
    AccountStrategies, AccountType, BalanceInfo, Beneficiary, DaoSetup, DonationMatch,
    DonationsReceived, EndowmentFee, EndowmentStatus, EndowmentType, GenericBalance, OneOffVaults,
    RebalanceDetails, SettingsController, SplitDetails, StrategyComponent, SwapOperation,
    VaultType, YieldVault,
};
use angel_core::utils::{
    check_splits, deposit_to_vaults, validate_deposit_fund, vault_endowment_balance,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, QueryRequest,
    ReplyOn, Response, StdError, SubMsg, SubMsgResult, Timestamp, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg};
use cw4::Member;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetUnchecked};
use cw_utils::{Duration, Expiration};

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
            let mut endowment = ENDOWMENTSETTINGS.load(deps.storage, id)?;
            endowment.dao = Some(dao);
            endowment.dao_token = Some(dao_token);
            ENDOWMENTSETTINGS.save(deps.storage, id, &endowment)?;

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
            let mut endowment = ENDOWMENTSETTINGS.load(deps.storage, id)?;
            endowment.donation_match_contract = Some(donation_match_contract);
            ENDOWMENTSETTINGS.save(deps.storage, id, &endowment)?;

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

pub fn update_endowment_settings(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentSettingsMsg,
) -> Result<Response, ContractError> {
    let mut endowment = ENDOWMENTSETTINGS.load(deps.storage, msg.id)?;
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &angel_core::messages::registrar::QueryMsg::Config {},
    )?;
    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        registrar_config.accounts_contract.unwrap(),
        &angel_core::messages::accounts::QueryMsg::Endowment { id: msg.id },
    )?;
    let endow_state: angel_core::responses::accounts::StateResponse =
        deps.querier.query_wasm_smart(
            registrar_config.accounts_contract.unwrap(),
            &angel_core::messages::accounts::QueryMsg::State { id: msg.id },
        )?;

    if endow_state.closing_endowment {
        return Err(ContractError::UpdatesAfterClosed {});
    }

    if !(info.sender == config.owner || info.sender == endow_detail.owner) {
        if endowment.dao.is_none() || info.sender != *endowment.dao.as_ref().unwrap() {
            return Err(ContractError::Unauthorized {});
        }
    }

    // only normalized endowments can update certain settings (ie. Charity Endowments have more fixed settings)
    if endow_detail.endow_type != EndowmentType::Charity {
        if let Some(whitelisted_beneficiaries) = msg.whitelisted_beneficiaries {
            let endow_mature_time = endow_detail
                .maturity_time
                .expect("Cannot get maturity time");
            if env.block.time.seconds() < endow_mature_time {
                if endowment
                    .settings_controller
                    .whitelisted_beneficiaries
                    .can_change(
                        &info.sender,
                        &endow_detail.owner,
                        endowment.dao.as_ref(),
                        env.block.time,
                    )
                {
                    endowment.whitelisted_beneficiaries = whitelisted_beneficiaries;
                }
            }
        }
        if let Some(whitelisted_contributors) = msg.whitelisted_contributors {
            let endow_mature_time = endow_detail
                .maturity_time
                .expect("Cannot get maturity time");
            if env.block.time.seconds() < endow_mature_time {
                if endowment
                    .settings_controller
                    .whitelisted_contributors
                    .can_change(
                        &info.sender,
                        &endow_detail.owner,
                        endowment.dao.as_ref(),
                        env.block.time,
                    )
                {
                    endowment.whitelisted_contributors = whitelisted_contributors;
                }
            }
        }
    }

    if let Some(whitelist) = msg.maturity_whitelist {
        let endow_mature_time = endow_detail
            .maturity_time
            .expect("Cannot get maturity time");
        if env.block.time.seconds() < endow_mature_time {
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

    endowment.settings_controller = match msg.settings_controller.clone() {
        Some(controller) => {
            if endowment.settings_controller.image.can_change(
                &info.sender,
                &endow_detail.owner,
                endowment.dao.as_ref(),
                env.block.time,
            ) {
                controller
            } else {
                endowment.settings_controller
            }
        }
        None => endowment.settings_controller,
    };
    ENDOWMENTSETTINGS.save(deps.storage, msg.id, &endowment)?;

    Ok(Response::new().add_attribute("action", "update_endowment_settings"))
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
    let mut endowment = ENDOWMENTSETTINGS.load(deps.storage, id)?;

    // grab a setting's permissions from SettingsController
    let mut permissions = endowment
        .settings_controller
        .get_permissions(setting.clone())?;

    // update the delegate field appropraitely based on action
    match action.as_str() {
        "set" => {
            permissions.set_delegate(
                &info.sender,
                &endowment.owner,
                endowment.dao.as_ref(),
                deps.api.addr_validate(&delegate_address)?,
                delegate_expiry,
            );
        }
        "revoke" => {
            permissions.revoke_delegate(
                &info.sender,
                &endowment.owner,
                endowment.dao.as_ref(),
                env.block.time,
            );
        }
        _ => unimplemented!(),
    }

    // save mutated permissions back to SettingsController
    endowment
        .settings_controller
        .set_permissions(setting, permissions)?;
    ENDOWMENTSETTINGS.save(deps.storage, id, &endowment)?;

    Ok(Response::default().add_attribute("action", "update_delegate"))
}

pub fn update_endowment_fees(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentFeesMsg,
) -> Result<Response, ContractError> {
    let mut endowment = ENDOWMENTSETTINGS.load(deps.storage, msg.id)?;

    // only normalized endowments can update the additional fees
    if endowment.endow_type != EndowmentType::Charity {
        return Err(ContractError::Std(StdError::generic_err(
            "Charity Endowments may not change endowment fees",
        )));
    }

    // Update the "EndowmentFee"s
    if endowment.settings_controller.earnings_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
        env.block.time,
    ) {
        endowment.earnings_fee = msg.earnings_fee;
    }

    if endowment.settings_controller.deposit_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
        env.block.time,
    ) {
        endowment.deposit_fee = msg.deposit_fee;
    }

    if endowment.settings_controller.withdraw_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
        env.block.time,
    ) {
        endowment.withdraw_fee = msg.withdraw_fee;
    }

    if endowment.settings_controller.aum_fee.can_change(
        &info.sender,
        &endowment.owner,
        endowment.dao.as_ref(),
        env.block.time,
    ) {
        endowment.aum_fee = msg.aum_fee;
    }

    ENDOWMENTSETTINGS.save(deps.storage, msg.id, &endowment)?;

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
    let endowment = ENDOWMENTSETTINGS.load(deps.storage, endowment_id)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.dao.is_some() {
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
                endow_type: endowment.endow_type,
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
    endowment_id: u32,
    setup: DonationMatch,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENTSETTINGS.load(deps.storage, endowment_id)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if endowment.dao.is_some() {
        return Err(ContractError::Std(StdError::generic_err(
            "A DAO does not exist yet for this Endowment. Please set that up first.",
        )));
    }

    if endowment.donation_match_contract.is_some() {
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
                        id: 2,
                        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                            code_id: match_code,
                            admin: None,
                            label: "new donation match contract".to_string(),
                            msg: to_binary(
                                &angel_core::messages::donation_match::InstantiateMsg {
                                    id: endowment_id,
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
                id: 2,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: match_code,
                    admin: None,
                    label: "new donation match contract".to_string(),
                    msg: to_binary(&angel_core::messages::donation_match::InstantiateMsg {
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
