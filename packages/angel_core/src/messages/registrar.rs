use crate::messages::dao_token::CurveType;
use crate::structs::{EndowmentType, SplitDetails, Tier};
use cosmwasm_std::{Addr, Api, Decimal, StdResult};
use cw4::Member;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    // [ (address, status, name, owner, tier), ...]
    pub endowments: Vec<MigrateEndowment>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateEndowment {
    pub addr: String,
    pub status: u64,
    pub name: String,
    pub owner: String,
    pub tier: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub accounts_code_id: Option<u64>,
    pub treasury: String,
    pub tax_rate: Decimal,
    pub default_vault: Option<Addr>,
    pub split_to_liquid: Option<SplitDetails>, // default %s to split off into liquid account, if donor provided split is not present
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateEndowment(CreateEndowmentMsg),
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    VaultAdd(VaultAddMsg),
    VaultRemove {
        vault_addr: String,
    },
    VaultUpdateStatus {
        vault_addr: String,
        approved: bool,
    },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner to change ownership
    UpdateOwner {
        new_owner: String,
    },
    // Allows the DANO/AP Team to harvest all active vaults
    Harvest {
        collector_address: String,
        collector_share: Decimal,
    },
    // Allows the DANO/AP Team to update the EndowmentEntry
    UpdateEndowmentType(UpdateEndowmentTypeMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub owner: String,
    pub name: String,
    pub description: String,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub split_max: Option<Decimal>,
    pub split_min: Option<Decimal>,
    pub split_default: Option<Decimal>,
    pub locked_endowment_configs: Vec<String>,
    pub whitelisted_beneficiaries: Vec<String>,
    pub whitelisted_contributors: Vec<String>,
    pub cw4_members: Vec<Member>,
    pub dao: bool,
    pub donation_match: bool,
    pub curve_type: Option<CurveType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_code_id: Option<u64>,
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub index_fund_contract: Option<String>,
    pub gov_contract: Option<String>,
    pub halo_token: Option<String>,
    pub treasury: Option<String>,
    pub tax_rate: Option<Decimal>,
    pub approved_charities: Option<Vec<String>>,
    pub default_vault: Option<String>,
    pub split_max: Option<Decimal>,
    pub split_min: Option<Decimal>,
    pub split_default: Option<Decimal>,
}

impl UpdateConfigMsg {
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
    pub beneficiary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultAddMsg {
    pub vault_addr: String,
    pub input_denom: String,
    pub yield_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentTypeMsg {
    pub endowment_addr: String,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub tier: Option<Option<Tier>>,
    pub endow_type: Option<EndowmentType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details on single vault
    Vault {
        vault_addr: String,
    },
    // Gets list of all Vaults
    VaultList {
        start_after: Option<String>,
        limit: Option<u64>,
    },
    // Get a list of all approved Vaults
    ApprovedVaultList {
        start_after: Option<String>,
        limit: Option<u64>,
    },
    // Get details of single Endowment
    Endowment {
        endowment_addr: String,
    },
    // Gets list of all registered Endowments
    EndowmentList {
        name: Option<String>,
        owner: Option<String>,
        status: Option<String>,
        tier: Option<Option<String>>,
        endow_type: Option<String>,
    },
    // Get all Config details for the contract
    Config {},
    // Get a list of all approved Vaults exchange rates
    ApprovedVaultRateList {},
}
