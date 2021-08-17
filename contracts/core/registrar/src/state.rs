use angel_core::structs::{EndowmentEntry, TaxParameters, YieldPortal};
use cosmwasm_std::{Addr, Order, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// static PREFIX_REGISTRY_INDEXER: &[u8] = b"registry_indexer";
// const MAX_LIMIT: u32 = 30;
// const DEFAULT_LIMIT: u32 = 10;

static PREFIX_REGISTRY: &[u8] = b"registry";
static PREFIX_PORTAL: &[u8] = b"portal";

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM / DANO Address
    pub index_fund_contract: Addr,
    pub accounts_code_id: u64,
    pub approved_charities: Vec<Addr>,
    pub treasury: Addr,
    pub taxes: TaxParameters,
    pub default_portal: Addr,
}

// REGISTRY Read/Write
pub fn registry_store(storage: &mut dyn Storage) -> Bucket<EndowmentEntry> {
    bucket(storage, PREFIX_REGISTRY)
}

pub fn registry_read(storage: &dyn Storage) -> ReadonlyBucket<EndowmentEntry> {
    bucket_read(storage, PREFIX_REGISTRY)
}

pub fn read_registry_entries<'a>(storage: &'a dyn Storage) -> StdResult<Vec<EndowmentEntry>> {
    let entries: ReadonlyBucket<'a, EndowmentEntry> = ReadonlyBucket::new(storage, PREFIX_REGISTRY);
    entries
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

// PORTAL Read/Write
pub fn portal_store(storage: &mut dyn Storage) -> Bucket<YieldPortal> {
    bucket(storage, PREFIX_PORTAL)
}

pub fn portal_read(storage: &dyn Storage) -> ReadonlyBucket<YieldPortal> {
    bucket_read(storage, PREFIX_PORTAL)
}

pub fn read_portals<'a>(storage: &'a dyn Storage) -> StdResult<Vec<YieldPortal>> {
    let entries: ReadonlyBucket<'a, YieldPortal> = ReadonlyBucket::new(storage, PREFIX_PORTAL);
    entries
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
