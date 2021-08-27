use cosmwasm_std::{Addr, Api, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub accounts_code_id: Option<u64>,
    pub treasury: String,
    pub tax_rate: u64,
    pub default_vault: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateEndowment(CreateEndowmentMsg),
    VaultAdd(VaultAddMsg),
    VaultUpdateStatus { vault_addr: String, approved: bool },
    CharityAdd { charity: String },
    CharityRemove { charity: String },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the DANO / AP Team to update the status of an Endowment
    // Approved, Frozen, (Liquidated, Terminated)
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Allows the SC owner to change ownership
    UpdateOwner { new_owner: String },
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_code_id: Option<u64>,
    pub index_fund_contract: String,
    pub vaults: Option<Vec<String>>,
    pub approved_charities: Option<Vec<String>>,
    pub default_vault: Option<String>,
}

impl UpdateConfigMsg {
    pub fn vaults_list(&self, api: &dyn Api) -> StdResult<Vec<Addr>> {
        match self.vaults.as_ref() {
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
    pub status: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultAddMsg {
    pub vault_addr: String,
    pub input_denom: String,
    pub yield_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details on single vault
    Vault { vault_addr: String },
    // Gets list of all Vaults
    VaultList {},
    // Get a list of all approved Vaults
    ApprovedVaultList {},
    // Get a list of all approved Endowments
    ApprovedEndowmentList {},
    // Gets list of all registered Endowments
    EndowmentList {},
    // Get all Config details for the contract
    Config {},
}
