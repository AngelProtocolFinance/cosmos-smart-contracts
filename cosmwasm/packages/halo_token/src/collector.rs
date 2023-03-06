use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;
use cw_asset::AssetInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub gov_contract: String, // collected rewards receiver
    pub swap_factory: String,
    pub halo_token: String,
    pub distributor_contract: String,
    pub reward_factor: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update config interface
    /// to enable reward_factor / gov_contract update
    UpdateConfig {
        reward_factor: Option<Decimal>,
        gov_contract: Option<String>,
        swap_factory: Option<String>,
    },
    /// Public Message
    /// Sweep all given denom balance to ANC token
    /// and execute Distribute message
    Sweep { denom: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AssetInfo)]
    Pair { denom: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub gov_contract: String, // collected rewards receiver
    pub swap_factory: String,
    pub halo_token: String,
    pub distributor_contract: String,
    pub reward_factor: Decimal,
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}
