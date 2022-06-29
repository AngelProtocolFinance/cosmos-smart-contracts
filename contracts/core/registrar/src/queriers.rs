use crate::state::{
    read_registry_entries, read_vaults, CONFIG, NETWORK_CONNECTIONS, REGISTRY, VAULTS,
};
use angel_core::responses::registrar::*;
use angel_core::structs::{EndowmentEntry, EndowmentType, Tier, VaultRate};
use angel_core::utils::vault_fx_rate;
use cosmwasm_std::{Deps, StdResult};
use cw2::get_contract_version;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: get_contract_version(deps.storage)?.contract,
        accounts_code_id: config.accounts_code_id,
        treasury: config.treasury.to_string(),
        tax_rate: config.tax_rate,
        default_vault: config.default_vault.map(|addr| addr.to_string()),
        index_fund: config.index_fund_contract.map(|addr| addr.to_string()),
        split_to_liquid: config.split_to_liquid,
        halo_token: config.halo_token.map(|addr| addr.to_string()),
        gov_contract: config.gov_contract.map(|addr| addr.to_string()),
        charity_shares_contract: config.charity_shares_contract.map(|addr| addr.to_string()),
        cw3_code: config.cw3_code,
        cw4_code: config.cw4_code,
        accepted_tokens: config.accepted_tokens,
    })
}

pub fn query_vault_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u64>,
) -> StdResult<VaultListResponse> {
    // returns a list of all Vaults
    let addr = match start_after {
        Some(start_after) => Some(deps.api.addr_validate(&start_after)?),
        None => None,
    };
    let vaults = read_vaults(deps.storage, addr, limit)?;
    Ok(VaultListResponse { vaults })
}

pub fn query_approved_vault_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u64>,
) -> StdResult<VaultListResponse> {
    // returns a list of approved Vaults
    let addr = match start_after {
        Some(start_after) => Some(deps.api.addr_validate(&start_after)?),
        None => None,
    };
    let vaults = read_vaults(deps.storage, addr, limit)?;
    Ok(VaultListResponse { vaults })
}

pub fn query_endowment_details(
    deps: Deps,
    endowment_addr: String,
) -> StdResult<EndowmentDetailResponse> {
    let endowment = REGISTRY.load(deps.storage, endowment_addr.as_bytes())?;
    Ok(EndowmentDetailResponse { endowment })
}

pub fn query_endowment_list(
    deps: Deps,
    name: Option<Option<String>>,
    owner: Option<Option<String>>,
    status: Option<String>,             // String -> EndowmentStatus
    tier: Option<Option<String>>,       // String -> Tier
    un_sdg: Option<Option<u64>>,        // u64 -> UN SDG
    endow_type: Option<Option<String>>, // String -> EndowmentType
) -> StdResult<EndowmentListResponse> {
    let endowments = read_registry_entries(deps.storage)?;
    let endowments = match name {
        Some(nm) => endowments
            .into_iter()
            .filter(|e| e.name == nm)
            .collect::<Vec<EndowmentEntry>>(),
        None => endowments,
    };

    let endowments = match owner {
        Some(oner) => endowments
            .into_iter()
            .filter(|e| e.owner == oner)
            .collect::<Vec<EndowmentEntry>>(),
        None => endowments,
    };
    let endowments = match status {
        Some(status) => endowments
            .into_iter()
            .filter(|e| e.status.to_string() == status)
            .collect::<Vec<EndowmentEntry>>(),
        None => endowments,
    };
    let endowments = match tier {
        Some(tier) => {
            let tier = tier.and_then(|v| match v.as_str() {
                "1" => Some(Tier::Level1),
                "2" => Some(Tier::Level2),
                "3" => Some(Tier::Level3),
                _ => unimplemented!(),
            });
            endowments
                .into_iter()
                .filter(|e| e.tier == tier)
                .collect::<Vec<EndowmentEntry>>()
        }
        None => endowments,
    };
    let endowments = match un_sdg {
        Some(un_sdg) => endowments
            .into_iter()
            .filter(|e| e.un_sdg == un_sdg)
            .collect::<Vec<EndowmentEntry>>(),
        None => endowments,
    };
    let endowments = match endow_type {
        Some(endow_type) => {
            let end_ty = endow_type.and_then(|v| match v.as_str() {
                "charity" => Some(EndowmentType::Charity),
                "normal" => Some(EndowmentType::Normal),
                _ => unimplemented!(),
            });
            endowments
                .into_iter()
                .filter(|e| e.endow_type == end_ty)
                .collect::<Vec<EndowmentEntry>>()
        }
        None => endowments,
    };

    Ok(EndowmentListResponse { endowments })
}

pub fn query_vault_details(deps: Deps, vault_addr: String) -> StdResult<VaultDetailResponse> {
    // this fails if no vault is found
    let addr = deps.api.addr_validate(&vault_addr)?;
    let vault = VAULTS.load(deps.storage, addr.as_bytes())?;
    Ok(VaultDetailResponse { vault })
}

pub fn query_approved_vaults_fx_rate(deps: Deps) -> StdResult<VaultRateResponse> {
    // returns a list of approved Vaults exchange rate
    let vaults = read_vaults(deps.storage, None, None)?;
    let mut vaults_rate: Vec<VaultRate> = vec![];
    for vault in vaults.iter().filter(|p| p.approved) {
        let fx_rate = vault_fx_rate(deps, vault.address.to_string());
        vaults_rate.push(VaultRate {
            vault_addr: vault.address.clone(),
            fx_rate,
        });
    }
    Ok(VaultRateResponse { vaults_rate })
}

pub fn query_network_connection(
    deps: Deps,
    chain_id: String,
) -> StdResult<NetworkConnectionResponse> {
    let network_connection = NETWORK_CONNECTIONS.load(deps.storage, &chain_id)?;
    Ok(NetworkConnectionResponse { network_connection })
}
