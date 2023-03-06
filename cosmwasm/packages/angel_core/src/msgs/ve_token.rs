use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Withdraw {},
    Checkpoint {},
    IncreaseEndLockTime {
        // unlock_week specifies the week at which to unlock
        // in units of weeks since the epoch
        end_lock_time: u64,
    },
    RegisterContracts {
        cw20_address: String,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    /// StakeVotingTokens a user can stake their mirror token to receive rewards
    /// or do vote on polls
    CreateLock {
        // unlock_week specifies the week at which to unlock
        // in units of weeks since the epoch
        end_lock_time: u64,
    },
    IncreaseLockAmount {},
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State { timestamp: Option<u64> },
    #[returns(StakerResponse)]
    Staker {
        address: String,
        timestamp: Option<u64>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub cw20_address: String,
}

#[derive(Default)]
#[cw_serde]
pub struct StateResponse {
    pub total_deposited_amount: Uint128,
    pub total_locked_amount: Uint128,
    pub total_balance: Uint128,
}

#[derive(Default)]
#[cw_serde]
pub struct StakerResponse {
    pub deposited_amount: Uint128,
    pub locked_amount: Uint128,
    pub balance: Uint128,
}
