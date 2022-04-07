use angel_core::structs::{EndowmentEntry, SplitDetails, YieldVault};
use angel_core::utils::calc_range_start_addr;
use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// static PREFIX_REGISTRY_INDEXER: &[u8] = b"registry_indexer";
const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM MULTISIG
    pub index_fund_contract: Option<Addr>,
    pub accounts_code_id: u64,
    pub treasury: Addr,
    pub tax_rate: Decimal,
    pub default_vault: Option<Addr>,
    pub cw3_code: Option<u64>,                // multisig wasm code
    pub cw4_code: Option<u64>,                // multisig wasm code
    pub subdao_gov_code: Option<u64>,         // subdao gov wasm code
    pub subdao_token_code: Option<u64>,       // subdao gov token (w/ bonding-curve) wasm code
    pub subdao_cw900_code: Option<u64>, // subdao gov ve-CURVE contract for locked token voting
    pub subdao_distributor_code: Option<u64>, // subdao gov fee distributor wasm code
    pub donation_match_code: Option<u64>, // donation matching contract wasm code
    pub split_to_liquid: SplitDetails, // set of max, min, and default Split paramenters to check user defined split input against
    pub halo_token: Option<Addr>,      // TerraSwap HALO token addr
    pub gov_contract: Option<Addr>,    // AP governance contract
    pub donation_match_charites_contract: Option<Addr>, // donation matching contract address for "Charities" endowments
}

pub const PREFIX_REGISTRY: Map<&[u8], EndowmentEntry> = Map::new("registry");

// REGISTRY Read/Write
pub fn registry_store(storage: &mut dyn Storage, k: &[u8], data: &EndowmentEntry) -> StdResult<()> {
    PREFIX_REGISTRY.save(storage, k, data)
}

pub fn registry_read(storage: &dyn Storage, k: &[u8]) -> StdResult<EndowmentEntry> {
    PREFIX_REGISTRY.load(storage, k)
}

pub fn read_registry_entries<'a>(storage: &'a dyn Storage) -> StdResult<Vec<EndowmentEntry>> {
    PREFIX_REGISTRY
        .range(storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

pub const PREFIX_PORTAL: Map<&[u8], YieldVault> = Map::new("vault");

// PORTAL Read/Write
pub fn vault_store(storage: &mut dyn Storage, k: &[u8], data: &YieldVault) -> StdResult<()> {
    PREFIX_PORTAL.save(storage, k, data)
}

pub fn vault_read(storage: &dyn Storage, k: &[u8]) -> StdResult<YieldVault> {
    PREFIX_PORTAL.load(storage, k)
}

pub fn vault_remove(storage: &mut dyn Storage, k: &[u8]) {
    PREFIX_PORTAL.remove(storage, k)
}

pub fn read_vaults<'a>(
    storage: &'a dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u64>,
) -> StdResult<Vec<YieldVault>> {
    let start = calc_range_start_addr(start_after);
    PREFIX_PORTAL
        .range(
            storage,
            start.and_then(|v| Some(Bound::Inclusive(v))),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
