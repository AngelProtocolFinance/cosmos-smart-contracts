use crate::state::{read_vaults, CONFIG, FEES, NETWORK_CONNECTIONS, VAULTS};
use angel_core::responses::registrar::*;
use angel_core::structs::{AccountType, EndowmentType, VaultType};
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
        split_to_liquid: config.split_to_liquid,
        halo_token: config.halo_token.map(|addr| addr.to_string()),
        gov_contract: config.gov_contract.map(|addr| addr.to_string()),
        charity_shares_contract: config.charity_shares_contract.map(|addr| addr.to_string()),
        cw3_code: config.cw3_code,
        cw4_code: config.cw4_code,
        accepted_tokens: config.accepted_tokens,
        applications_review: config.applications_review.to_string(),
        applications_impact_review: config.applications_impact_review.to_string(),
        swaps_router: config.swaps_router.map(|addr| addr.to_string()),
    })
}

pub fn query_vault_list(
    deps: Deps,
    network: Option<String>,
    endowment_type: Option<EndowmentType>,
    acct_type: Option<AccountType>,
    vault_type: Option<VaultType>,
    approved: Option<bool>,
    start_after: Option<String>,
    limit: Option<u64>,
) -> StdResult<VaultListResponse> {
    // returns a list of all Vaults
    let addr = match start_after {
        Some(start_after) => Some(deps.api.addr_validate(&start_after)?),
        None => None,
    };
    let vaults = read_vaults(
        deps.storage,
        network,
        endowment_type,
        acct_type,
        vault_type,
        approved,
        addr,
        limit,
    )?;
    Ok(VaultListResponse { vaults })
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
