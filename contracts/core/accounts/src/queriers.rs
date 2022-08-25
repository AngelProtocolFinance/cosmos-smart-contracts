use crate::state::{CONFIG, ENDOWMENTS, STATES};
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::responses::accounts::*;
use angel_core::responses::registrar::VaultListResponse;
use angel_core::structs::{AccountType, BalanceInfo};
use angel_core::utils::vault_endowment_balance;
use cosmwasm_std::{to_binary, Deps, QueryRequest, StdResult, Uint128, WasmQuery};
use cw2::get_contract_version;
use cw20::{Balance, Cw20CoinVerified};
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
        closing_beneficiary: state
            .closing_beneficiary
            .map_or("".to_string(), |v| v.to_string()),
    })
}

pub fn query_account_balance(deps: Deps, id: u32) -> StdResult<BalanceInfo> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATES.load(deps.storage, id)?;
    // setup the basic response object w/ account's balances locked & liquid (held by this contract)
    let mut balances = state.balances;
    // add any Vault balances into locked
    let vault_list: VaultListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::VaultList {
                network: None,
                endowment_type: None,
                acct_type: None,
                approved: None,
                start_after: None,
                limit: None,
            })?,
        }))?;

    for vault in vault_list.vaults.iter() {
        let vault_bal = vault_endowment_balance(deps, vault.address.clone(), id);
        balances
            .locked_balance
            .add_tokens(Balance::Cw20(Cw20CoinVerified {
                amount: vault_bal,
                address: deps.api.addr_validate(&vault.address).unwrap(),
            }));
    }

    Ok(balances)
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
            state.balances.liquid_balance.get_denom_amount(denom).amount
        }
        (AssetInfo::Native(denom), AccountType::Locked) => {
            state.balances.locked_balance.get_denom_amount(denom).amount
        }
        (AssetInfo::Cw20(addr), AccountType::Liquid) => {
            state.balances.liquid_balance.get_token_amount(addr).amount
        }
        (AssetInfo::Cw20(addr), AccountType::Locked) => {
            state.balances.locked_balance.get_token_amount(addr).amount
        }
        (AssetInfo::Cw1155(_, _), _) => unimplemented!(),
    };
    Ok(balance)
}

pub fn query_endowment_details(deps: Deps, id: u32) -> StdResult<EndowmentDetailsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    Ok(EndowmentDetailsResponse {
        owner: endowment.owner,
        withdraw_before_maturity: endowment.withdraw_before_maturity,
        maturity_time: endowment.maturity_time,
        maturity_height: endowment.maturity_height,
        strategies: endowment.strategies,
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
    })
}
