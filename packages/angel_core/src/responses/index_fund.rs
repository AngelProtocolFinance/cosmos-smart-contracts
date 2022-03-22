use crate::structs::{AcceptedTokens, IndexFund};
use cosmwasm_std::Uint128;
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TcaListResponse {
    pub tca_members: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DonationDetailResponse {
    pub address: String,
    pub total_ust: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DonationListResponse {
    pub donors: Vec<DonationDetailResponse>,
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
    pub registrar_contract: String,
    pub fund_rotation: Option<u64>,
    pub fund_member_limit: u32,
    pub funding_goal: Option<Uint128>,
    pub accepted_tokens: AcceptedTokens,
}
