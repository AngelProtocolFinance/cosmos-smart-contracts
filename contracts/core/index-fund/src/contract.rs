use crate::executers;
use crate::queriers;
use crate::state::{Config, State, CONFIG, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::index_fund::*;
use angel_core::structs::AcceptedTokens;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

// version info for future migration info
const CONTRACT_NAME: &str = "index-fund";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let configs = Config {
        owner: info.sender,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        fund_rotation: msg.fund_rotation.unwrap_or(None), // blocks
        fund_member_limit: msg.fund_member_limit.unwrap_or(10),
        funding_goal: msg.funding_goal.unwrap_or(None),
        accepted_tokens: msg.accepted_tokens.unwrap_or_else(AcceptedTokens::default),
    };
    CONFIG.save(deps.storage, &configs)?;

    // setup default state values
    STATE.save(
        deps.storage,
        &State {
            total_funds: 0,
            active_fund: 0,
            round_donations: Uint128::zero(),
            next_rotation_block: env.block.height + configs.fund_rotation.unwrap_or(0u64),
            terra_alliance: vec![],
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
        ExecuteMsg::UpdateOwner { new_owner } => executers::update_owner(deps, info, new_owner),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, info, new_registrar)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, info, msg),
        ExecuteMsg::UpdateTcaList { new_list } => executers::update_tca_list(deps, info, new_list),
        ExecuteMsg::CreateFund { fund } => executers::create_index_fund(deps, info, fund),
        ExecuteMsg::RemoveFund { fund_id } => {
            executers::remove_index_fund(deps, env, info, fund_id)
        }
        ExecuteMsg::RemoveMember(msg) => executers::remove_member(deps, info, msg.member),
        ExecuteMsg::UpdateMembers {
            fund_id,
            add,
            remove,
        } => executers::update_fund_members(deps, env, info, fund_id, add, remove),
        ExecuteMsg::Deposit(msg) => executers::deposit(deps, env, info.clone(), info.sender, msg),
        ExecuteMsg::Recieve(msg) => executers::receive(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::config(deps)?),
        QueryMsg::State {} => to_binary(&queriers::state(deps)?),
        QueryMsg::TcaList {} => to_binary(&queriers::tca_list(deps)?),
        QueryMsg::FundsList {} => to_binary(&queriers::funds_list(deps)?),
        QueryMsg::FundDetails { fund_id } => to_binary(&queriers::fund_details(deps, fund_id)?),
        QueryMsg::InvolvedFunds { address } => to_binary(&queriers::involved_funds(deps, address)?),
        QueryMsg::ActiveFundDetails {} => to_binary(&queriers::active_fund_details(deps)?),
        QueryMsg::ActiveFundDonations {} => to_binary(&queriers::active_fund_donations(deps)?),
        QueryMsg::Deposit {
            amount,
            fund_id,
            split,
        } => to_binary(&queriers::deposit_msg_builder(
            deps, env, amount, fund_id, split,
        )?),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
