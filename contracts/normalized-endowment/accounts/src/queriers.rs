use crate::state::{CONFIG, ENDOWMENTS, REDEMPTIONS, STATES};
use angel_core::messages::vault::QueryMsg as VaultQuerier;
use angel_core::responses::accounts::*;
use angel_core::structs::BalanceInfo;
use cosmwasm_std::{to_binary, Deps, Env, Order, QueryRequest, StdResult, WasmQuery};
use cw2::get_contract_version;
use cw20::{Balance, Cw20CoinVerified};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: get_contract_version(deps.storage)?.contract,
        registrar_contract: config.registrar_contract.to_string(),
    })
}

pub fn query_state(deps: Deps, id: String) -> StdResult<StateResponse> {
    let state = STATES.load(deps.storage, &id)?;

    Ok(StateResponse {
        donations_received: state.donations_received,
        closing_endowment: state.closing_endowment,
        closing_beneficiary: state
            .closing_beneficiary
            .map_or("".to_string(), |v| v.to_string()),
        last_earnings_harvest: state.last_earnings_harvest,
        last_harvest_fx: state
            .last_harvest_fx
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
    })
}

pub fn query_account_balance(deps: Deps, env: Env, id: String) -> StdResult<BalanceInfo> {
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;
    let state = STATES.load(deps.storage, &id)?;
    // setup the basic response object w/ account's balances locked & liquid (held by this contract)
    let mut balances = state.balances;
    // add stategies' (locked) balances
    for strategy in endowment.strategies {
        balances
            .locked_balance
            .add_tokens(Balance::Cw20(Cw20CoinVerified {
                amount: deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: strategy.vault.to_string(),
                    msg: to_binary(&VaultQuerier::Balance {
                        address: env.contract.address.to_string(),
                    })?,
                }))?,
                address: deps.api.addr_validate(&strategy.vault)?,
            }));
    }

    Ok(balances)
}

pub fn query_endowment_details(deps: Deps, id: String) -> StdResult<EndowmentDetailsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;
    let redemptions = REDEMPTIONS.load(deps.storage, &id)?;
    Ok(EndowmentDetailsResponse {
        owner: endowment.owner,
        dao: endowment.dao,
        dao_token: endowment.dao_token,
        name: endowment.profile.name,
        description: endowment.profile.overview,
        withdraw_before_maturity: endowment.withdraw_before_maturity,
        maturity_time: endowment.maturity_time,
        strategies: endowment.strategies,
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
        settings_controller: endowment.settings_controller,
        pending_redemptions: redemptions,
        deposit_approved: endowment.deposit_approved,
        withdraw_approved: endowment.withdraw_approved,
    })
}

pub fn query_profile(deps: Deps, id: String) -> StdResult<ProfileResponse> {
    let profile = ENDOWMENTS.load(deps.storage, &id)?.profile;
    Ok(ProfileResponse {
        name: profile.name,
        overview: profile.overview,
        un_sdg: profile.un_sdg,
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
        endowment_type: profile.endow_type,
    })
}

pub fn query_endowment_fees(deps: Deps, id: String) -> StdResult<EndowmentFeesResponse> {
    let endowment = ENDOWMENTS.load(deps.storage, &id)?;
    Ok(EndowmentFeesResponse {
        earnings_fee: endowment.earnings_fee,
        deposit_fee: endowment.deposit_fee,
        withdraw_fee: endowment.withdraw_fee,
        aum_fee: endowment.aum_fee,
    })
}

pub fn query_all_ids(deps: Deps) -> StdResult<Vec<String>> {
    Ok(ENDOWMENTS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|id| String::from(id.unwrap()))
        .collect::<Vec<String>>())
}
