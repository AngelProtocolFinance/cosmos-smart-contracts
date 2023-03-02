use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterContracts {
        dao_token: String,
        ve_token: String,
        terraswap_factory: String,
    },
    /// Public Message
    Claim {
        limit: Option<u32>,
    },
    DistributeDaoToken {},
    UpdateConfig {
        owner: Option<String>,
    },
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    State {},
    Staker {
        address: String,
        fee_limit: Option<u32>,
        fee_start_after: Option<u64>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub dao_token: String,
    pub ve_token: String,
    pub terraswap_factory: String,
}

#[cw_serde]
pub struct StateResponse {
    pub contract_addr: String,
    pub total_distributed_unclaimed_fees: Uint128,
}

#[cw_serde]
pub struct StakerResponse {
    pub balance: Uint128,
    pub initial_last_claimed_fee_timestamp: u64,
    pub last_claimed_fee_timestamp: u64,
    pub claimable_fees_lower_bound: Uint128,
}
