use angel_core::messages::guardians_multisig::Threshold;
use cosmwasm_std::{CosmosMsg, Empty};
use cw0::{Duration, Expiration};
use cw3::Vote;
use cw4::MemberChangedHookMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

// We can also add this as a cw3 extension
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_percentage() {
        // TODO: test the error messages

        // 0 is never a valid percentage
        let err = valid_percentage(&Decimal::zero()).unwrap_err();
        assert_eq!(err.to_string(), ContractError::ZeroThreshold {}.to_string());

        // 100% is
        valid_percentage(&Decimal::one()).unwrap();

        // 101% is not
        let err = valid_percentage(&Decimal::percent(101)).unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );
        // not 100.1%
        let err = valid_percentage(&Decimal::permille(1001)).unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );

        // other values in between 0 and 1 are valid
        valid_percentage(&Decimal::permille(1)).unwrap();
        valid_percentage(&Decimal::percent(17)).unwrap();
        valid_percentage(&Decimal::percent(99)).unwrap();
    }

    #[test]
    fn validate_threshold() {
        // absolute count ensures 0 < required <= total_weight
        let err = Threshold::AbsoluteCount { weight: 0 }
            .validate(5)
            .unwrap_err();
        // TODO: remove to_string() when PartialEq implemented
        assert_eq!(err.to_string(), ContractError::ZeroThreshold {}.to_string());
        let err = Threshold::AbsoluteCount { weight: 6 }
            .validate(5)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );

        Threshold::AbsoluteCount { weight: 1 }.validate(5).unwrap();
        Threshold::AbsoluteCount { weight: 5 }.validate(5).unwrap();

        // AbsolutePercentage just enforces valid_percentage (tested above)
        let err = Threshold::AbsolutePercentage {
            percentage: Decimal::zero(),
        }
        .validate(5)
        .unwrap_err();
        assert_eq!(err.to_string(), ContractError::ZeroThreshold {}.to_string());
        Threshold::AbsolutePercentage {
            percentage: Decimal::percent(51),
        }
        .validate(5)
        .unwrap();

        // Quorum enforces both valid just enforces valid_percentage (tested above)
        Threshold::ThresholdQuorum {
            threshold: Decimal::percent(51),
            quorum: Decimal::percent(40),
        }
        .validate(5)
        .unwrap();
        let err = Threshold::ThresholdQuorum {
            threshold: Decimal::percent(101),
            quorum: Decimal::percent(40),
        }
        .validate(5)
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );
        let err = Threshold::ThresholdQuorum {
            threshold: Decimal::percent(51),
            quorum: Decimal::percent(0),
        }
        .validate(5)
        .unwrap_err();
        assert_eq!(err.to_string(), ContractError::ZeroThreshold {}.to_string());
    }

    #[test]
    fn threshold_response() {
        let total_weight: u64 = 100;

        let res = Threshold::AbsoluteCount { weight: 42 }.to_response(total_weight);
        assert_eq!(
            res,
            ThresholdResponse::AbsoluteCount {
                weight: 42,
                total_weight
            }
        );

        let res = Threshold::AbsolutePercentage {
            percentage: Decimal::percent(51),
        }
        .to_response(total_weight);
        assert_eq!(
            res,
            ThresholdResponse::AbsolutePercentage {
                percentage: Decimal::percent(51),
                total_weight
            }
        );

        let res = Threshold::ThresholdQuorum {
            threshold: Decimal::percent(66),
            quorum: Decimal::percent(50),
        }
        .to_response(total_weight);
        assert_eq!(
            res,
            ThresholdResponse::ThresholdQuorum {
                threshold: Decimal::percent(66),
                quorum: Decimal::percent(50),
                total_weight
            }
        );
    }
}
