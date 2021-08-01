use crate::structs::{SplitDetails, Strategy};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub admin_addr: String,
    pub index_fund_contract: String,
    pub endowment_owner: String, // address that originally setup the endowment account
    pub endowment_beneficiary: String, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub deposit_approved: bool,        // DANO has approved to receive donations & transact
    pub withdraw_approved: bool,       // DANO has approved to withdraw funds
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub split_to_liquid: SplitDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt(DepositMsg),
    // Winding up of an endowment in good standing. Returns all funds to the Beneficiary.
    Liquidate {
        beneficiary: String, // Addr of the Beneficiary to receive funds
    },
    // Destroys the endowment and returns all Balance funds to an index fund and to the
    // Index Fund ID provided
    TerminateToFund {
        fund: String, // Index Fund ID to receive funds
    },
    // Destroys the endowment and returns all Balance funds to the beneficiary addr (DANO treasury)
    TerminateToAddress {
        beneficiary: String, // Addr of the Beneficiary to receive funds
    },
    // // Allows the contract parameter to be updated (only by the owner...for now)
    // UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateRegistrar {
        new_registrar: String,
    },
    UpdateConfig(UpdateConfigMsg),
    // Replace an Account's Strategy with that given.
    UpdateStrategy {
        account_type: String, // prefix ("locked" or "liquid")
        strategy: Strategy,
    },
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub beneficiary: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt(DepositMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub account_type: String, // prefix ("locked" or "liquid")
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details for a single Account, given an Account ID argument
    // Returns AccountDetailsResponse
    Account { account_type: String },
    // Get details on all Accounts. If passed, restrict to a given EID argument
    // Returns AccountListResponse
    AccountList {},
    // Get all Config details for the contract
    // Returns ConfigResponse
    Config {},
}
