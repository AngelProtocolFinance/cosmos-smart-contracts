use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};
use crate::state::{Config, CONFIG};

// version info for future migration info
const CONTRACT_NAME: &str = "liquid-account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender,
        locked_account: None,
        index_fund: None,
        investment_strategy: None,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
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

#[cfg(test)]
mod tests {
    // use super::*;
    // use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    // use cosmwasm_std::{coins};

    #[test]
    fn proper_initialization() {
        // let mut deps = mock_dependencies(&[]);

        // let info = mock_info("creator", &coins(1000, "earth"));

        // TODO
    }

    #[test]
    fn increment() {
        // let mut deps = mock_dependencies(&coins(2, "token"));

        // let info = mock_info("creator", &coins(2, "token"));

        // TODO
    }

    #[test]
    fn reset() {
        // let mut deps = mock_dependencies(&coins(2, "token"));

        // let info = mock_info("creator", &coins(2, "token"));

        // TODO
    }
}
