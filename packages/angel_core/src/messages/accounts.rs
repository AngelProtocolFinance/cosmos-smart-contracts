use crate::messages::portal::AccountTransferMsg;
use crate::structs::{SplitDetails, StrategyComponent};
use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub admin_addr: String,
    pub registrar_contract: String,
    pub index_fund_contract: String,
    pub owner: String,       // address that originally setup the endowment account
    pub beneficiary: String, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub name: String,        // name of the Charity Endowment
    pub description: String, // description of the Charity Endowment
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
    PortalReceipt(AccountTransferMsg),
    // Winding up of an endowment in good standing. Returns all funds to the Beneficiary.
    Liquidate {
        beneficiary: String, // Addr of the Beneficiary to receive funds
    },
    // Destroys the endowment and returns all Balance funds to an index fund and to the
    // Index Fund ID provided
    TerminateToFund {
        fund: u64, // Index Fund ID to receive funds
    },
    // Destroys the endowment and returns all Balance funds to the beneficiary addr (DANO treasury)
    TerminateToAddress {
        beneficiary: String, // Addr of the Beneficiary to receive funds
    },
    // update admin addr
    UpdateAdmin {
        new_admin: String,
    },
    // Allows the SC owner (only!) to change ownership
    UpdateRegistrar {
        new_registrar: String,
    },
    // Update an Endowment owner, beneficiary, and other settings
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    // Update an Endowment ability to receive/send funds
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Replace an Account's Strategy with that given.
    UpdateStrategy {
        strategies: Vec<StrategyComponent>,
    },
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentSettingsMsg {
    pub beneficiary: String,
    pub owner: String,
    pub split_to_liquid: SplitDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentStatusMsg {
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from a Portal
    PortalReceipt(AccountTransferMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub locked_percentage: Decimal,
    pub liquid_percentage: Decimal,
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
    // Get all Endowment details
    Endowment {},
}
