
use std::string::String;

use cosmwasm_std::{
    to_binary, entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20::Balance;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, FundResponse};
use crate::state::{Config, CONFIG, FUNDS};


// Note, you can use StdResult in some functions where you do not
// make use of the custom errors

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Default placeholders used in config to check compiling. Should take from InistantiateMsg.
    let configs = Config {
        owner: info.sender,
        account_ledgers_sc: deps.api.addr_validate(&msg.account_ledgers_sc)?,
        terra_alliance: vec![],
        active_fund_index: Uint128::zero(),
        fund_rotation_limit: Uint128::from(500000 as u128), // blocks
        fund_member_limit: 10,
        funding_goal: Some(Balance::default()),
    };
    CONFIG.save(deps.storage, &configs)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {

        QueryMsg::GetFund {fund_id} => to_binary(&query_fund(_deps, fund_id)?),
    }

}

fn query_fund(deps: Deps, fund_id: String) -> StdResult<FundResponse> {
    
    let fund = FUNDS.load(deps.storage, fund_id)?;
    let name = fund.name;
    let description = fund.description;
    let members = fund.members;

    Ok(FundResponse {
        name,
        description,
        members,
        })
    
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
    fn increment() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            count: Uint128::from(17u128),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(18u128), value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            count: Uint128::from(17u128),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset {
            count: Uint128::from(5u128),
        };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset {
            count: Uint128::from(5u128),
        };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::from(5u128), value.count);

    }
}
