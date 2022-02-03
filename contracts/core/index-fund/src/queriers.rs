use crate::state::{fund_read, read_funds, CONFIG, STATE, TCA_DONATIONS};
use angel_core::messages::index_fund::DepositMsg;
use angel_core::messages::index_fund::ExecuteMsg::Deposit;
use angel_core::responses::index_fund::*;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Decimal, Deps, Env, StdResult, Uint128, WasmMsg};

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
        terra_alliance: state.tca_human_addresses(),
    })
}

pub fn tca_list(deps: Deps) -> StdResult<TcaListResponse> {
    // Return a list of TCA Member Addrs
    let state = STATE.load(deps.storage)?;
    Ok(TcaListResponse {
        tca_members: state.tca_human_addresses(),
    })
}

pub fn funds_list(deps: Deps) -> StdResult<FundListResponse> {
    // Return a list of Index Funds
    let funds = read_funds(deps.storage)?;
    Ok(FundListResponse { funds })
}

pub fn fund_details(deps: Deps, fund_id: u64) -> StdResult<FundDetailsResponse> {
    Ok(FundDetailsResponse {
        fund: fund_read(deps.storage).may_load(&fund_id.to_be_bytes())?,
    })
}

pub fn active_fund_details(deps: Deps) -> StdResult<FundDetailsResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(FundDetailsResponse {
        fund: fund_read(deps.storage).may_load(&state.active_fund.to_be_bytes())?,
    })
}

pub fn active_fund_donations(deps: Deps) -> StdResult<DonationListResponse> {
    let state = STATE.load(deps.storage)?;
    let mut donors = vec![];
    for tca in state.terra_alliance.into_iter() {
        // add to response vector
        donors.push(DonationDetailResponse {
            address: tca.to_string(),
            total_ust: TCA_DONATIONS
                .may_load(deps.storage, tca.to_string())
                .unwrap()
                .unwrap_or_default()
                .get_ust()
                .amount,
        });
    }
    Ok(DonationListResponse { donors })
}

pub fn involved_funds(deps: Deps, address: String) -> StdResult<FundListResponse> {
    let query_addr = deps.api.addr_validate(&address)?;
    let all_funds = read_funds(deps.storage)?;
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
            denom: "uusd".to_string(),
            amount,
        }],
    }))
}
