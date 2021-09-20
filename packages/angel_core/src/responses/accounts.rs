use crate::structs::{RebalanceDetails, SplitDetails, StrategyComponent};
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StateResponse {
    pub donations_received: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailsResponse {
    pub owner: Addr,
    pub beneficiary: Addr,
    pub name: String,
    pub description: String,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub split_to_liquid: SplitDetails,
    pub strategies: Vec<StrategyComponent>,
    pub rebalance: RebalanceDetails,
}
