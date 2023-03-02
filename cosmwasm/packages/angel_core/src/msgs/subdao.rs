use crate::common::OrderBy;
use crate::structs::{DaoToken, EndowmentType};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use std::fmt;

#[cw_serde]
pub struct InstantiateMsg {
    pub id: u32,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub token: DaoToken,
    pub endow_type: EndowmentType,
    pub endow_owner: String,
    pub registrar_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    RegisterContracts {
        ve_token: String,
        swap_factory: String,
    },
    /// Public Message
    /// Sweep all given denom balance to GLOW token
    Sweep {
        denom: String,
    },
    UpdateConfig {
        owner: Option<String>,
        quorum: Option<Decimal>,
        threshold: Option<Decimal>,
        voting_period: Option<u64>,
        timelock_period: Option<u64>,
        expiration_period: Option<u64>,
        proposal_deposit: Option<Uint128>,
        snapshot_period: Option<u64>,
    },
    CastVote {
        poll_id: u64,
        vote: VoteOption,
    },
    EndPoll {
        poll_id: u64,
    },
    ExecutePoll {
        poll_id: u64,
    },
    ExpirePoll {
        poll_id: u64,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    /// CreatePoll need to receive deposit from a proposer
    CreatePoll {
        title: String,
        description: String,
        link: Option<String>,
        execute_msgs: Option<Vec<PollExecuteMsg>>,
    },
}

#[cw_serde]
pub struct PollExecuteMsg {
    pub order: u64,
    pub contract: String,
    pub msg: Binary,
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {
    pub ve_token: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State {},
    #[returns(PollResponse)]
    Poll { poll_id: u64 },
    #[returns(PollsResponse)]
    Polls {
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    #[returns(VotersResponse)]
    Voters {
        poll_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub dao_token: String,
    pub swap_factory: String,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[cw_serde]
pub struct StateResponse {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

#[cw_serde]
pub struct PollResponse {
    pub id: u64,
    pub creator: String,
    pub status: PollStatus,
    pub start_time: u64,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub deposit_amount: Uint128,
    pub execute_data: Option<Vec<PollExecuteMsg>>,
    pub yes_votes: Uint128, // balance
    pub no_votes: Uint128,  // balance
    pub staked_amount: Option<Uint128>,
    pub total_balance_at_end_poll: Option<Uint128>,
}

#[cw_serde]
pub struct PollsResponse {
    pub polls: Vec<PollResponse>,
}

#[cw_serde]
pub struct PollCountResponse {
    pub poll_count: u64,
}

#[cw_serde]
pub struct VotersResponseItem {
    pub voter: String,
    pub vote: VoteOption,
    pub balance: Uint128,
}

#[cw_serde]
pub struct VotersResponse {
    pub voters: Vec<VotersResponseItem>,
}

#[cw_serde]
pub struct VoterInfo {
    pub vote: VoteOption,
    pub balance: Uint128,
}

#[cw_serde]
pub enum PollStatus {
    InProgress,
    Passed,
    Rejected,
    Executed,
    Expired,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cw_serde]
pub enum VoteOption {
    Yes,
    No,
}

impl fmt::Display for VoteOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == VoteOption::Yes {
            write!(f, "yes")
        } else {
            write!(f, "no")
        }
    }
}
