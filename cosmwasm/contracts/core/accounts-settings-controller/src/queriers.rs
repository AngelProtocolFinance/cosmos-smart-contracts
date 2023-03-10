use crate::state::{CONFIG, CONTROLLER, SETTINGS};
use angel_core::msgs::accounts::EndowmentDetailsResponse;
use angel_core::msgs::accounts_settings_controller::{
    ConfigResponse, EndowmentPermissionsResponse,
};
use angel_core::msgs::registrar::ConfigExtensionResponse as RegistrarConfigResponse;
use angel_core::structs::{EndowmentController, EndowmentSettings, EndowmentType};
use cosmwasm_std::{Addr, Deps, Env, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
    })
}

pub fn query_endowment_settings(deps: Deps, id: u32) -> StdResult<EndowmentSettings> {
    // this fails if no account is found
    let settings = SETTINGS.load(deps.storage, id)?;

    Ok(settings)
}

pub fn query_endowment_controller(deps: Deps, id: u32) -> StdResult<EndowmentController> {
    // this fails if no account is found
    let controller = CONTROLLER.load(deps.storage, id)?;

    Ok(controller)
}

pub fn query_endowment_permissions(
    deps: Deps,
    env: Env,
    id: u32,
    updater: Addr,
) -> StdResult<EndowmentPermissionsResponse> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &angel_core::msgs::registrar::QueryMsg::ConfigExtension {},
    )?;
    let accounts_contract = registrar_config.accounts_contract.unwrap();

    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        accounts_contract.to_string(),
        &angel_core::msgs::accounts::QueryMsg::Endowment { id },
    )?;

    let endow_settings = match endow_detail.endow_type {
        EndowmentType::Charity => SETTINGS
            .load(deps.storage, id)
            .unwrap_or(EndowmentSettings::default()),
        EndowmentType::Normal => SETTINGS
            .load(deps.storage, id)
            .unwrap_or(EndowmentSettings::default()),
    };

    let endow_controller = match endow_detail.endow_type {
        EndowmentType::Charity => CONTROLLER
            .load(deps.storage, id)
            .unwrap_or(EndowmentController::default(&endow_detail.endow_type)),
        EndowmentType::Normal => CONTROLLER
            .load(deps.storage, id)
            .unwrap_or(EndowmentController::default(&endow_detail.endow_type)),
    };

    let dao_ref = endow_settings.dao.as_ref();
    let owner = &endow_detail.owner;

    Ok(EndowmentPermissionsResponse {
        endowment_controller: endow_controller
            .get_permissions("endowment_controller".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        split_to_liquid: endow_controller
            .get_permissions("split_to_liquid".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        ignore_user_splits: endow_controller
            .get_permissions("ignore_user_splits".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        beneficiaries_allowlist: endow_controller
            .get_permissions("beneficiaries_allowlist".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        contributors_allowlist: endow_controller
            .get_permissions("contributors_allowlist".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        maturity_allowlist: endow_controller
            .get_permissions("maturity_allowlist".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        earnings_fee: endow_controller
            .get_permissions("earnings_fee".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        withdraw_fee: endow_controller
            .get_permissions("withdraw_fee".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        deposit_fee: endow_controller
            .get_permissions("deposit_fee".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        aum_fee: endow_controller
            .get_permissions("aum_fee".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        kyc_donors_only: endow_controller
            .get_permissions("kyc_donors_only".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        name: endow_controller
            .get_permissions("name".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        image: endow_controller
            .get_permissions("image".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        logo: endow_controller
            .get_permissions("logo".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
        categories: endow_controller
            .get_permissions("categories".to_string())
            .unwrap()
            .can_change(&updater, owner, dao_ref, env.block.time),
    })
}
