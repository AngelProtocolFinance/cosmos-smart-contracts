use crate::structs::{
    AcceptedTokens, AccountType, EndowmentType, NetworkInfo, RebalanceDetails, SplitDetails,
    VaultType,
};
use cosmwasm_std::{Addr, Api, Decimal, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub accounts_settings_controller: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateEndowment {
    pub addr: String,
    pub status: u64,
    pub name: String,
    pub owner: String,
    pub tier: Option<u64>,
    pub un_sdg: Option<u64>,
    pub logo: Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub treasury: String,
    pub tax_rate: Decimal,
    pub rebalance: Option<RebalanceDetails>,
    pub split_to_liquid: Option<SplitDetails>, // default %s to split off into liquid account, if donor provided split is not present
    pub accepted_tokens: Option<AcceptedTokens>, // list of approved native and CW20 coins can accept inward
    pub swap_factory: Option<String>,
    pub accounts_settings_controller: String,
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
    UpdateFees {
        fees: Vec<(String, Decimal)>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub accounts_contract: Option<String>,
    pub tax_rate: Option<Decimal>,
    pub rebalance: Option<RebalanceDetails>,
    pub approved_charities: Option<Vec<String>>,
    pub split_max: Option<Decimal>,
    pub split_min: Option<Decimal>,
    pub split_default: Option<Decimal>,
    pub collector_share: Option<Decimal>,
    pub accepted_tokens: Option<AcceptedTokens>,
    /// WASM CODES
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub subdao_gov_code: Option<u64>,        // subdao gov wasm code
    pub subdao_cw20_token_code: Option<u64>, // subdao gov token (basic CW20) wasm code
    pub subdao_bonding_token_code: Option<u64>, // subdao gov token (w/ bonding-curve) wasm code
    pub subdao_cw900_code: Option<u64>,      // subdao gov ve-CURVE contract for locked token voting
    pub subdao_distributor_code: Option<u64>, // subdao gov fee distributor wasm code
    pub donation_match_code: Option<u64>,    // donation matching contract wasm code
    /// CONTRACT ADDRESSES
    pub index_fund_contract: Option<String>,
    pub gov_contract: Option<String>,
    pub treasury: Option<String>,
    pub donation_match_charites_contract: Option<String>,
    pub halo_token: Option<String>,
    pub halo_token_lp_contract: Option<String>,
    pub charity_shares_contract: Option<String>,
    pub collector_addr: Option<String>,
    pub swap_factory: Option<String>,
    pub fundraising_contract: Option<String>,
    pub applications_review: Option<String>,
    pub swaps_router: Option<String>,
    pub accounts_settings_controller: Option<String>,
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
    pub vault_type: VaultType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get details on single vault
    Vault { vault_addr: String },
    // Get all Config details for the contract
    Config {},
    // Get a network connection info
    NetworkConnection { chain_id: String },
    Fee { name: String },
}
