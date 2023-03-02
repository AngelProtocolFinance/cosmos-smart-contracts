use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub gov_contract: String, // anchor gov contract
    pub halo_token: String,   // anchor token address
    pub spend_limit: Uint128, // spend limit per each `spend` request
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        spend_limit: Option<Uint128>,
        gov_contract: Option<String>,
    },
    Spend {
        recipient: String,
        amount: Uint128,
    },
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub gov_contract: String,
    pub halo_token: String,
    pub spend_limit: Uint128,
}
