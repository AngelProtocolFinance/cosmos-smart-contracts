use crate::state::{CONFIG, CONTROLLER, SETTINGS};
use angel_core::responses::accounts_settings_controller::{
    ConfigResponse, EndowmentPermissionsResponse,
};
use angel_core::structs::{EndowmentController, EndowmentSettings};
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
    let settings = SETTINGS
        .load(deps.storage, id)
        .unwrap_or(EndowmentSettings::default());

    Ok(settings)
}

pub fn query_endowment_controller(deps: Deps, id: u32) -> StdResult<EndowmentController> {
    // this fails if no account is found
    let controller = CONTROLLER
        .load(deps.storage, id)
        .unwrap_or(EndowmentController::default());

    Ok(controller)
}

pub fn query_endowment_permissions(
    deps: Deps,
    env: Env,
    id: u32,
    setting_updater: Addr,
    endowment_owner: Addr,
) -> StdResult<EndowmentPermissionsResponse> {
    let endow_settings = SETTINGS
        .load(deps.storage, id)
        .unwrap_or(EndowmentSettings::default());
    let endow_controller = CONTROLLER
        .load(deps.storage, id)
        .unwrap_or(EndowmentController::default());
    let dao_ref = endow_settings.dao.as_ref();

    Ok(EndowmentPermissionsResponse {
        endowment_controller: endow_controller
            .get_permissions("endowment_controller".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        strategies: endow_controller
            .get_permissions("strategies".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        split_to_liquid: endow_controller
            .get_permissions("split_to_liquid".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        ignore_user_splits: endow_controller
            .get_permissions("ignore_user_splits".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        beneficiaries_allowlist: endow_controller
            .get_permissions("beneficiaries_allowlist".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        contributors_allowlist: endow_controller
            .get_permissions("contributors_allowlist".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        maturity_allowlist: endow_controller
            .get_permissions("maturity_allowlist".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        earnings_fee: endow_controller
            .get_permissions("earnings_fee".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        withdraw_fee: endow_controller
            .get_permissions("withdraw_fee".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        deposit_fee: endow_controller
            .get_permissions("deposit_fee".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        aum_fee: endow_controller
            .get_permissions("aum_fee".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        kyc_donors_only: endow_controller
            .get_permissions("kyc_donors_only".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        name: endow_controller
            .get_permissions("name".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        image: endow_controller
            .get_permissions("image".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        logo: endow_controller
            .get_permissions("logo".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        categories: endow_controller
            .get_permissions("categories".to_string())
            .unwrap()
            .can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
    })
}
