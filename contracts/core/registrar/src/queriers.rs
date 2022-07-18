use crate::state::{
    read_registry_entries, read_vaults, CONFIG, ENDOWTYPE_FEES, NETWORK_CONNECTIONS, REGISTRY,
    VAULTS,
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
        cw3_code: config.cw3_code,
        cw4_code: config.cw4_code,
        subdao_gov_code: config.subdao_gov_code,
        subdao_token_code: config.subdao_token_code,
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

pub fn query_fees(deps: Deps) -> StdResult<FeesResponse> {
    // returns all Fees(both BaseFee & all of the EndowmentTypeFees)
    let tax_rate = CONFIG.load(deps.storage)?.tax_rate;
    let endowtype_charity = ENDOWTYPE_FEES
        .load(deps.storage, "charity".to_string())
        .unwrap_or(None);
    let endowtype_normal = ENDOWTYPE_FEES
        .load(deps.storage, "normal".to_string())
        .unwrap_or(None);
    Ok(FeesResponse {
        tax_rate,
        endowtype_charity,
        endowtype_normal,
    })
}

pub fn query_network_connection(
    deps: Deps,
    chain_id: String,
) -> StdResult<NetworkConnectionResponse> {
    let network_connection = NETWORK_CONNECTIONS.load(deps.storage, &chain_id)?;
    Ok(NetworkConnectionResponse { network_connection })
}
