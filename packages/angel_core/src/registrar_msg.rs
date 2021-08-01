use crate::structs::EndowmentStatus;
use cosmwasm_std::{Addr, Api, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub accounts_code_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Add new AssetVault to VAULTS
    VaultAdd { vault_addr: String },
    // Mark an AssetVault as approved (or not)
    VaultUpdateStatus { vault_addr: String, approved: bool },
    // Removes an AssetVault from VAULTS
    VaultRemove { vault_addr: String },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the DANO / AP Team to update the status of an Endowment
    // Approved, Frozen, (Liquidated, Terminated)
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateOwner { new_owner: String },
    CreateEndowment(CreateEndowmentMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub endowment_owner: String,
    pub endowment_beneficiary: String,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_code_id: Option<u64>,
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
pub struct UpdateEndowmentStatusMsg {
    pub address: String,
    pub status: EndowmentStatus,
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
    // Get all Config details for the contract
    // Returns ConfigResponse
    Config {},
}
