use crate::state::{CONFIG, ENDOWMENTS};
use angel_core::responses::settings_controller::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
    })
}

pub fn query_endowment_settings(deps: Deps, id: u32) -> StdResult<EndowmentDetailsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    Ok(EndowmentDetailsResponse {
        owner: endowment.owner,
        status: endowment.status,
        endow_type: endowment.endow_type,
        maturity_time: endowment.maturity_time,
        strategies: endowment.strategies,
        oneoff_vaults: endowment.oneoff_vaults,
        rebalance: endowment.rebalance,
        donation_match_contract: endowment
            .donation_match_contract
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "".to_string()),
        kyc_donors_only: endowment.kyc_donors_only,
        maturity_whitelist: endowment
            .maturity_whitelist
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>(),
        deposit_approved: endowment.deposit_approved,
        withdraw_approved: endowment.withdraw_approved,
        pending_redemptions: endowment.pending_redemptions,
        dao: None,
        dao_token: None,
        description: "test endowment desc".to_string(),
        copycat_strategy: endowment.copycat_strategy,
        proposal_link: endowment.proposal_link,
        name: endowment.name,
        tier: endowment.tier,
        categories: endowment.categories,
        logo: endowment.logo,
        image: endowment.image,
        parent: endowment.parent,
        settings_controller: endowment.settings_controller,
    })
}
