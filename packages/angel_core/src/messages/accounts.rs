use crate::structs::{AccountType, Beneficiary, Categories, Profile, SwapOperation};
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw4::Member;
use cw_asset::{Asset, AssetInfo};
use cw_utils::Threshold;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    /// reinvest vault assets from Liquid to Locked
    ReinvestToLocked {
        id: u32,
        amount: Uint128,
        vault_addr: String,
    },
    // Pull funds from Endowment locked/liquid free balances (TOH) to a Beneficiary address
    Withdraw {
        id: u32,
        acct_type: AccountType,
        beneficiary: String,
        assets: Vec<Asset>,
    },
    SwapToken {
        id: u32,
        acct_type: AccountType,
        amount: Uint128,
        operations: Vec<SwapOperation>,
    },
    // Router notifies the Accounts of final tokens from a Swap
    // Allows Accounts to credit the Endowment's involved Balance
    // with the amount returned to the main Accounts contract
    SwapReceipt {
        id: u32,
        acct_type: AccountType,
        final_asset: Asset,
    },
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt {
        id: u32,
        acct_type: AccountType,
    },
    // Invest TOH funds into Vaults
    VaultsInvest {
        id: u32,
        acct_type: AccountType,
        vaults: Vec<(String, Asset)>,
    },
    // Redeem TOH funds from Vaults
    VaultsRedeem {
        id: u32,
        acct_type: AccountType,
        vaults: Vec<(String, Uint128)>,
    },
    // set another endowment's strategy to "copycat" as your own
    CopycatStrategies {
        id: u32,
        acct_type: AccountType,
        id_to_copy: u32,
    },
    // create a new endowment
    CreateEndowment(CreateEndowmentMsg),
    // Winding up / closing of an endowment. Returns all funds to a specified Beneficiary address if provided.
    // If not provided, looks up the Index Fund an Endowment is tied to to donates the funds to it.
    CloseEndowment {
        id: u32,
        beneficiary: Beneficiary,
    },
    DistributeToBeneficiary {
        id: u32,
    },
    // update owner addr
    UpdateOwner {
        new_owner: String,
    },
    // Allows the SC owner (only!) to change ownership & upper limit of general categories ID allowed
    UpdateConfig {
        new_registrar: String,
        max_general_category_id: u8,
    },
    // Update an Endowment owner, beneficiary, and other settings
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    // Update an Endowment ability to receive/send funds
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Replace an Account's Strategy with that given.
    UpdateStrategies {
        id: u32,
        acct_type: AccountType,
        strategies: Vec<Strategy>,
    },
    // Update Endowment profile
    UpdateProfile(UpdateProfileMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub owner: String, // address that originally setup the endowment account
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub profile: Profile,               // struct holding the Endowment info
    pub cw4_members: Vec<Member>,
    pub kyc_donors_only: bool,
    pub cw3_threshold: Threshold,
    pub cw3_max_voting_period: u64,
    pub proposal_link: Option<u64>, // link back to the proposal that created an Endowment (set @ init)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentStatusMsg {
    pub endowment_id: u32,
    pub status: u8,
    pub beneficiary: Option<Beneficiary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Strategy {
    pub vault: String,       // Vault SC Address
    pub percentage: Decimal, // percentage of funds to invest
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentSettingsMsg {
    pub id: u32,
    pub owner: String,
    pub kyc_donors_only: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from a Vault
    VaultReceipt { id: u32, acct_type: AccountType },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub id: u32,
    pub locked_percentage: Decimal,
    pub liquid_percentage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateProfileMsg {
    pub id: u32,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub categories: Option<Categories>,
    pub tier: Option<u8>,
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
    pub number_of_employees: Option<u16>,
    pub average_annual_budget: Option<String>,
    pub annual_revenue: Option<String>,
    pub charity_navigator_rating: Option<String>,
    pub endow_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get all Config details for the contract
    Config {},
    // Get the balance of available UST and the invested portion balances
    Balance {
        id: u32,
    },
    // Get state details (like total donations received so far)
    State {
        id: u32,
    },
    // Get all Endowment details
    Endowment {
        id: u32,
    },
    // Gets list of all registered Endowments
    EndowmentList {
        status: Option<String>,
        name: Option<Option<String>>,
        owner: Option<String>,
        tier: Option<Option<String>>,
        endow_type: Option<String>,
        proposal_link: Option<u64>,
    },
    // Get the profile info
    GetProfile {
        id: u32,
    },
    // Get endowment token balance
    TokenAmount {
        id: u32,
        asset_info: AssetInfo,
        acct_type: AccountType,
    },
}
