use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw2::{get_contract_version, set_contract_version};

use angel_core::errors::core::ContractError;
use angel_core::msgs::accounts_settings_controller::*;

use crate::state::{Config, CONFIG};
use crate::{executers, queriers};

// version info for future migration info
const CONTRACT_NAME: &str = "accounts-settings-controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner_sc)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        },
    )?;

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
        ExecuteMsg::CreateEndowmentSettings(msg) => {
            executers::create_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentController(msg) => {
            executers::update_endowment_controller(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentFees(msg) => {
            executers::update_endowment_fees(deps, env, info, msg)
        }
        ExecuteMsg::SetupDao {
            endowment_id,
            setup,
        } => executers::setup_dao(deps, env, info, endowment_id, setup),
        ExecuteMsg::SetupDonationMatch {
            endowment_id,
            setup,
        } => executers::setup_donation_match(deps, env, info, endowment_id, setup),
        ExecuteMsg::UpdateDelegate {
            endowment_id,
            setting,
            action,
            delegate_address,
            delegate_expiry,
        } => executers::update_delegate(
            deps,
            env,
            info,
            endowment_id,
            setting,
            action,
            delegate_address,
            delegate_expiry,
        ),
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::EndowmentSettings { id } => {
            to_binary(&queriers::query_endowment_settings(deps, id)?)
        }
        QueryMsg::EndowmentController { id } => {
            to_binary(&queriers::query_endowment_controller(deps, id)?)
        }
        QueryMsg::EndowmentPermissions {
            id,
            setting_updater,
            endowment_owner,
        } => to_binary(&queriers::query_endowment_permissions(
            deps,
            env,
            id,
            setting_updater,
            endowment_owner,
        )?),
    }
}

/// Replies back to the Endowment Account from various multisig contract calls (@ some passed code_id)
/// should be caught and handled to fire subsequent setup calls and ultimately the storing of the multisig
/// as the Accounts endowment owner
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        // 0 => executers::cw3_reply(deps, env, msg.result),
        1 => executers::dao_reply(deps, env, msg.result),
        2 => executers::donation_match_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        }));
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        }));
    }

    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
