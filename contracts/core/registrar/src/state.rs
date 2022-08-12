use angel_core::structs::{
    AcceptedTokens, EndowmentEntry, EndowmentType, NetworkInfo, SplitDetails, YieldVault,
};
use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM MULTISIG
    pub index_fund_contract: Option<Addr>,
    pub accounts_contract: Option<Addr>,
    pub treasury: Addr,
    pub tax_rate: Decimal,
    pub default_vault: Option<Addr>,
    pub cw3_code: Option<u64>,                  // multisig wasm code
    pub cw4_code: Option<u64>,                  // multisig wasm code
    pub subdao_gov_code: Option<u64>,           // subdao gov wasm code
    pub subdao_cw20_token_code: Option<u64>,    // subdao gov cw20 token wasm code
    pub subdao_bonding_token_code: Option<u64>, // subdao gov bonding curve token wasm code
    pub subdao_cw900_code: Option<u64>, // subdao gov ve-CURVE contract for locked token voting
    pub subdao_distributor_code: Option<u64>, // subdao gov fee distributor wasm code
    pub donation_match_code: Option<u64>, // donation matching contract wasm code
    pub donation_match_charites_contract: Option<Addr>, // donation matching contract address for "Charities" endowments
    pub split_to_liquid: SplitDetails, // set of max, min, and default Split paramenters to check user defined split input against
    pub halo_token: Option<Addr>,      // TerraSwap HALO token addr
    pub halo_token_lp_contract: Option<Addr>,
    pub gov_contract: Option<Addr>,   // AP governance contract
    pub collector_addr: Option<Addr>, // Collector address for new fee
    pub collector_share: Decimal,
    pub charity_shares_contract: Option<Addr>,
    pub accepted_tokens: AcceptedTokens, // list of approved native and CW20 coins can accept inward
    pub swap_factory: Option<Addr>,
    pub fundraising_contract: Option<Addr>,
    pub account_id_char_limit: usize,
}

pub const REGISTRY: Map<&str, EndowmentEntry> = Map::new("registry");
pub const VAULTS: Map<&[u8], YieldVault> = Map::new("vault");
pub const NETWORK_CONNECTIONS: Map<&str, NetworkInfo> = Map::new("network_connections");
pub const ENDOWTYPE_FEES: Map<String, Option<Decimal>> = Map::new("endowment_type_fees");

pub fn read_registry_entries(storage: &dyn Storage) -> StdResult<Vec<EndowmentEntry>> {
    REGISTRY
        .range(storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

pub fn read_vaults(
    storage: &dyn Storage,
    network: Option<String>,
    endowment_type: Option<EndowmentType>,
    approved: Option<bool>,
    start_after: Option<Addr>,
    limit: Option<u64>,
) -> StdResult<Vec<YieldVault>> {
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.as_bytes().to_vec()));
    VAULTS
        .range(storage, start, None, Order::Ascending)
        .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .filter(|vault| match vault {
            Ok(v) => match (approved, v.approved) {
                (None, _) => true,
                (Some(true), true) | (Some(false), false) => true,
                _ => false,
            },
            &Err(_) => false,
        })
        .filter(|vault| match vault {
            Ok(v) => match &endowment_type {
                Some(et) => match v.restricted_from.iter().position(|t| &t == &et) {
                    Some(_) => false,
                    None => true,
                },
                None => true,
            },
            &Err(_) => false,
        })
        .filter(|vault| match vault {
            Ok(v) => match &network {
                Some(n) => n == &v.network,
                None => true,
            },
            &Err(_) => false,
        })
        .collect()
}
