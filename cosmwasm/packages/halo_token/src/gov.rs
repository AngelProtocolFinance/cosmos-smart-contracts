use crate::common::OrderBy;
#[allow(unused_imports)]
use crate::staking::StakerInfoResponse;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
#[allow(unused_imports)]
use cw_controllers::{Claim, ClaimsResponse};
use cw_utils::Duration;
use std::fmt;

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub quorum: u64,
    pub threshold: u64,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub registrar_contract: String,
    pub halo_token: String,    // halo token address
    pub unbonding_period: u64, // days of unbonding
    pub gov_hodler: String,    // contract to hold maturing claims
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ExecutePollMsgs {
        poll_id: u64,
    },
    RegisterContracts {
        halo_token: String,
    },
    UpdateConfig {
        owner: Option<String>,
        quorum: Option<u64>,
        threshold: Option<u64>,
        voting_period: Option<u64>,
        timelock_period: Option<u64>,
        proposal_deposit: Option<Uint128>,
        snapshot_period: Option<u64>,
        unbonding_period: Option<u64>,
        gov_hodler: Option<String>,
    },
    CastVote {
        poll_id: u64,
        vote: VoteOption,
        amount: Uint128,
    },
    WithdrawVotingTokens {
        amount: Option<Uint128>,
    },
    ClaimVotingTokens {},
    EndPoll {
        poll_id: u64,
    },
    ExecutePoll {
        poll_id: u64,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    /// StakeVotingTokens a user can stake their mirror token to receive rewards
    /// or do vote on polls
    StakeVotingTokens {},
    /// CreatePoll need to receive deposit from a proposer
    CreatePoll {
        title: String,
        description: String,
        link: Option<String>,
        proposal_type: Option<String>,
        options: Option<Vec<PollExecuteMsg>>,
    },
}

#[cw_serde]
pub struct PollExecuteMsg {
    pub order: u64,
    pub msg: Binary,
    pub funding_goal: Option<Uint128>,
    pub fund_rotation: Option<u64>,
    pub split_to_liquid: Option<Decimal>,
    pub treasury_tax_rate: Option<Decimal>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State {},
    /// Claims shows the number of tokens this address can access when they are done unbonding
    #[returns(ClaimsResponse)]
    Claims { address: String },
    #[returns(StakerInfoResponse)]
    Staker { address: String },
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
    pub halo_token: String,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub unbonding_period: Duration,
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
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub proposal_type: Option<String>,
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
pub struct StakerResponse {
    pub balance: Uint128,
    pub share: Uint128,
    pub locked_balance: Vec<(u64, VoterInfo)>,
    pub claims: Vec<Claim>,
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
    Expired, // Depricated
    Failed,
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
