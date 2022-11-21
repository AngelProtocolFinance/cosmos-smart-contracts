use crate::executers;
use crate::queriers;
use crate::state::ENDOWMENTSETTINGS;
use crate::state::{Config, CONFIG};
use angel_core::errors::core::ContractError;
use angel_core::messages::settings_controller::*;
use angel_core::structs::SettingsController;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner_sc)?,
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
    todo!()

    // match msg {
    //     ExecuteMsg::UpdateEndowmentSettings(msg) => {
    //         executers::update_endowment_settings(deps, env, info, msg)
    //     }
    //     ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
    //     ExecuteMsg::UpdateOwner { new_owner } => {
    //         executers::update_owner(deps, env, info, new_owner)
    //     }
    //     ExecuteMsg::UpdateEndowmentFees(msg) => {
    //         executers::update_endowment_fees(deps, env, info, msg)
    //     }
    //     ExecuteMsg::SetupDao {
    //         endowment_id,
    //         setup,
    //     } => executers::setup_dao(deps, env, info, endowment_id, setup),
    //     ExecuteMsg::SetupDonationMatch {
    //         endowment_id,
    //         setup,
    //     } => executers::setup_donation_match(deps, env, info, endowment_id, setup),
    // }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::EndowmentSettings { id } => {
            to_binary(&queriers::query_endowment_settings(deps, id)?)
        }
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
