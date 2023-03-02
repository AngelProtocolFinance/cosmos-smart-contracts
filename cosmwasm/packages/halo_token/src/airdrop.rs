use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub halo_token: String, // halo token address
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
    },
    RegisterMerkleRoot {
        merkle_root: String,
    },
    Claim {
        stage: u8,
        amount: Uint128,
        proof: Vec<String>,
    },
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    MerkleRoot { stage: u8 },
    LatestStage {},
    IsClaimed { stage: u8, address: String },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub halo_token: String,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct MerkleRootResponse {
    pub stage: u8,
    pub merkle_root: String,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct LatestStageResponse {
    pub latest_stage: u8,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct IsClaimedResponse {
    pub is_claimed: bool,
}
