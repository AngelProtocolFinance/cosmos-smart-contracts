use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub moneymarket: String,
    pub registrar_contract: String,
    pub tax_per_block: Decimal,
    pub treasury_withdraw_threshold: Uint128,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
