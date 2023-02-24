use crate::structs::IndexFund;
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct FundDetailsResponse {
    pub fund: Option<IndexFund>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct FundListResponse {
    pub funds: Vec<IndexFund>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_funds: u64,
    pub active_fund: u64,
    pub round_donations: Uint128,
    pub next_rotation_block: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
    pub fund_rotation: Option<u64>,
    pub fund_member_limit: u32,
    pub funding_goal: Option<Uint128>,
    pub alliance_members: Vec<Addr>,
}
