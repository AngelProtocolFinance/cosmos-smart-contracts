use crate::structs::{IndexFund, SplitDetails};
use cosmwasm_std::Uint128;
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
pub struct DonationDetailResponse {
    pub address: String,
    pub tokens: Vec<Cw20Coin>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DonationListResponse {
    pub donors: Vec<DonationDetailResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub active_fund_index: String,
    pub fund_rotation_limit: Uint128, // blocks
    pub fund_member_limit: u32,
    pub funding_goal: Balance,
    pub split_to_liquid: SplitDetails,
}
