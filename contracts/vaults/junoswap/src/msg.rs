use cosmwasm_std::Decimal;
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub swap_pool_addr: String,
    pub staking_addr: String,
    pub output_token_denom: Denom,
    pub registrar_contract: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
