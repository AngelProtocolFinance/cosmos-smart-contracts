use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr
};

use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg, UpdateConfigMsg, CreateAccountMsg};
use crate::state::{Config, CONFIG, Account, ACCOUNTS, Splits, SplitParameters};

// Version info for future migration info
const CONTRACT_NAME: &str = "liquid-account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &Config {
        owner: info.sender,
        locked_account: None,
        index_fund: None,
        investment_strategy: None,
    })?;

    Ok(Response::default())
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

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateAccount(msg) => execute_create(deps, _env, msg, &info.sender),
        ExecuteMsg::UpdateConfig(msg) => update_config(deps, _env, info, msg),
        ExecuteMsg::Approve { address } => execute_approve(deps, _env, info, address),
    }
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        match msg.owner {
            Some(owner) => { config.owner = owner; },
            None => {}
        }

        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn execute_approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // TODO: raise error if there no account
    let mut account = ACCOUNTS.load(deps.storage, address.clone())?;

    if info.sender != account.arbiter {
        Err(ContractError::Unauthorized {})
    } else if account.approved {
        Err(ContractError::AccountAlreadyApproved {})
    } else {
        account.approved = true;

        ACCOUNTS.save(deps.storage, address.clone(), &account)?;

        Ok(Response::default())
    }
}

pub fn execute_create(
    deps: DepsMut,
    _env: Env,
    msg: CreateAccountMsg,
    sender: &Addr,
) -> Result<Response, ContractError> {

    // TODO: check that sender === locked_account.address 

    let account = Account {
        arbiter: deps.api.addr_validate(&msg.arbiter)?,
        approved: false,
        beneficiary: deps.api.addr_validate(&msg.beneficiary)?,
        originator: sender.clone(),
        splits: Splits {
            deposit: SplitParameters {
                max: 100,
                min: 20,
                default: 50,
            },
            interest: SplitParameters {
                max: 100,
                min: 20,
                default: 50,
            },
        },
        end_height: msg.end_height,
        end_time: msg.end_time,
    };

    ACCOUNTS.update(
        deps.storage,
        sender.clone().into(),
        |existing| match existing {
            None => Ok(account),
            Some(_) => Err(ContractError::AlreadyInUse {}),
        },
    )?;

    Ok(Response::default())
}

pub fn execute_deposit() {
    // TODO
}

pub fn execute_withdrawal() {
    // TODO
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
    }
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
