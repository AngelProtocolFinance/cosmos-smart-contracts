use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Coin;
use cw20::{Cw20Coin, Cw20ReceiveMsg};

use crate::state::{AssetVault, Strategy};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    // All possible contracts that we can accept Cw20 tokens from
    pub cw20_approved_coins: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateAcct(CreateAcctMsg),
    // Add new AssetVault to VAULTS
    VaultAdd {
        vault_addr: String,
        vault: AssetVault,
    },
    // Mark an AssetVault as approved (or not)
    VaultUpdateStatus {
        vault_addr: String,
        approved: bool,
    },
    // Removes an AssetVault from VAULTS
    VaultRemove {
        vault_addr: String,
    },
    // Winding up of an endowment in good standing. Returns all funds to the Beneficiary.
    Liquidate {
        eid: String,
    },
    // Destroys the endowment and returns all Balance funds to the parent index fund (if available)
    // and to the current active index fund if not.
    Terminate {
        eid: String,
    },
    // Adds all sent native tokens to the contract (sent from Asset Vaults)
    Deposit {
        account_id: String,
    },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateOwner {
        new_owner: String,
    },
    // Replace an Account's Strategy with that given.
    UpdateStrategy {
        account_id: String,
        strategy: Strategy,
    },
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    CreateAcct(CreateAcctMsg),
    // Add cw20 tokens sent for a specific account
    DepositSpecific { account_id: String },
    // Add cw20 tokens sent for a general endowment, not a specific account,
    // this general deposit optionally includes a split value
    DepositGeneral { eid: String, split: Option<u8> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateAcctMsg {
    pub eid: String, // Endowment EID serves as the base for prefixed account IDs
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub charity_endowment_sc: String,
    pub index_fund_sc: String,
    pub cw20_approved_coins: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details on a specific Vault
    // Returns VaultDetailsResponse
    Vault { address: String },
    // Gets list of all Vaults. Passing the optional non_approved arg to see all vaults, not just Approved
    // Returns VaultListResponse
    VaultList { non_approved: Option<bool> },
    // Get details for a single Account, given an Account ID argument
    // Returns AccountDetailsResponse
    Account { account_id: String },
    // Get details on all Accounts. If passed, restrict to a given EID argument
    // Returns AccountListResponse
    AccountList { eid: Option<String> },
    // Get all Config details for the contract
    // Returns ConfigResponse
    Config {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VaultDetailsResponse {
    pub address: String,
    pub name: String,
    pub description: String,
    pub approved: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VaultListResponse {
    pub vaults: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountDetailsResponse {
    pub account_id: String,
    pub native_balance: Vec<Coin>,
    pub cw20_balance: Vec<Cw20Coin>,
    pub strategy: Strategy,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountListResponse {
    pub accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub charity_endowment_sc: String,
    pub index_fund_sc: String,
    pub cw20_approved_coins: Vec<String>,
}
