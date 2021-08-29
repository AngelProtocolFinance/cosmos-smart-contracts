use crate::responses::vault::VaultBalanceResponse;
use crate::structs::{RebalanceDetails, SplitDetails, StrategyComponent};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountListResponse {
    pub locked_account: AccountDetailsResponse,
    pub liquid_account: AccountDetailsResponse,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountDetailsResponse {
    pub account_type: String, // prefix ("locked" or "liquid")
    pub ust_balance: Uint256,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountBalanceResponse {
    pub balances: Vec<VaultBalanceResponse>,
}
