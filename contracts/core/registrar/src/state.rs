use angel_core::structs::{EndowmentEntry, YieldVault};
use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// static PREFIX_REGISTRY_INDEXER: &[u8] = b"registry_indexer";
// const MAX_LIMIT: u32 = 30;
// const DEFAULT_LIMIT: u32 = 10;

static PREFIX_REGISTRY: &[u8] = b"registry";
static PREFIX_PORTAL: &[u8] = b"vault";

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM / DANO Address
    pub index_fund_contract: Addr,
    pub accounts_code_id: u64,
    pub approved_charities: Vec<Addr>,
    pub treasury: Addr,
    pub tax_rate: Decimal,
    pub default_vault: Addr,
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
pub fn vault_store(storage: &mut dyn Storage) -> Bucket<YieldVault> {
    bucket(storage, PREFIX_PORTAL)
}

pub fn vault_read(storage: &dyn Storage) -> ReadonlyBucket<YieldVault> {
    bucket_read(storage, PREFIX_PORTAL)
}

pub fn read_vaults<'a>(storage: &'a dyn Storage) -> StdResult<Vec<YieldVault>> {
    let entries: ReadonlyBucket<'a, YieldVault> = ReadonlyBucket::new(storage, PREFIX_PORTAL);
    entries
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
