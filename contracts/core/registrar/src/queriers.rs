use crate::state::{read_registry_entries, read_vaults, vault_read, CONFIG};
use angel_core::responses::registrar::*;
use angel_core::structs::{VaultRate, YieldVault};
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
        default_vault: config.default_vault.to_string(),
        index_fund: config.index_fund_contract.to_string(),
        endowment_owners_group_addr: config.endowment_owners_group_addr,
        guardians_multisig_addr: config.guardians_multisig_addr,
        split_to_liquid: config.split_to_liquid,
        halo_token: match config.halo_token {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
        gov_contract: match config.gov_contract {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
        charity_shares_contract: match config.charity_shares_contract {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
    })
}

pub fn query_vault_list(deps: Deps) -> StdResult<VaultListResponse> {
    // returns a list of approved Vaults
    let vaults = read_vaults(deps.storage)?;
    Ok(VaultListResponse { vaults })
}

pub fn query_approved_vault_list(deps: Deps) -> StdResult<VaultListResponse> {
    // returns a list of approved Vaults
    let vaults = read_vaults(deps.storage)?;
    Ok(VaultListResponse {
        vaults: vaults.into_iter().filter(|p| p.approved).collect(),
    })
}

pub fn query_endowment_list(deps: Deps) -> StdResult<EndowmentListResponse> {
    let endowments = read_registry_entries(deps.storage)?;
    Ok(EndowmentListResponse { endowments })
}

pub fn query_vault_details(deps: Deps, vault_addr: String) -> StdResult<VaultDetailResponse> {
    // this fails if no vault is found
    let addr = deps.api.addr_validate(&vault_addr)?;
    let vault = vault_read(deps.storage).load(addr.as_bytes())?;
    Ok(VaultDetailResponse { vault })
}

pub fn query_approved_vaults_fx_rate(deps: Deps) -> StdResult<VaultRateResponse> {
    // returns a list of approved Vaults exchange rate
    let vaults = read_vaults(deps.storage)?;
    let approved_vaults: Vec<YieldVault> = vaults.into_iter().filter(|p| p.approved).collect();
    let mut vaults_rate: Vec<VaultRate> = vec![];
    for vault in approved_vaults.into_iter() {
        let fx_rate = vault_fx_rate(deps, vault.address.to_string());
        vaults_rate.push(VaultRate {
            vault_addr: vault.address,
            fx_rate: fx_rate,
        });
    }
    Ok(VaultRateResponse {
        vaults_rate: vaults_rate,
    })
}
