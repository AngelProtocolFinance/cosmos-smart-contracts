use crate::state::{read_registry_entries, read_vaults, vault_read, CONFIG};
use angel_core::registrar_rsp::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        owner: config.owner.to_string(),
        approved_coins: config.human_approved_coins(),
        accounts_code_id: config.accounts_code_id,
        treasury: config.treasury.to_string(),
        taxes: config.taxes,
    };
    Ok(res)
}

pub fn query_vault_details(deps: Deps, vault_addr: String) -> StdResult<VaultDetailResponse> {
    // this fails if no vault is found
    let addr = deps.api.addr_validate(&vault_addr)?;
    let vault = vault_read(deps.storage).load(&addr.as_bytes())?;
    let details = VaultDetailResponse { vault: vault };
    Ok(details)
}

pub fn query_vault_list(deps: Deps) -> StdResult<VaultListResponse> {
    let vaults = read_vaults(deps.storage)?;
    let list = VaultListResponse { vaults: vaults };
    Ok(list)
}

pub fn query_endowment_list(deps: Deps) -> StdResult<EndowmentListResponse> {
    let endowments = read_registry_entries(deps.storage)?;
    let list = EndowmentListResponse {
        endowments: endowments,
    };
    Ok(list)
}
