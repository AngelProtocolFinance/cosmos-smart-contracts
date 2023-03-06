use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub gov_contract: String,   // halo gov contract
    pub halo_token: String,     // halo token address
    pub whitelist: Vec<String>, // whitelisted contract addresses to spend distributor
    pub spend_limit: Uint128,   // spend limit per each `spend` request
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
    AddDistributor {
        distributor: String,
    },
    RemoveDistributor {
        distributor: String,
    },
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub gov_contract: String,
    pub halo_token: String,
    pub whitelist: Vec<String>,
    pub spend_limit: Uint128,
}
