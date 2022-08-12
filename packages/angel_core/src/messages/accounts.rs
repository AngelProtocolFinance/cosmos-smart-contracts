use crate::structs::{
    DaoSetup, DonationMatch, EndowmentFee, FundingSource, GenericBalance, Profile,
    RebalanceDetails, SettingsController, StrategyComponent,
};
use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use cw4::Member;
use cw_utils::{Duration, Threshold};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub last_earnings_harvest: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Pull funds from investment vault(s) to the Endowment Beneficiary as <asset_info>
    // NOTE: Atm, the "vault" logic is not fixed.
    //       Hence, it SHOULD be updated when the "vault" logic is implemented.
    Withdraw {
        id: String,
        beneficiary: String,
        sources: Vec<FundingSource>,
    },
    WithdrawLiquid {
        id: String,
        beneficiary: String,
        assets: GenericBalance,
    },
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt {
        id: String,
    },
    // create a new endowment
    CreateEndowment(CreateEndowmentMsg),
    // Winding up / closing of an endowment. Returns all funds to a specified Beneficiary address if provided.
    // If not provided, looks up the Index Fund an Endowment is tied to to donates the funds to it.
    CloseEndowment {
        id: String,
        beneficiary: Option<String>, // Optional Addr of the Beneficiary to receive funds
    },
    // update owner addrInstantiateMsg
    UpdateOwner {
        new_owner: String,
    },
    // update config
    // UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateRegistrar {
        new_registrar: String,
    },
    // Update an Endowment owner, beneficiary, and other settings
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    // Update an Endowment ability to receive/send funds
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Replace an Account's Strategy with that given.
    UpdateStrategies {
        id: String,
        strategies: Vec<Strategy>,
    },
    // Update Endowment profile
    UpdateProfile(UpdateProfileMsg),
    // Update various "EndowmentFee"s
    UpdateEndowmentFees(UpdateEndowmentFeesMsg),
    // AUM harvest
    HarvestAum {
        id: String,
    },
    // Set up dao token for "Endowment"
    SetupDao(DaoSetup),
    // Setup Donation match contract for the Endowment
    SetupDonationMatch {
        id: String,
        setup: DonationMatch,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub id: String,    // human-readable ID for the endowment (allows for nicer URL slugs!)
    pub owner: String, // address that originally setup the endowment account
    pub whitelisted_beneficiaries: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_whitelist: Vec<String>,
    pub maturity_time: Option<u64>, // datetime int of endowment maturity(unit: seconds)
    pub split_max: Decimal,
    pub split_min: Decimal,
    pub split_default: Decimal,
    pub profile: Profile, // struct holding the Endowment info
    pub cw4_members: Vec<Member>,
    pub earnings_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
    pub dao: Option<DaoSetup>, // SubDAO setup options
    pub settings_controller: Option<SettingsController>,
    pub parent: Option<String>,
    pub kyc_donors_only: bool,
    pub cw3_threshold: Threshold,
    pub cw3_max_voting_period: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Strategy {
    pub vault: String,       // Vault SC Address
    pub percentage: Decimal, // percentage of funds to invest
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateMaturityWhitelist {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentSettingsMsg {
    pub id: String,
    pub owner: Option<String>,
    pub whitelisted_beneficiaries: Option<Vec<String>>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Option<Vec<String>>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub withdraw_before_maturity: Option<bool>, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<Option<u64>>,     // datetime int of endowment maturity
    pub strategies: Option<Vec<StrategyComponent>>, // list of vaults and percentage for locked/liquid accounts
    pub locked_endowment_configs: Option<Vec<String>>, // list of endowment configs that cannot be changed/altered once set at creation
    pub rebalance: Option<RebalanceDetails>,
    pub kyc_donors_only: bool,
    pub maturity_whitelist: Option<UpdateMaturityWhitelist>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentStatusMsg {
    pub id: String,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from a Vault
    VaultReceipt { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub id: String,
    pub locked_percentage: Decimal,
    pub liquid_percentage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RedeemMsg {
    pub id: String,
    pub sources: Vec<FundingSource>,
    // pub reinvest: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawMsg {
    pub id: String,
    pub sources: Vec<FundingSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateProfileMsg {
    pub id: String,
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
pub struct UpdateEndowmentFeesMsg {
    pub id: String,
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get all Config details for the contract
    Config {},
    // Get a list of all endowment IDs
    GetAllIds {},
    // Get the balance of available UST and the invested portion balances
    Balance { id: String },
    // Get state details (like total donations received so far)
    State { id: String },
    // Get all Endowment details
    Endowment { id: String },
    // Get all "EndowmentFee"s
    GetEndowmentFees { id: String },
    // Get the profile info
    GetProfile { id: String },
}
