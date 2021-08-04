use crate::state::{Config, CONFIG, FUNDS};
use angel_core::error::ContractError;
use angel_core::index_fund_msg::*;
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
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateFund(msg) => execute_create_index_fund(deps, info, msg),
        ExecuteMsg::RemoveFund(msg) => execute_remove_index_fund(deps, info, msg.fund_id),
        ExecuteMsg::RemoveMember(msg) => execute_remove_member(deps, info, msg.member),
        ExecuteMsg::UpdateMembers(msg) => execute_update_fund_members(deps, info, msg),
    }
}

fn execute_create_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    msg: CreateFundMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Add the new Fund to FUNDS
    FUNDS.save(deps.storage, msg.fund_id, &msg.fund)?;

    Ok(Response::default())
}

fn execute_remove_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    fund_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // this will fail if fund ID passed is not found
    let _fund = FUNDS.load(deps.storage, fund_id.clone());

    // remove the fund from FUNDS
    FUNDS.remove(deps.storage, fund_id);

    Ok(Response::default())
}

fn execute_update_fund_members(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateMembersMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // this will fail if fund ID passed is not found
    let mut fund = FUNDS.load(deps.storage, msg.fund_id.clone())?;

    // add members to the fund, only if they do not already exist
    for add in msg.add.into_iter() {
        let add_addr = deps.api.addr_validate(&add)?;
        let pos = fund.members.iter().position(|m| *m == add_addr);
        // ignore if that member was found in the list
        if pos != None {
            fund.members.push(add_addr);
        }
    }

    // remove the members to the fund
    for remove in msg.remove.into_iter() {
        let remove_addr = deps.api.addr_validate(&remove)?;
        // ignore if no member is found
        if let Some(pos) = fund.members.iter().position(|m| *m == remove_addr) {
            fund.members.swap_remove(pos);
        }
    }

    // save revised fund to storage
    FUNDS.save(deps.storage, msg.fund_id, &fund)?;

    Ok(Response::default())
}

fn execute_remove_member(
    deps: DepsMut,
    info: MessageInfo,
    member: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // check the string is proper addr
    let _member_addr = deps.api.addr_validate(&member)?;

    // TO DO: build out member replacement logic.
    // Check all Funds for the given member and remove the member Addr, if found.
    Ok(Response::default())
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // TO DO: look up a single fund details
        // TO DO: look up list of all funds
    }
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
