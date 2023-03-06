use crate::state::ProposalType;
use angel_core::msgs::accounts::CreateEndowmentMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, CosmosMsg, Decimal, Empty};
use cw3::{Status, Vote};
use cw4::MemberChangedHookMsg;
use cw_asset::Asset;
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub registrar_contract: String,
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
    ProposeApplication {
        ref_id: String,
        msg: CreateEndowmentMsg,
        // note: we ignore API-spec'd earliest if passed, always opens immediately
        latest: Option<Expiration>,
        meta: Option<String>,
    },
    Vote {
        proposal_id: u64,
        vote: Vote,
    },
    VoteApplication {
        proposal_id: u64,
        vote: Vote,
        reason: Option<String>,
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
        require_execution: bool,
        seed_asset: Option<Asset>,
        seed_split_to_liquid: Decimal,
        new_endow_gas_money: Option<Coin>,
    },
    /// Handles update hook messages from the group contract
    MemberChangedHook(MemberChangedHookMsg),
}

#[cw_serde]
pub struct ConfigResponse {
    pub registrar_contract: String,
    pub version: String,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
    pub group_addr: String,
    pub require_execution: bool,
    pub seed_asset: Option<Asset>,
    pub seed_split_to_liquid: Decimal,
    pub new_endow_gas_money: Option<Coin>,
}

#[cw_serde]
pub struct MetaApplicationsProposalResponse<T = Empty> {
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
    pub proposal_type: ProposalType,
    /// metadata field allows for a UI to easily set and display data about the proposal
    pub meta: Option<String>,
}

#[cw_serde]
pub struct MetaApplicationsProposalListResponse {
    pub proposals: Vec<MetaApplicationsProposalResponse>,
}
