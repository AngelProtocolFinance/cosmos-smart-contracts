use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Empty};
use cw3::{Status, Vote};
use cw4::{Member, MemberChangedHookMsg};
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub group_addr: String,
    pub id: u32,
    pub cw4_members: Vec<Member>,
    pub cw4_code: u64,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
}

#[cw_serde]
pub struct EndowmentInstantiateMsg {
    pub id: u32,
    pub cw4_members: Vec<Member>,
    pub cw4_code: u64,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
    pub registrar_contract: String,
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

// TODO: add some T variants? Maybe good enough as fixed Empty for now
#[cw_serde]
pub enum ExecuteMsg {
    Propose {
        title: String,
        description: String,
        msgs: Vec<CosmosMsg<Empty>>,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
        latest: Option<Expiration>,
        meta: Option<String>,
    },
    Vote {
        proposal_id: u64,
        vote: Vote,
    },
    Execute {
        proposal_id: u64,
    },
    Close {
        proposal_id: u64,
    },
    UpdateConfig {
        threshold: Threshold,
        max_voting_period: Duration,
    },
    /// Handles update hook messages from the group contract
    MemberChangedHook(MemberChangedHookMsg),
}

#[cw_serde]
pub enum QueryMsg {
    /// Return ConfigResponse
    /// (mostly to expose CW4 address for easier updating members polls)
    Config {},
    /// Return ThresholdResponse
    Threshold {},
    /// Returns ProposalResponse
    Proposal { proposal_id: u64 },
    /// Returns ProposalListResponse
    ListProposals {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Returns ProposalListResponse
    ReverseProposals {
        start_before: Option<u64>,
        limit: Option<u32>,
    },
    /// Returns VoteResponse
    Vote { proposal_id: u64, voter: String },
    /// Returns VoteListResponse
    ListVotes {
        proposal_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns VoterInfo
    Voter { address: String },
    /// Returns VoterListResponse
    ListVoters {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub threshold: Threshold,
    pub max_voting_period: Duration,
    pub group_addr: String,
    pub require_execution: bool,
}

#[cw_serde]
pub struct MetaProposalResponse<T = Empty> {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub msgs: Vec<CosmosMsg<T>>,
    pub status: Status,
    pub expires: Expiration,
    /// This is the threshold that is applied to this proposal. Both the rules of the voting contract,
    /// as well as the total_weight of the voting group may have changed since this time. That means
    /// that the generic `Threshold{}` query does not provide valid information for existing proposals.
    pub threshold: ThresholdResponse,
    /// metadata field allows for a UI to easily set and display data about the proposal
    pub meta: Option<String>,
}

#[cw_serde]
pub struct MetaProposalListResponse {
    pub proposals: Vec<MetaProposalResponse>,
}
