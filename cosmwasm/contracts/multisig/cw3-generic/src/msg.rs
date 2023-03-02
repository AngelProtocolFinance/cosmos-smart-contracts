use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{CosmosMsg, Empty};
use cw3::Vote;
use cw4::MemberChangedHookMsg;
use cw_utils::{Duration, Expiration, Threshold};

/// We currently take no arguments for migrations

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub group_addr: String,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
}

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
        require_execution: bool,
        threshold: Threshold,
        max_voting_period: Duration,
    },
    /// Handles update hook messages from the group contract
    MemberChangedHook(MemberChangedHookMsg),
}
