use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{CONFIG, Config, Strategy};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let configs = Config {
        owner: info.sender,
        account_ledgers_sc: deps.api.addr_validate(&msg.account_ledgers_sc)?,
    };
    CONFIG.save(deps.storage, &configs)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateStrategy {
            eid,
            account_type,
            strategy,
        } => execute_update_strategy(deps, env, info, eid, account_type, strategy),
    }
}

pub fn execute_update_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    eid: String,
    account_type: String,
    strategy: Strategy,
) -> Result<Response, ContractError> {
    // TODO (borodanov): implement
    // check sender is endowment owner
    // check eid is correct
    // check is approved?
    // then updateStrategy in the account-ledgers SC

    Ok(Response::default())
}

pub fn update_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    eid: String,
    account_type: String,
    strategy: Strategy,
) -> Result<Response, ContractError> {
    // TODO (borodanov): implement

    Ok(Response::default())
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[entry_point]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            account_ledgers_sc: String::from("some-account-ledgers-sc"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_update_strategy() {
        // TODO (borodanov): implement
        assert_eq!(0, 0);
    }
}
