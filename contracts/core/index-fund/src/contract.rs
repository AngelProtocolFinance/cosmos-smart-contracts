use crate::handlers::{executers as ExecuteHandlers, queriers as QueryHandlers};
use crate::state::{Config, State, CONFIG, STATE};
use angel_core::error::ContractError;
use angel_core::index_fund_msg::*;
use angel_core::structs::{AcceptedTokens, SplitDetails};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

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
        owner: info.sender.clone(),
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        fund_rotation: msg.fund_rotation.unwrap_or(500000 as u64), // blocks
        fund_member_limit: msg.fund_member_limit.unwrap_or(10),
        funding_goal: msg.funding_goal.unwrap_or(Some(Uint128::zero())),
        split_to_liquid: msg.split_to_liquid.unwrap_or(SplitDetails::default()),
        accepted_tokens: msg.accepted_tokens.unwrap_or(AcceptedTokens::default()),
    };
    CONFIG.save(deps.storage, &configs)?;

    // setup default state values
    STATE.save(deps.storage, &State::default())?;
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
        ExecuteMsg::UpdateOwner { new_owner } => {
            ExecuteHandlers::update_owner(deps, info, new_owner)
        }
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            ExecuteHandlers::update_registrar(deps, info, new_registrar)
        }
        ExecuteMsg::UpdateTcaList { new_list } => {
            ExecuteHandlers::update_tca_list(deps, info, new_list)
        }
        ExecuteMsg::CreateFund { fund } => ExecuteHandlers::create_index_fund(deps, info, fund),
        ExecuteMsg::RemoveFund(msg) => ExecuteHandlers::remove_index_fund(deps, info, msg.fund_id),
        ExecuteMsg::RemoveMember(msg) => ExecuteHandlers::remove_member(deps, info, msg.member),
        ExecuteMsg::UpdateMembers(msg) => ExecuteHandlers::update_fund_members(deps, info, msg),
        ExecuteMsg::Deposit(msg) => {
            ExecuteHandlers::deposit(deps, env, info.sender, info.funds[0].amount, msg)
        }
        ExecuteMsg::Recieve(msg) => ExecuteHandlers::receive(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryHandlers::config(deps)?),
        QueryMsg::State {} => to_binary(&QueryHandlers::state(deps)?),
        QueryMsg::TcaList {} => to_binary(&QueryHandlers::tca_list(deps)?),
        QueryMsg::FundsList {} => to_binary(&QueryHandlers::funds_list(deps)?),
        QueryMsg::FundDetails { fund_id } => {
            to_binary(&QueryHandlers::fund_details(deps, fund_id)?)
        }
        QueryMsg::ActiveFundDetails {} => to_binary(&QueryHandlers::active_fund_details(deps)?),
        QueryMsg::ActiveFundDonations {} => to_binary(&QueryHandlers::active_fund_donations(deps)?),
    }
}

#[entry_point]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
