use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Decimal;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub gov_contract: String, // collected rewards receiver
    pub terraswap_factory: String,
    pub halo_token: String,
    pub distributor_contract: String,
    pub reward_factor: Decimal,
    pub treasury_addr: String, // recieves leftover rewards
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update the contract's owner
    UpdateOwner { owner: String },
    /// Update config interface
    /// to enable reward_factor / gov_contract update
    UpdateConfig {
        reward_factor: Option<Decimal>,
        gov_contract: Option<String>,
    },
    /// Public Message
    /// Sweep all given denom balance to ANC token
    /// and execute Distribute message
    Sweep { denom: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Pair { denom: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub gov_contract: String,  // collected rewards receiver
    pub treasury_addr: String, // recieves leftover rewards
    pub terraswap_factory: String,
    pub halo_token: String,
    pub reward_factor: Decimal,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
