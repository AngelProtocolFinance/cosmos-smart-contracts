use crate::structs::Strategy;
use cw20::Cw20Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountListResponse {
    pub accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountDetailsResponse {
    pub account_type: String, // prefix ("locked" or "liquid")
    pub balance: Vec<Cw20Coin>,
    pub strategy: Strategy,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin_addr: String,
}
