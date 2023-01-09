use cosmwasm_std::{CosmosMsg, Empty};
use cw3::{Status, Vote};
use cw4::MemberChangedHookMsg;
use cw_asset::AssetUnchecked;
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

// TODO: add some T variants? Maybe good enough as fixed Empty for now
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Propose {
        title: String,
        description: String,
        msgs: Vec<CosmosMsg<Empty>>,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
        latest: Option<Expiration>,
        meta: Option<String>,
    },
    ProposeLockedWithdraw {
        endowment_id: u32,
        description: String,
        beneficiary: String,
        assets: Vec<AssetUnchecked>,
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
        require_execution: Option<bool>,
        guardians: Option<Vec<String>>,
    },
    /// Handles update hook messages from the group contract
    MemberChangedHook(MemberChangedHookMsg),
    // Guardian messages
    GuardianPropose {
        title: String,
        description: String,
        old_member: String,
        new_member: String,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
        latest: Option<Expiration>,
        meta: Option<String>,
    },
    GuardianVote {
        proposal_id: u64,
        vote: Vote,
    },
    GuardianExecute {
        proposal_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub require_execution: bool,
    pub registrar_contract: String,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
    pub group_addr: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MetaProposalResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub id: u64,
    pub title: String,
    pub description: String,
    pub msgs: Vec<CosmosMsg<T>>,
    pub status: Status,
    pub expires: Expiration,
    pub confirmation_proposal: Option<u64>,
    /// This is the threshold that is applied to this proposal. Both the rules of the voting contract,
    /// as well as the total_weight of the voting group may have changed since this time. That means
    /// that the generic `Threshold{}` query does not provide valid information for existing proposals.
    pub threshold: ThresholdResponse,
    /// metadata field allows for a UI to easily set and display data about the proposal
    pub meta: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MetaProposalListResponse {
    pub proposals: Vec<MetaProposalResponse>,
}
