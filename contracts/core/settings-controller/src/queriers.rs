use crate::state::{CONFIG, ENDOWMENTSETTINGS};
use angel_core::responses::settings_controller::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
    })
}

pub fn query_endowment_settings(deps: Deps, id: u32) -> StdResult<EndowmentSettingsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENTSETTINGS.load(deps.storage, id)?;
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
