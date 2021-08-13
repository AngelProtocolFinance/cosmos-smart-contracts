use angel_core::structs::{AssetVault, EndowmentEntry, TaxParameters};
use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// static PREFIX_REGISTRY_INDEXER: &[u8] = b"registry_indexer";
// static PREFIX_VAULT_INDEXER: &[u8] = b"vault_indexer";
// const MAX_LIMIT: u32 = 30;
// const DEFAULT_LIMIT: u32 = 10;

static PREFIX_REGISTRY: &[u8] = b"registry";
static PREFIX_VAULT: &[u8] = b"vault";

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM / DANO Address
    pub index_fund_contract: Addr,
    // List of all possible CW20 Token demoninations that we can accept
    // This is required to avoid a DoS attack with an invalid cw20 contract. See https://github.com/CosmWasm/cosmwasm-plus/issues/19
    pub approved_coins: Vec<Addr>,
    pub accounts_code_id: u64,
    pub treasury: Addr,
    pub taxes: TaxParameters,
}

impl Config {
    pub fn human_approved_coins(&self) -> Vec<String> {
        self.approved_coins.iter().map(|a| a.to_string()).collect()
    }
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

// VAULT Read/Write
pub fn vault_store(storage: &mut dyn Storage) -> Bucket<AssetVault> {
    bucket(storage, PREFIX_VAULT)
}

pub fn vault_read(storage: &dyn Storage) -> ReadonlyBucket<AssetVault> {
    bucket_read(storage, PREFIX_VAULT)
}

pub fn read_vaults<'a>(storage: &'a dyn Storage) -> StdResult<Vec<AssetVault>> {
    let entries: ReadonlyBucket<'a, AssetVault> = ReadonlyBucket::new(storage, PREFIX_VAULT);
    entries
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
