use crate::structs::{AcceptedTokens, AccountType, EndowmentType, NetworkInfo, SplitDetails};
use cosmwasm_std::{Addr, Api, Decimal, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub treasury: String,
    pub tax_rate: Decimal,
    pub split_to_liquid: Option<SplitDetails>, // default %s to split off into liquid account, if donor provided split is not present
    pub accepted_tokens: Option<AcceptedTokens>, // list of approved native and CW20 coins can accept inward
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    VaultAdd(VaultAddMsg),
    VaultRemove {
        vault_addr: String,
    },
    VaultUpdate {
        vault_addr: String,
        approved: bool,
        restricted_from: Vec<EndowmentType>,
    },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner to change ownership
    UpdateOwner {
        new_owner: String,
    },
    // Updates the NETWORK_CONNECTIONS
    UpdateNetworkConnections {
        network_info: NetworkInfo,
        action: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_contract: Option<String>,
    pub index_fund_contract: Option<String>,
    pub treasury: Option<String>,
    pub tax_rate: Option<Decimal>,
    pub approved_charities: Option<Vec<String>>,
    pub split_max: Option<Decimal>,
    pub split_min: Option<Decimal>,
    pub split_default: Option<Decimal>,
    pub halo_token: Option<String>,
    pub gov_contract: Option<String>,
    pub charity_shares_contract: Option<String>,
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub accepted_tokens_native: Option<Vec<String>>,
    pub accepted_tokens_cw20: Option<Vec<String>>,
    pub applications_review: Option<String>,
    pub swaps_router: Option<String>,
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
pub struct VaultAddMsg {
    pub network: Option<String>,
    pub vault_addr: String,
    pub input_denom: String,
    pub yield_token: String,
    pub restricted_from: Vec<EndowmentType>,
    pub acct_type: AccountType,
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
        network: Option<String>,
        endowment_type: Option<EndowmentType>,
        acct_type: Option<AccountType>,
        approved: Option<bool>,
        start_after: Option<String>,
        limit: Option<u64>,
    },
    // Get all Config details for the contract
    Config {},
    // Get a list of all approved Vaults exchange rates
    ApprovedVaultRateList {},
    // Get a network connection info
    NetworkConnection {
        chain_id: String,
    },
}
