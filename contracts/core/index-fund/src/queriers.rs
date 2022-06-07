use crate::state::{
    read_alliance_members, read_funds, ALLIANCE_MEMBERS, CONFIG, FUND, STATE, TCA_DONATIONS,
};
use angel_core::messages::index_fund::ExecuteMsg::Deposit;
use angel_core::responses::index_fund::*;
use angel_core::{messages::index_fund::DepositMsg, structs::DEPOSIT_TOKEN_DENOM};
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, Env, StdError, StdResult, Uint128, WasmMsg,
};

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        fund_rotation: config.fund_rotation,
        fund_member_limit: config.fund_member_limit,
        funding_goal: config.funding_goal,
        accepted_tokens: config.accepted_tokens,
    })
}

pub fn state(deps: Deps) -> StdResult<StateResponse> {
    // return state values
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        total_funds: state.total_funds,
        active_fund: state.active_fund,
        round_donations: state.round_donations,
        next_rotation_block: state.next_rotation_block,
    })
}

pub fn funds_list(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u64>,
) -> StdResult<FundListResponse> {
    let funds = read_funds(deps.storage, start_after, limit)?;
    Ok(FundListResponse { funds })
}

pub fn fund_details(deps: Deps, fund_id: u64) -> StdResult<FundDetailsResponse> {
    Ok(FundDetailsResponse {
        fund: FUND.may_load(deps.storage, &fund_id.to_be_bytes())?,
    })
}

pub fn active_fund_details(deps: Deps) -> StdResult<FundDetailsResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(FundDetailsResponse {
        fund: FUND.may_load(deps.storage, &state.active_fund.to_be_bytes())?,
    })
}

pub fn active_fund_donations(deps: Deps) -> StdResult<DonationListResponse> {
    let mut donors = vec![];
    let alliance_addr_list: Vec<Addr> = ALLIANCE_MEMBERS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<_>>()?;
    let mut alliance_members: Vec<String> = vec![];
    for member in alliance_addr_list {
        alliance_members.push(member.to_string());
    }
    for member in alliance_members.into_iter() {
        // add to response vector
        donors.push(DonationDetailResponse {
            address: member.to_string(),
            total_ust: TCA_DONATIONS
                .may_load(deps.storage, member.to_string())
                .unwrap()
                .unwrap_or_default()
                .get_usd()
                .amount,
        });
    }
    Ok(DonationListResponse { donors })
}

pub fn involved_funds(deps: Deps, address: String) -> StdResult<FundListResponse> {
    let query_addr = deps.api.addr_validate(&address)?;
    let all_funds = read_funds(deps.storage, None, None)?;
    let mut involved_funds = vec![];
    for fund in all_funds.iter() {
        let pos = fund.members.iter().position(|m| *m == query_addr);
        if pos != None {
            involved_funds.push(fund.clone());
        }
    }
    Ok(FundListResponse {
        funds: involved_funds,
    })
}

pub fn deposit_msg_builder(
    _deps: Deps,
    env: Env,
    amount: Uint128,
    fund_id: Option<u64>,
    split: Option<Decimal>,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&Deposit(DepositMsg { fund_id, split }))?,
        funds: vec![Coin {
            denom: DEPOSIT_TOKEN_DENOM.to_string(),
            amount,
        }],
    }))
}

pub fn alliance_member(deps: Deps, wallet: Addr) -> StdResult<AllianceMemberResponse> {
    let alliance_member = match ALLIANCE_MEMBERS.may_load(deps.storage, wallet.clone()) {
        Ok(res) => match res {
            Some(m) => m,
            None => {
                return Err(StdError::GenericErr {
                    msg: "Cannot find member".to_string(),
                })
            }
        },
        Err(_) => {
            return Err(StdError::GenericErr {
                msg: "Cannot find member".to_string(),
            })
        }
    };

    Ok(AllianceMemberResponse {
        wallet: wallet.to_string(),
        name: alliance_member.name,
        logo: alliance_member.logo,
        website: alliance_member.website,
    })
}

pub fn alliance_members(
    deps: Deps,
    start_after: Option<Addr>,
    limit: Option<u64>,
) -> StdResult<AllianceMemberListResponse> {
    // return a list of angel alliance members
    let alliance_members = read_alliance_members(deps.storage, start_after, limit)?;
    Ok(AllianceMemberListResponse { alliance_members })
}
