use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Api, StdResult};
use cw20::{Cw20Coin, Cw20ReceiveMsg};

use crate::state::{AssetVault, Strategy};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateAcct(CreateAcctMsg),
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt(DepositMsg),
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
        eid: String,         // EID
        beneficiary: String, // Addr of the Beneficiary to receive funds
    },
    // Destroys the endowment and returns all Balance funds to an index fund and to the
    // Index Fund ID provided
    TerminateToFund {
        eid: String,  // EID
        fund: String, // Index Fund ID to receive funds
    },
    // Destroys the endowment and returns all Balance funds to the beneficiary addr (DANO treasury)
    TerminateToAddress {
        eid: String,         // EID
        beneficiary: String, // Addr of the Beneficiary to receive funds
    },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateOwner {
        new_owner: String,
    },
    // Replace an Account's Strategy with that given.
    UpdateStrategy {
        eid: String,          // EID
        account_type: String, // prefix ("locked" or "liquid")
        strategy: Strategy,
    },
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
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
    pub eid: String,          // EID
    pub account_type: String, // prefix ("locked" or "liquid")
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateAcctMsg {
    pub eid: String, // Endowment EID serves as the base for prefixed account IDs
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub charity_endowment_contract: String,
    pub index_fund_contract: String,
    pub approved_coins: Option<Vec<String>>,
}

impl UpdateConfigMsg {
    pub fn addr_approved_list(&self, api: &dyn Api) -> StdResult<Vec<Addr>> {
        match self.approved_coins.as_ref() {
            Some(v) => v.iter().map(|h| api.addr_validate(h)).collect(),
            None => Ok(vec![]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details on a specific Vault
    // Returns VaultDetailsResponse
    Vault {
        address: String,
    },
    // Gets list of all Vaults. Passing the optional non_approved arg to see all vaults, not just Approved
    // Returns VaultListResponse
    VaultList {
        non_approved: Option<bool>,
    },
    // Get details for a single Account, given an Account ID argument
    // Returns AccountDetailsResponse
    Account {
        eid: String,          // EID
        account_type: String, // prefix ("locked" or "liquid")
    },
    // Get details on all Accounts. If passed, restrict to a given EID argument
    // Returns AccountListResponse
    AccountList {
        eid: Option<String>,
    },
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
    pub eid: String,          // EID
    pub account_type: String, // prefix ("locked" or "liquid")
    pub balance: Vec<Cw20Coin>,
    pub strategy: Strategy,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AccountListResponse {
    pub accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub charity_endowment_contract: String,
    pub index_fund_contract: String,
    pub approved_coins: Vec<String>,
}
