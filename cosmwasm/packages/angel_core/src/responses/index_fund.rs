use crate::structs::IndexFund;
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct FundDetailsResponse {
    pub fund: Option<IndexFund>,
}

#[cw_serde]
pub struct FundListResponse {
    pub funds: Vec<IndexFund>,
}

#[cw_serde]
pub struct StateResponse {
    pub total_funds: u64,
    pub active_fund: u64,
    pub round_donations: Uint128,
    pub next_rotation_block: u64,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
    pub fund_rotation: Option<u64>,
    pub fund_member_limit: u32,
    pub funding_goal: Option<Uint128>,
    pub alliance_members: Vec<Addr>,
}
