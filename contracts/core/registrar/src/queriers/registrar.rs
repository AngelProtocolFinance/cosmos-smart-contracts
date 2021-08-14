use crate::state::{read_registry_entries, CONFIG};
use angel_core::registrar_rsp::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        owner: config.owner.to_string(),
        accounts_code_id: config.accounts_code_id,
        treasury: config.treasury.to_string(),
        taxes: config.taxes.clone(),
        portals: config.portals_list(),
    };
    Ok(res)
}

pub fn query_portal_list(deps: Deps) -> StdResult<PortalListResponse> {
    let config = CONFIG.load(deps.storage)?;
    let list = PortalListResponse {
        portals: config.portals_list(),
    };
    Ok(list)
}

pub fn query_endowment_list(deps: Deps) -> StdResult<EndowmentListResponse> {
    let endowments = read_registry_entries(deps.storage)?;
    let list = EndowmentListResponse {
        endowments: endowments,
    };
    Ok(list)
}
