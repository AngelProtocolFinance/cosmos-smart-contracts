use crate::errors::multisig::ContractError;
use cosmwasm_std::Decimal;
use cw_utils::{Duration, Threshold};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
    pub group_addr: String,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
}

/// Asserts that the 0.0 < percent <= 1.0
pub fn valid_percentage(percent: &Decimal) -> Result<(), ContractError> {
    if percent.is_zero() {
        Err(ContractError::ZeroThreshold {})
    } else if *percent > Decimal::one() {
        Err(ContractError::UnreachableThreshold {})
    } else {
        Ok(())
    }
}
