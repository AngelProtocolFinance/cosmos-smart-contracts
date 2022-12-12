use crate::state::{CONFIG, ENDOWMENTSETTINGS};
use angel_core::{
    responses::settings_controller::*,
    structs::{EndowmentSettings, SettingsController},
};
use cosmwasm_std::{Addr, Deps, Env, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
    })
}

pub fn query_endowment_settings(deps: Deps, id: u32) -> StdResult<EndowmentSettingsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENTSETTINGS
        .load(deps.storage, id)
        .unwrap_or(EndowmentSettings::default());

    Ok(EndowmentSettingsResponse {
        dao: endowment.dao,
        dao_token: endowment.dao_token,
        donation_match_contract: endowment.donation_match_contract,
        donation_match_active: endowment.donation_match_active,
        whitelisted_beneficiaries: endowment.whitelisted_beneficiaries,
        whitelisted_contributors: endowment.whitelisted_contributors,
        maturity_whitelist: endowment.maturity_whitelist,
        earnings_fee: endowment.earnings_fee,
        withdraw_fee: endowment.withdraw_fee,
        deposit_fee: endowment.deposit_fee,
        aum_fee: endowment.aum_fee,
        settings_controller: endowment.settings_controller,
        parent: endowment.parent,
        split_to_liquid: endowment.split_to_liquid,
        ignore_user_splits: endowment.ignore_user_splits,
    })
}

pub fn query_endowment_permissions(
    deps: Deps,
    env: Env,
    id: u32,
    setting_updater: Addr,
    endowment_owner: Addr,
) -> StdResult<EndowmentPermissionsResponse> {
    let endow_settings = ENDOWMENTSETTINGS
        .load(deps.storage, id)
        .unwrap_or(EndowmentSettings::default());
    let dao_ref = endow_settings.dao.as_ref();
    let SettingsController {
        settings_controller,
        strategies,
        whitelisted_beneficiaries,
        whitelisted_contributors,
        maturity_time,
        profile,
        earnings_fee,
        withdraw_fee,
        deposit_fee,
        aum_fee,
        kyc_donors_only,
        name,
        image,
        logo,
        categories,
    } = endow_settings.settings_controller;

    Ok(EndowmentPermissionsResponse {
        settings_controller: settings_controller.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        strategies: strategies.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        whitelisted_beneficiaries: whitelisted_beneficiaries.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        whitelisted_contributors: whitelisted_contributors.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        maturity_time: maturity_time.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        profile: profile.can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        earnings_fee: earnings_fee.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        withdraw_fee: withdraw_fee.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        deposit_fee: deposit_fee.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        aum_fee: aum_fee.can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        kyc_donors_only: kyc_donors_only.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
        name: name.can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        image: image.can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        logo: logo.can_change(&setting_updater, &endowment_owner, dao_ref, env.block.time),
        categories: categories.can_change(
            &setting_updater,
            &endowment_owner,
            dao_ref,
            env.block.time,
        ),
    })
}
