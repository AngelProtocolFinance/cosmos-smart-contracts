use crate::executers;
use crate::queriers;
use crate::state::{Config, State, CONFIG, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::index_fund::*;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfoBase};

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
    };
    CONFIG.save(deps.storage, &configs)?;

    // setup default state values
    STATE.save(
        deps.storage,
        &State {
            total_funds: 0,
            active_fund: 0,
            next_fund_id: 1,
            round_donations: Uint128::zero(),
            next_rotation_block: env.block.height + configs.fund_rotation.unwrap_or(0u64),
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
        ExecuteMsg::UpdateAllianceMemberList {
            address,
            member,
            action,
        } => executers::update_alliance_member_list(deps, info, address, member, action),
        ExecuteMsg::CreateFund {
            name,
            description,
            members,
            rotating_fund,
            split_to_liquid,
            expiry_time,
            expiry_height,
        } => executers::create_index_fund(
            deps,
            info,
            name,
            description,
            members,
            rotating_fund,
            split_to_liquid,
            expiry_time,
            expiry_height,
        ),
        ExecuteMsg::RemoveFund { fund_id } => {
            executers::remove_index_fund(deps, env, info, fund_id)
        }
        ExecuteMsg::RemoveMember(msg) => executers::remove_member(deps, info, msg.member),
        ExecuteMsg::UpdateMembers {
            fund_id,
            add,
            remove,
        } => executers::update_fund_members(deps, env, info, fund_id, add, remove),
        ExecuteMsg::Deposit(msg) => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::deposit(deps, env, info.clone(), info.sender, msg, native_fund)
        }
        ExecuteMsg::UpdateAllianceMember { address, member } => {
            executers::update_alliance_member(deps, env, info, address, member)
        }
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    let cw20_fund = Asset {
        info: AssetInfoBase::Cw20(info.sender.clone()),
        amount: cw20_msg.amount,
    };
    match from_binary(&cw20_msg.msg) {
        Ok(ReceiveMsg::Deposit(msg)) => executers::deposit(
            deps,
            env,
            info,
            api.addr_validate(&cw20_msg.sender)?,
            msg,
            cw20_fund,
        ),
        _ => Err(ContractError::InvalidInputs {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::config(deps)?),
        QueryMsg::State {} => to_binary(&queriers::state(deps)?),
        QueryMsg::FundsList { start_after, limit } => {
            to_binary(&queriers::funds_list(deps, start_after, limit)?)
        }
        QueryMsg::FundDetails { fund_id } => to_binary(&queriers::fund_details(deps, fund_id)?),
        QueryMsg::InvolvedFunds { address } => to_binary(&queriers::involved_funds(deps, address)?),
        QueryMsg::ActiveFundDetails {} => to_binary(&queriers::active_fund_details(deps)?),
        QueryMsg::ActiveFundDonations {} => to_binary(&queriers::active_fund_donations(deps)?),
        QueryMsg::Deposit {
            token_denom,
            amount,
            fund_id,
            split,
        } => to_binary(&queriers::deposit_msg_builder(
            deps,
            env,
            token_denom,
            amount,
            fund_id,
            split,
        )?),
        QueryMsg::AllianceMember { address } => {
            to_binary(&queriers::alliance_member(deps, address)?)
        }
        QueryMsg::AllianceMembers { start_after, limit } => {
            to_binary(&queriers::alliance_members(deps, start_after, limit)?)
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
