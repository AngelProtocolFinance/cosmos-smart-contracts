use crate::state::{CONFIG, FEES, NETWORK_CONNECTIONS, VAULTS};
use angel_core::responses::registrar::*;
use cosmwasm_std::{Decimal, Deps, StdResult};
use cw2::get_contract_version;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: format!(
            "{}-{}",
            get_contract_version(deps.storage)?.contract,
            get_contract_version(deps.storage)?.version
        ),
        accounts_contract: config.accounts_contract.map(|addr| addr.to_string()),
        treasury: config.treasury.to_string(),
        rebalance: config.rebalance,
        index_fund: config.index_fund_contract.map(|addr| addr.to_string()),
        cw3_code: config.cw3_code,
        cw4_code: config.cw4_code,
        subdao_gov_code: config.subdao_gov_code,
        subdao_cw20_token_code: config.subdao_cw20_token_code,
        subdao_bonding_token_code: config.subdao_bonding_token_code,
        subdao_cw900_code: config.subdao_cw900_code,
        subdao_distributor_code: config.subdao_distributor_code,
        donation_match_code: config.donation_match_code,
        split_to_liquid: config.split_to_liquid,
        halo_token: Some(
            config
                .halo_token
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "".to_string()),
        ),
        halo_token_lp_contract: Some(
            config
                .halo_token_lp_contract
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "".to_string()),
        ),
        gov_contract: Some(
            config
                .gov_contract
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "".to_string()),
        ),
        donation_match_charites_contract: Some(
            config
                .donation_match_charites_contract
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "".to_string()),
        ),
        collector_addr: config
            .collector_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "".to_string()),
        collector_share: config.collector_share,
        accepted_tokens: config.accepted_tokens,
        charity_shares_contract: config.charity_shares_contract.map(|addr| addr.to_string()),
        swap_factory: config.swap_factory.map(|addr| addr.to_string()),
        applications_review: config.applications_review.to_string(),
        swaps_router: config.swaps_router.map(|addr| addr.to_string()),
        settings_controller: config.settings_controller.map(|addr| addr.to_string()),
    })
}

pub fn query_vault_details(deps: Deps, vault_addr: String) -> StdResult<VaultDetailResponse> {
    // this fails if no vault is found
    let addr = deps.api.addr_validate(&vault_addr)?;
    let vault = VAULTS.load(deps.storage, addr.as_bytes())?;
    Ok(VaultDetailResponse { vault })
}

pub fn query_network_connection(
    deps: Deps,
    chain_id: String,
) -> StdResult<NetworkConnectionResponse> {
    let network_connection = NETWORK_CONNECTIONS.load(deps.storage, &chain_id)?;
    Ok(NetworkConnectionResponse { network_connection })
}

pub fn query_fee(deps: Deps, name: String) -> StdResult<Decimal> {
    Ok(FEES.load(deps.storage, &name).unwrap_or(Decimal::zero()))
}
