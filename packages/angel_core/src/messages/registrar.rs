use crate::structs::{EndowmentType, Profile, SplitDetails, Tier};
use cosmwasm_std::{Addr, Api, Decimal, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

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
    UpdateEndowmentEntry(UpdateEndowmentEntryMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub owner: String,
    pub beneficiary: String,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub guardians_multisig_addr: Option<String>,
    pub profile: Profile,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_code_id: Option<u64>,
    pub index_fund_contract: Option<String>,
    pub treasury: Option<String>,
    pub tax_rate: Option<Decimal>,
    pub approved_charities: Option<Vec<String>>,
    pub default_vault: Option<String>,
    pub guardians_multisig_addr: Option<String>,
    pub endowment_owners_group_addr: Option<String>,
    pub split_max: Option<Decimal>,
    pub split_min: Option<Decimal>,
    pub split_default: Option<Decimal>,
    pub halo_token: Option<String>,
    pub gov_contract: Option<String>,
    pub charity_shares_contract: Option<String>,
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
pub struct UpdateEndowmentEntryMsg {
    pub endowment_addr: String,
    pub name: Option<String>,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub owner: Option<String>,
    pub tier: Option<Option<Tier>>,
    pub un_sdg: Option<Option<u64>>,
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
        status: Option<String>,
        name: Option<Option<String>>,
        owner: Option<Option<String>>,
        tier: Option<Option<String>>,
        un_sdg: Option<Option<u64>>,
        endow_type: Option<Option<String>>,
    },
    // Get all Config details for the contract
    Config {},
    // Get a list of all approved Vaults exchange rates
    ApprovedVaultRateList {},
}
