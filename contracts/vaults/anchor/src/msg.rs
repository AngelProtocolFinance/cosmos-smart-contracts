use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub moneymarket: String,
    pub registrar_contract: String,
    pub tax_per_block: Decimal,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub harvest_to_liquid: Decimal,
    pub collector_contract: String, // recieves harvest proceeds
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
