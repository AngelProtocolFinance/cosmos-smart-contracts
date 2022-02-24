use angel_core::structs::{EndowmentEntry, SplitDetails, YieldVault};
use angel_core::utils::calc_range_start_addr;
use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// static PREFIX_REGISTRY_INDEXER: &[u8] = b"registry_indexer";
const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

static PREFIX_REGISTRY: &[u8] = b"registry";
static PREFIX_PORTAL: &[u8] = b"vault";

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,           // AP TEAM MULTISIG
    pub guardian_angels: Addr, // GUARDIAN ANGELS MULTISIG
    pub index_fund_contract: Addr,
    pub accounts_code_id: u64,
    pub approved_charities: Vec<Addr>,
    pub treasury: Addr,
    pub tax_rate: Decimal,
    pub default_vault: Addr,
    pub guardians_multisig_addr: Option<String>,
    pub endowment_owners_group_addr: Option<String>,
    pub split_to_liquid: SplitDetails, // set of max, min, and default Split paramenters to check user defined split input against
    pub halo_token: Option<Addr>,      // TerraSwap HALO token addr
    pub gov_contract: Option<Addr>,    // AP governance contract
    pub charity_shares_contract: Option<Addr>, // Charity Shares staking contract
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

pub fn read_vaults<'a>(
    storage: &'a dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u64>,
) -> StdResult<Vec<YieldVault>> {
    let vaults: ReadonlyBucket<'a, YieldVault> = ReadonlyBucket::new(storage, PREFIX_PORTAL);
    vaults
        .range(
            calc_range_start_addr(start_after).as_deref(),
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
