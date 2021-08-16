use crate::state::{portal_read, read_portals, read_registry_entries, CONFIG};
use angel_core::registrar_rsp::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let res = ConfigResponse {
        owner: config.owner.to_string(),
        accounts_code_id: config.accounts_code_id,
        treasury: config.treasury.to_string(),
        taxes: config.taxes.clone(),
    };
    Ok(res)
}

pub fn query_portal_list(deps: Deps) -> StdResult<PortalListResponse> {
    let portals = read_portals(deps.storage)?;
    let list = PortalListResponse { portals: portals };
    Ok(list)
}

pub fn query_endowment_list(deps: Deps) -> StdResult<EndowmentListResponse> {
    let endowments = read_registry_entries(deps.storage)?;
    let list = EndowmentListResponse {
        endowments: endowments,
    };
    Ok(list)
}

pub fn query_portal_details(deps: Deps, portal_addr: String) -> StdResult<PortalDetailResponse> {
    // this fails if no portal is found
    let addr = deps.api.addr_validate(&portal_addr)?;
    let portal = portal_read(deps.storage).load(&addr.as_bytes())?;
    let details = PortalDetailResponse { portal: portal };
    Ok(details)
}
