use crate::handlers::{executers as ExecuteHandlers, queriers as QueryHandlers};
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

    let configs = Config {
        owner: info.sender.clone(),
        index_fund_contract: info.sender,
        approved_coins: vec![],
        accounts_code_id: msg.accounts_code_id.unwrap_or(0u64),
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
        ExecuteMsg::CreateEndowment(msg) => {
            ExecuteHandlers::execute_create_endowment(deps, env, info, msg)
        }
        ExecuteMsg::UpdateConfig(msg) => {
            ExecuteHandlers::execute_update_config(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            ExecuteHandlers::execute_update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            ExecuteHandlers::execute_update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::VaultAdd {
            vault_addr,
            vault_name,
            vault_description,
        } => ExecuteHandlers::vault_add(deps, env, info, vault_addr, vault_name, vault_description),
        ExecuteMsg::VaultUpdateStatus {
            vault_addr,
            approved,
        } => ExecuteHandlers::vault_update_status(deps, env, info, vault_addr, approved),
        ExecuteMsg::VaultRemove { vault_addr } => {
            ExecuteHandlers::vault_remove(deps, env, info, vault_addr)
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
        QueryMsg::Vault { vault_addr } => {
            to_binary(&QueryHandlers::query_vault_details(deps, vault_addr)?)
        }
        QueryMsg::VaultList {} => to_binary(&QueryHandlers::query_vault_list(deps)?),
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
