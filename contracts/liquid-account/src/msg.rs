use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
  // Allows the contract parameter to be updated
  UpdateConfig(UpdateConfigMsg),

  CreateAccount(CreateAccountMsg),

  // Approve allows an Endowment to start accepting funds
  // Only the arbiter can perform this action
  Approve { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
  pub owner: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateAccountMsg {
    // arbiter can decide to approve or refund the escrow
    pub arbiter: String,

    // if approved, funds go to the beneficiary
    pub beneficiary: String,

    // When end height set and block height exceeds this value, the escrow is expired.
    // Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_height: Option<u64>,

    // When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    // block time exceeds this value, the escrow is expired.
    // Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_time: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
}