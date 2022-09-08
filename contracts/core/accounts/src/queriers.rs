use crate::state::{read_endowments, Endowment, CONFIG, ENDOWMENTS, STATES};
use angel_core::responses::accounts::*;
use angel_core::structs::{
    AccountType, EndowmentBalanceResponse, EndowmentEntry, EndowmentType, Tier, VaultsBalanceInfo,
};
use angel_core::utils::vault_endowment_balance;
use cosmwasm_std::{Deps, StdResult, Uint128};
use cw2::get_contract_version;
use cw_asset::AssetInfo;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: get_contract_version(deps.storage)?.contract,
        registrar_contract: config.registrar_contract.to_string(),
    })
}

pub fn query_state(deps: Deps, id: u32) -> StdResult<StateResponse> {
    let state = STATES.load(deps.storage, id)?;

    Ok(StateResponse {
        donations_received: state.donations_received,
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
    let mut oneoff_locked = vec![];
    for vault in endowment.oneoff_vaults.locked.into_iter() {
        let vault_bal = vault_endowment_balance(deps, vault.clone().to_string(), id);
        oneoff_locked.push((vault.to_string(), vault_bal));
    }
    let mut oneoff_liquid = vec![];
    for vault in endowment.oneoff_vaults.liquid.into_iter() {
        let vault_bal = vault_endowment_balance(deps, vault.clone().to_string(), id);
        oneoff_liquid.push((vault.to_string(), vault_bal));
    }
    let mut strategies_locked = vec![];

    // process all strategies vaults
    for strat in endowment.strategies.locked.iter() {
        let vault_bal = vault_endowment_balance(deps, strat.vault.clone(), id);
        strategies_locked.push((strat.vault.to_string(), vault_bal));
    }
    let mut strategies_liquid = vec![];
    for strat in endowment.strategies.liquid.iter() {
        let vault_bal = vault_endowment_balance(deps, strat.vault.clone(), id);
        strategies_liquid.push((strat.vault.to_string(), vault_bal));
    }

    Ok(EndowmentBalanceResponse {
        tokens_on_hand,
        oneoff_locked,
        oneoff_liquid,
        strategies_locked,
        strategies_liquid,
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
        (AssetInfo::Cw1155(_, _), _) => unimplemented!(),
    };
    Ok(balance)
}

pub fn query_endowment_list(
    deps: Deps,
    name: Option<Option<String>>,
    owner: Option<String>,
    status: Option<String>, // EndowmentStatus
    tier: Option<Option<String>>,
    endow_type: Option<String>, // EndowmentType
) -> StdResult<EndowmentListResponse> {
    let endowments: Vec<(u32, Endowment)> = read_endowments(deps.storage)?;
    let entries: Vec<EndowmentEntry> = endowments
        .iter()
        .map(|(i, e)| EndowmentEntry {
            id: *i,
            owner: e.owner.to_string(),
            status: e.status.clone(),
            endow_type: e.profile.endow_type.clone(),
            name: Some(e.profile.name.clone()),
            logo: e.profile.logo.clone(),
            image: e.profile.image.clone(),
            tier: match e.profile.tier.unwrap() {
                1 => Some(Tier::Level1),
                2 => Some(Tier::Level2),
                3 => Some(Tier::Level3),
                _ => None,
            },
            categories: e.profile.categories.clone(),
        })
        .collect();
    let entries = match name {
        Some(nm) => entries
            .into_iter()
            .filter(|e| e.name == nm)
            .collect::<Vec<EndowmentEntry>>(),
        None => entries,
    };

    let entries = match owner {
        Some(owner) => entries
            .into_iter()
            .filter(|e| e.owner == owner)
            .collect::<Vec<EndowmentEntry>>(),
        None => entries,
    };
    let entries = match status {
        Some(status) => entries
            .into_iter()
            .filter(|e| e.status.to_string() == status)
            .collect::<Vec<EndowmentEntry>>(),
        None => entries,
    };
    let entries = match tier {
        Some(tier) => {
            let tier = tier.and_then(|v| match v.as_str() {
                "1" => Some(Tier::Level1),
                "2" => Some(Tier::Level2),
                "3" => Some(Tier::Level3),
                _ => unimplemented!(),
            });
            entries
                .into_iter()
                .filter(|e| e.tier == tier)
                .collect::<Vec<EndowmentEntry>>()
        }
        None => entries,
    };
    let entries = match endow_type {
        Some(endow_type) => {
            let end_ty = match endow_type.as_str() {
                "charity" => EndowmentType::Charity,
                "normal" => EndowmentType::Normal,
                _ => unimplemented!(),
            };
            entries
                .into_iter()
                .filter(|e| e.endow_type == end_ty)
                .collect::<Vec<EndowmentEntry>>()
        }
        None => entries,
    };
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
        endow_type: endowment.profile.endow_type,
        withdraw_before_maturity: endowment.withdraw_before_maturity,
        maturity_time: endowment.maturity_time,
        maturity_height: endowment.maturity_height,
        strategies: endowment.strategies,
        oneoff_vaults: endowment.oneoff_vaults,
        rebalance: endowment.rebalance,
        kyc_donors_only: endowment.kyc_donors_only,
        deposit_approved: endowment.deposit_approved,
        withdraw_approved: endowment.withdraw_approved,
        pending_redemptions: endowment.pending_redemptions,
    })
}

pub fn query_profile(deps: Deps, id: u32) -> StdResult<ProfileResponse> {
    let profile = ENDOWMENTS.load(deps.storage, id)?.profile;
    Ok(ProfileResponse {
        name: profile.name,
        overview: profile.overview,
        categories: profile.categories,
        tier: profile.tier,
        logo: profile.logo,
        image: profile.image,
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
    })
}
