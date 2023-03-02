use crate::common::OrderBy;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub halo_token: String, // halo token address
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        halo_token: Option<String>,
    },
    RegisterVestingAccounts {
        vesting_accounts: Vec<VestingAccount>,
    },
    AddSchedulesToVestingAccount {
        address: String,
        new_schedules: Vec<(u64, u64, Uint128)>,
    },
    UpdateVestingAccount {
        vesting_account: VestingAccount,
    },
    Claim {},
}

/// CONTRACT: end_time > start_time
#[cw_serde]
pub struct VestingAccount {
    pub address: String,
    pub schedules: Vec<(u64, u64, Uint128)>,
}

#[cw_serde]
pub struct VestingInfo {
    pub schedules: Vec<(u64, u64, Uint128)>,
    pub last_claim_time: u64,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    VestingAccount {
        address: String,
    },
    VestingAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub halo_token: String,
    pub genesis_time: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct VestingAccountResponse {
    pub address: String,
    pub info: VestingInfo,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct VestingAccountsResponse {
    pub vesting_accounts: Vec<VestingAccountResponse>,
}
