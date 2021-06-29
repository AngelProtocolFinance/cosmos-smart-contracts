use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, StdError, StdResult};
use cw20::{Cw20Coin, Cw20ReceiveMsg};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub payout_rate: u32,                         // blocks per payout cycle
    pub mgmnt_fee: u32,                           // AUM fee taken
    pub cw20_approved_coins: Option<Vec<String>>, // All possible contracts that we can accept Cw20 tokens from
}

impl InstantiateMsg {
    pub fn validate(&self) -> StdResult<()> {
        // Check expires, payout_rate, mgmnt_fee
        if Some(&self.payout_rate) == None {
            return Err(StdError::generic_err(
                "Payout Rate (blocks) and Expires configs must be given.",
            ));
        }
        if self.mgmnt_fee > 100 {
            return Err(StdError::generic_err(
                "Management Fee must not exceed 100(%).",
            ));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateAcct(CreateAcctMsg),
    // Approve allows an Endowment to start acepting funds and sets up a Liquid Account
    // Only the arbiter can perform this action
    Approve { address: String },
    // Destroys the endowment and returns all Balance funds to the beneficiary
    Terminate { address: String },
    // Adds all sent native tokens to the contract
    Deposit { address: String },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateOwner { new_owner: String },
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    CreateAcct(CreateAcctMsg),
    // Adds all sent native tokens to the contract
    Deposit { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateAcctMsg {
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
pub struct UpdateConfigMsg {
    pub payout_rate: Option<u32>,
    pub mgmnt_fee: Option<u32>,
    pub cw20_approved_coins: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Show all open Accounts. Return type is ListResponse.
    // Accepts an optional argument of an originator address to filter by
    // List { address: Option<String> },
    // Returns the details of the named escrow, error if not created
    // Return type: DetailsResponse.
    Details { address: String },
    // Get all Config details for the contract
    Config {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListResponse {
    // list all registered accounts
    pub accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DetailsResponse {
    pub arbiter: Addr,
    pub beneficiary: Addr,
    pub owner: Addr,
    pub approved: bool,
    // When end height set and block height exceeds this value, the escrow is expired.
    // Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_height: Option<u64>,
    // When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    // block time exceeds this value, the escrow is expired.
    // Once an escrow is expired, it can be returned to the original funder (via "refund").
    pub end_time: Option<u64>,
    pub native_balance: Vec<Coin>,
    pub cw20_balance: Vec<Cw20Coin>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub payout_rate: u32,
    pub mgmnt_fee: u32,
    pub cw20_approved_coins: Vec<String>,
}
