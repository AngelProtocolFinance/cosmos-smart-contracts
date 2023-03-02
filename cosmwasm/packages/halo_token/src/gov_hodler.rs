use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub gov_contract: String,
    pub halo_token: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update config interface (gov_contract update)
    UpdateConfig { gov_contract: String },
    /// Gov Contract can request some amount of HALO to be released
    /// from the Hodler's death grip and return a claim to a user
    ClaimHalo { recipient: String, amount: Uint128 },
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}
