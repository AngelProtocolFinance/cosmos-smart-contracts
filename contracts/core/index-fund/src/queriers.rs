use crate::state::{read_funds, CONFIG, FUND, STATE};
use angel_core::messages::index_fund::{DepositMsg, ExecuteMsg::Deposit};
use angel_core::responses::index_fund::*;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Decimal, Deps, Env, StdResult, Uint128, WasmMsg};
use cw2::get_contract_version;

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: format!(
            "{}-{}",
            get_contract_version(deps.storage)?.contract,
            get_contract_version(deps.storage)?.version
        ),
        registrar_contract: config.registrar_contract.to_string(),
        fund_rotation: config.fund_rotation,
        fund_member_limit: config.fund_member_limit,
        funding_goal: config.funding_goal,
        alliance_members: config.alliance_members,
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

pub fn involved_funds(deps: Deps, endowment_id: u32) -> StdResult<FundListResponse> {
    let mut involved_funds = vec![];
    let all_funds = read_funds(deps.storage, None, None)?;
    for fund in all_funds.iter() {
        let pos = fund.members.iter().position(|m| *m == endowment_id);
        if pos.is_some() {
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
    token_denom: String,
    amount: Uint128,
    fund_id: Option<u64>,
    split: Option<Decimal>,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&Deposit(DepositMsg { fund_id, split }))?,
        funds: vec![Coin {
            denom: token_denom,
            amount,
        }],
    }))
}
