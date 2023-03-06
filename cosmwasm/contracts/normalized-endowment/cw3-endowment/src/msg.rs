use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{CosmosMsg, Empty};
use cw3::{Status, Vote};
use cw4::MemberChangedHookMsg;
use cw_asset::AssetUnchecked;
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};


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
    ProposeLockedWithdraw {
        endowment_id: u32,
        description: String,
        beneficiary_wallet: Option<String>,
        beneficiary_endow: Option<u32>,
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
    },
    /// Handles update hook messages from the group contract
    MemberChangedHook(MemberChangedHookMsg),
}

#[cw_serde]
pub struct ConfigResponse {
    pub require_execution: bool,
    pub registrar_contract: String,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
    pub group_addr: String,
}

#[cw_serde]
pub struct MetaProposalResponse<T = Empty> {
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

#[cw_serde]
pub struct MetaProposalListResponse {
    pub proposals: Vec<MetaProposalResponse>,
}
