use crate::messages::dao_token::CurveType;
use crate::structs::{EndowmentFee, FundingSource, RebalanceDetails, StrategyComponent};
use crate::{messages::vault::AccountTransferMsg, structs::Profile};
use cosmwasm_std::Decimal;
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
    pub dao: bool,
    pub donation_match: bool,
    pub curve_type: Option<CurveType>,
    pub owner: String,       // address that originally setup the endowment account
    pub name: String,        // name of the Charity Endowment
    pub description: String, // description of the Charity Endowment
    pub whitelisted_beneficiaries: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub locked_endowment_configs: Vec<String>, // list of endowment configs that cannot be changed/altered once set at creation
    pub split_max: Decimal,
    pub split_min: Decimal,
    pub split_default: Decimal,
    pub beneficiary: String, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub profile: Profile,    // struct holding the Endowment info
    pub cw4_members: Vec<Member>,
    pub earnings_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
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
    // update owner addrInstantiateMsg
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
    pub owner: Option<String>,
    pub whitelisted_beneficiaries: Option<Vec<String>>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Option<Vec<String>>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub name: Option<String>,                          // name of the Charity Endowment
    pub description: Option<String>,                   // description of the Charity Endowment
    pub withdraw_before_maturity: Option<bool>, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<Option<u64>>,     // datetime int of endowment maturity
    pub maturity_height: Option<Option<u64>>,   // block equiv of the maturity_datetime
    pub strategies: Option<Vec<StrategyComponent>>, // list of vaults and percentage for locked/liquid accounts
    pub locked_endowment_configs: Option<Vec<String>>, // list of endowment configs that cannot be changed/altered once set at creation
    pub rebalance: Option<RebalanceDetails>,
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
    pub country_of_origin: Option<String>,
    pub street_address: Option<String>,
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
    // Get all "EndowmentFee"s
    GetEndowmentFees {},
}
