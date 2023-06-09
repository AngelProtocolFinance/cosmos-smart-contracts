use cosmwasm_std::Decimal;
use cw_asset::Asset;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub keeper: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    // Claim a deposited Asset to Addr Balance
    // for now, this is only doable by Keeper Addr
    Claim {
        deposit: u64,
        recipient: String,
    },
    // Spend token/amount specified from sender balance to Endowment
    Spend {
        asset: Asset,
        endow_id: u32,
        locked_percentage: Decimal,
        liquid_percentage: Decimal,
    },
    UpdateConfig {
        owner: Option<String>,
        keeper: Option<String>,
        registrar_contract: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    Balance { address: String },
    Config {},
    Deposit { deposit_id: u64 },
}
