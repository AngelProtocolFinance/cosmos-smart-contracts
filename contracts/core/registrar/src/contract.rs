use crate::executers::executers as ExecuteHandlers;
use crate::queriers::registrar as QueryHandlers;
use crate::state::{Config, CONFIG};
use angel_core::error::ContractError;
use angel_core::registrar_msg::*;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};

// version info for future migration info
const CONTRACT_NAME: &str = "registrar";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let treasury = deps.api.addr_validate(&msg.treasury)?;

    let configs = Config {
        owner: info.sender.clone(),
        index_fund_contract: info.sender.clone(),
        accounts_code_id: msg.accounts_code_id.unwrap_or(0u64),
        approved_charities: vec![],
        treasury: treasury,
        taxes: msg.taxes,
        default_portal: msg.default_portal.unwrap_or(info.sender),
    };

    CONFIG.save(deps.storage, &configs)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateEndowment(msg) => ExecuteHandlers::create_endowment(deps, env, info, msg),
        ExecuteMsg::UpdateConfig(msg) => ExecuteHandlers::update_config(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            ExecuteHandlers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            ExecuteHandlers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::CharityAdd { charity } => {
            ExecuteHandlers::charity_add(deps, env, info, charity)
        }
        ExecuteMsg::CharityRemove { charity } => {
            ExecuteHandlers::charity_remove(deps, env, info, charity)
        }
        ExecuteMsg::PortalAdd(msg) => ExecuteHandlers::portal_add(deps, env, info, msg),
        ExecuteMsg::PortalUpdateStatus {
            portal_addr,
            approved,
        } => ExecuteHandlers::portal_update_status(deps, env, info, portal_addr, approved),

        ExecuteMsg::PortalRemove { portal_addr } => {
            ExecuteHandlers::portal_remove(deps, env, info, portal_addr)
        }
    }
}

/// Replies back to the registrar from instantiate calls to Accounts SC (@ some code_id)
/// should be cuaght and handled to register the Endowment's newly created Accounts SC
/// in the REGISTRY storage
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => ExecuteHandlers::new_accounts_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryHandlers::query_config(deps)?),
        QueryMsg::EndowmentList {} => to_binary(&QueryHandlers::query_endowment_list(deps)?),
        QueryMsg::ApprovedPortalList {} => {
            to_binary(&QueryHandlers::query_approved_portal_list(deps)?)
        }
        QueryMsg::PortalList {} => to_binary(&QueryHandlers::query_portal_list(deps)?),
        QueryMsg::Portal { portal_addr } => {
            to_binary(&QueryHandlers::query_portal_details(deps, portal_addr)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
