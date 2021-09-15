use crate::state::{fund_read, read_funds, CONFIG, STATE, TCA_DONATIONS};
use angel_core::responses::index_fund::*;
use cosmwasm_std::{Deps, StdResult};
use cw2::get_contract_version;
use cw20::Cw20Coin;

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: get_contract_version(deps.storage)?.contract,
        registrar_contract: config.registrar_contract.to_string(),
        fund_rotation: config.fund_rotation,
        fund_member_limit: config.fund_member_limit,
        funding_goal: config.funding_goal.unwrap(),
        split_to_liquid: config.split_to_liquid,
    })
}

pub fn state(deps: Deps) -> StdResult<StateResponse> {
    // return state values
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        total_funds: state.total_funds,
        active_fund: state.active_fund,
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
        let donations = TCA_DONATIONS.may_load(deps.storage, tca.to_string())?;
        if donations != None {
            let cw20_bal: StdResult<Vec<_>> = donations
                .unwrap()
                .cw20
                .into_iter()
                .map(|token| {
                    Ok(Cw20Coin {
                        address: token.address.into(),
                        amount: token.amount,
                    })
                })
                .collect();
            // add to response vector
            donors.push(DonationDetailResponse {
                address: tca.to_string(),
                tokens: cw20_bal?,
            });
        }
    }
    Ok(DonationListResponse { donors })
}
