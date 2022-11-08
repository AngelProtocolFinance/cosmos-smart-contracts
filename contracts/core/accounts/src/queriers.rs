use crate::state::{
    read_endowments, Allowances, Endowment, ALLOWANCES, CONFIG, ENDOWMENTS, PROFILES, STATES,
};
use angel_core::responses::accounts::*;
use angel_core::structs::{AccountType, EndowmentBalanceResponse, EndowmentEntry, Tier};
use angel_core::utils::vault_endowment_balance;
use cosmwasm_std::{Deps, StdResult, Uint128};
use cw_asset::AssetInfo;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        settings_controller: config.settings_controller,
        next_account_id: config.next_account_id,
        max_general_category_id: config.max_general_category_id,
    })
}

pub fn query_state(deps: Deps, id: u32) -> StdResult<StateResponse> {
    let state = STATES.load(deps.storage, id)?;

    Ok(StateResponse {
        donations_received: state.donations_received,
        balances: state.balances,
        closing_endowment: state.closing_endowment,
        closing_beneficiary: state.closing_beneficiary,
    })
}

pub fn query_endowment_balance(deps: Deps, id: u32) -> StdResult<EndowmentBalanceResponse> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    let state = STATES.load(deps.storage, id)?;

    // setup the basic response object w/ account's balances locked & liquid (held by this contract)
    let tokens_on_hand = state.balances;

    // process all one-off vaults
    let mut invested_locked = vec![];
    for vault in endowment.oneoff_vaults.locked.into_iter() {
        let vault_bal = vault_endowment_balance(deps, vault.clone().to_string(), id);
        invested_locked.push((vault.to_string(), vault_bal));
    }
    let mut invested_liquid = vec![];
    for vault in endowment.oneoff_vaults.liquid.into_iter() {
        let vault_bal = vault_endowment_balance(deps, vault.clone().to_string(), id);
        invested_liquid.push((vault.to_string(), vault_bal));
    }

    Ok(EndowmentBalanceResponse {
        tokens_on_hand,
        invested_locked,
        invested_liquid,
    })
}

pub fn query_token_amount(
    deps: Deps,
    id: u32,
    asset_info: AssetInfo,
    acct_type: AccountType,
) -> StdResult<Uint128> {
    let _endowment = ENDOWMENTS.load(deps.storage, id)?;
    let state = STATES.load(deps.storage, id)?;
    let balance: Uint128 = match (asset_info, acct_type) {
        (AssetInfo::Native(denom), AccountType::Liquid) => {
            state.balances.liquid.get_denom_amount(denom).amount
        }
        (AssetInfo::Native(denom), AccountType::Locked) => {
            state.balances.locked.get_denom_amount(denom).amount
        }
        (AssetInfo::Cw20(addr), AccountType::Liquid) => {
            state.balances.liquid.get_token_amount(addr).amount
        }
        (AssetInfo::Cw20(addr), AccountType::Locked) => {
            state.balances.locked.get_token_amount(addr).amount
        }
        _ => unreachable!(),
    };
    Ok(balance)
}

pub fn query_endowment_list(
    deps: Deps,
    proposal_link: Option<u64>,
    start_after: Option<u32>,
    limit: Option<u64>,
) -> StdResult<EndowmentListResponse> {
    let endowments: Vec<(u32, Endowment)> =
        read_endowments(deps.storage, proposal_link, start_after, limit)?;
    let entries: Vec<EndowmentEntry> = endowments
        .iter()
        .map(|(i, e)| EndowmentEntry {
            id: *i,
            owner: e.owner.to_string(),
            status: e.status.clone(),
            endow_type: e.endow_type.clone(),
            name: Some(e.name.clone()),
            logo: e.logo.clone(),
            image: e.image.clone(),
            tier: match e.tier.unwrap_or(0) {
                1 => Some(Tier::Level1),
                2 => Some(Tier::Level2),
                3 => Some(Tier::Level3),
                _ => None,
            },
            categories: e.categories.clone(),
            proposal_link: e.proposal_link.clone(),
        })
        .collect();

    Ok(EndowmentListResponse {
        endowments: entries,
    })
}

pub fn query_endowment_details(deps: Deps, id: u32) -> StdResult<EndowmentDetailsResponse> {
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
    })
}

pub fn query_profile(deps: Deps, id: u32) -> StdResult<ProfileResponse> {
    let profile = PROFILES.load(deps.storage, id)?;
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    Ok(ProfileResponse {
        name: endowment.name,
        categories: endowment.categories,
        tier: endowment.tier,
        logo: endowment.logo,
        image: endowment.image,
        overview: profile.overview,
        url: profile.url,
        registration_number: profile.registration_number,
        country_of_origin: profile.country_of_origin,
        street_address: profile.street_address,
        contact_email: profile.contact_email,
        social_media_urls: profile.social_media_urls,
        number_of_employees: profile.number_of_employees,
        average_annual_budget: profile.average_annual_budget,
        annual_revenue: profile.annual_revenue,
        charity_navigator_rating: profile.charity_navigator_rating,
        endowment_type: endowment.endow_type,
    })
}

pub fn query_allowances(deps: Deps, id: u32, spender: String) -> StdResult<Allowances> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    let spender = deps.api.addr_validate(&spender)?;
    let allowances = ALLOWANCES.may_load(deps.storage, (&endowment.owner, &spender))?;
    Ok(allowances.unwrap_or_default())
}
