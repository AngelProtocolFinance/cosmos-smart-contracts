use crate::state::{Config, CONFIG};
use angel_core::error::ContractError;
use angel_core::index_fund_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use angel_core::structs::SplitDetails;
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20::Balance;

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
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        terra_alliance: msg.terra_alliance.unwrap_or(vec![]),
        active_fund_index: msg.active_fund_index.unwrap_or(Uint128::zero()),
        fund_rotation_limit: msg
            .fund_rotation_limit
            .unwrap_or(Uint128::from(500000 as u128)), // blocks
        fund_member_limit: msg.fund_member_limit.unwrap_or(10),
        funding_goal: msg.funding_goal.unwrap_or(Some(Balance::default())),
        split_to_liquid: msg.split_to_liquid.unwrap_or(SplitDetails::default()),
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
            registrar_contract: String::from("some-registrar-sc"),
            terra_alliance: Some(vec![]),
            active_fund_index: Some(Uint128::from(1u128)),
            fund_rotation_limit: Some(Uint128::from(500000u128)),
            fund_member_limit: Some(10),
            funding_goal: None,
            split_to_liquid: None,
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
