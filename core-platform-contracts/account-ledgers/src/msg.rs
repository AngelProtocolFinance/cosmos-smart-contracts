use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Coin;
use cw20::{Cw20Coin, Cw20ReceiveMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg { }

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub charity_endowment_sc: String, // Address of Charity Endowment SC
    pub index_fund_sc: String, // Address of Index Fund SC
    // All possible contracts that we can accept Cw20 tokens from
    pub cw20_approved_coins: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateAcct(CreateAcctMsg),
    // Destroys the endowment and returns all Balance funds to the beneficiary
    Terminate { account_id: String },
    // Adds all sent native tokens to the contract
    Deposit { account_id: String },
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
    Deposit { account_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateAcctMsg {
    pub account_id: String, // Endowment EID + some prefix serves the Account ID
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub charity_endowment_sc: Option<String>,
    pub index_fund_sc: Option<String>,
    pub cw20_approved_coins: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Show all open Accounts. Return type is ListResponse.
    // Accepts an optional argument of an originator account_id to filter by
    // List { account_id: Option<String> },
    // Returns the details of the named escrow, error if not created
    // Return type: DetailsResponse.
    Details { account_id: String },
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
    pub native_balance: Vec<Coin>,
    pub cw20_balance: Vec<Cw20Coin>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub charity_endowment_sc: String,
    pub index_fund_sc: String,
    pub cw20_approved_coins: Vec<String>,
}
