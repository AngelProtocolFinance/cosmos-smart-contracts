use crate::messages::vault::AccountTransferMsg;
use crate::structs::FundingSource;
use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub name: String,
    pub overview: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
    pub owner: String,       // address that originally setup the endowment account
    pub beneficiary: String, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub name: String,        // name of the Charity Endowment
    pub description: String, // description of the Charity Endowment
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Pull funds from investment vault(s) to the Endowment Beneficiary as UST
    Withdraw {
        sources: Vec<FundingSource>,
        beneficiary: String,
    },
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt(AccountTransferMsg),
    // Winding up / closing of an endowment. Returns all funds to a specified Beneficiary address if provided.
    // If not provided, looks up the Index Fund an Endowment is tied to to donates the funds to it.
    CloseEndowment {
        beneficiary: Option<String>, // Optional Addr of the Beneficiary to receive funds
    },
    // update owner addr
    UpdateOwner {
        new_owner: String,
    },
    // Allows the SC owner (only!) to change ownership
    UpdateRegistrar {
        new_registrar: String,
    },
    UpdateConfig(UpdateConfigMsg),
    // Update an Endowment owner, beneficiary, and other settings
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    // Update an Endowment ability to receive/send funds
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Replace an Account's Strategy with that given.
    UpdateStrategies {
        strategies: Vec<Strategy>,
    },
    UpdateGuardians {
        add: Vec<String>,
        remove: Vec<String>,
    },
    // Update Endowment profile
    UpdateProfile(UpdateProfileMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accepted_tokens_native: Vec<String>,
    pub accepted_tokens_cw20: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Strategy {
    pub vault: String,              // Vault SC Address
    pub locked_percentage: Decimal, // percentage of funds to invest
    pub liquid_percentage: Decimal, // percentage of funds to invest
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentSettingsMsg {
    pub beneficiary: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentStatusMsg {
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum ReceiveMsg {
//     // Add tokens sent for a specific account
//     Deposit(DepositMsg),
//     // Tokens are sent back to an Account from a Vault
//     VaultReceipt(AccountTransferMsg),
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub locked_percentage: Decimal,
    pub liquid_percentage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RedeemMsg {
    pub sources: Vec<FundingSource>,
    // pub reinvest: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawMsg {
    pub sources: Vec<FundingSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateProfileMsg {
    pub name: Option<String>,
    pub overview: Option<String>,
    pub un_sdg: Option<u64>,
    pub tier: Option<u64>,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub url: Option<String>,
    pub registration_number: Option<String>,
    pub country_city_origin: Option<String>,
    pub contact_email: Option<String>,
    pub facebook: Option<String>,
    pub twitter: Option<String>,
    pub linkedin: Option<String>,
    pub number_of_employees: Option<u64>,
    pub average_annual_budget: Option<String>,
    pub annual_revenue: Option<String>,
    pub charity_navigator_rating: Option<String>,
    pub endow_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get the balance of available UST and the invested portion balances
    Balance {},
    // Get state details (like total donations received so far)
    State {},
    // Get all Config details for the contract
    // Returns ConfigResponse
    Config {},
    // Get all Endowment details
    Endowment {},
    // Get the profile info
    GetProfile {},
    // Get the transaction records
    GetTxRecords {
        sender: Option<String>,
        recipient: Option<String>,
        denom: Option<String>,
    },
}
