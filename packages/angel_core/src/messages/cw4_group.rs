use cw4::Member;
use cw_utils::{Duration, Threshold};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// The admin is the only account that can update the group state.
    /// Omit it to make the group immutable.
    pub admin: Option<String>,
    pub members: Vec<Member>,
    pub cw3_code: u64,
    pub cw3_threshold: Threshold,
    pub cw3_max_voting_period: Duration,
}
