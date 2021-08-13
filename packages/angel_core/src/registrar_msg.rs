use crate::structs::{EndowmentStatus, SplitDetails, TaxParameters};
use cosmwasm_std::{Addr, Api, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub approved_coins: Option<Vec<Addr>>,
    pub accounts_code_id: Option<u64>,
    pub treasury: String,
    pub taxes: TaxParameters,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Add new AssetVault to VAULTS
    VaultAdd {
        vault_addr: String,
        vault_name: String,
        vault_description: String,
    },
    // Mark an AssetVault as approved (or not)
    VaultUpdateStatus {
        vault_addr: String,
        approved: bool,
    },
    // Removes an AssetVault
    VaultRemove {
        vault_addr: String,
    },
    CharityAdd {
        charity: String,
    },
    CharityRemove {
        charity: String,
    },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the DANO / AP Team to update the status of an Endowment
    // Approved, Frozen, (Liquidated, Terminated)
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Allows the SC owner (only!) to change ownership
    UpdateOwner {
        new_owner: String,
    },
    CreateEndowment(CreateEndowmentMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub owner: String,
    pub beneficiary: String,
    pub name: String,
    pub description: String,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub split_to_liquid: Option<SplitDetails>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_code_id: Option<u64>,
    pub index_fund_contract: String,
    pub approved_coins: Option<Vec<String>>,
    pub approved_charities: Option<Vec<String>>,
}

impl UpdateConfigMsg {
    pub fn coins_list(&self, api: &dyn Api) -> StdResult<Vec<Addr>> {
        match self.approved_coins.as_ref() {
            Some(v) => v.iter().map(|h| api.addr_validate(h)).collect(),
            None => Ok(vec![]),
        }
    }

    pub fn charities_list(&self, api: &dyn Api) -> StdResult<Vec<Addr>> {
        match self.approved_charities.as_ref() {
            Some(v) => v.iter().map(|h| api.addr_validate(h)).collect(),
            None => Ok(vec![]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentStatusMsg {
    pub endowment_addr: String,
    pub status: EndowmentStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details on a specific Vault
    Vault { vault_addr: String },
    // Gets list of all Vaults.
    VaultList {},
    // Gets list of all registered Endowments.
    EndowmentList {},
    // Get all Config details for the contract
    Config {},
}
