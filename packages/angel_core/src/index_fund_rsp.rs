use crate::structs::{IndexFund, SplitDetails};
use cosmwasm_std::{Addr, Uint128};
use cw20::{Balance, Cw20Coin};
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
    pub tca_members: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DonationDetailResponse {
    pub address: String,
    pub tokens: Vec<Cw20Coin>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DonationListResponse {
    pub donors: Vec<DonationDetailResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_funds: u64,
    pub active_fund: Option<u64>,
    pub terra_alliance: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub fund_rotation_limit: Uint128, // blocks
    pub fund_member_limit: u32,
    pub funding_goal: Balance,
    pub split_to_liquid: SplitDetails,
}
