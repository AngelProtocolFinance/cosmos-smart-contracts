use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Deps, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, ReadonlyBucket};
use halo_token::common::OrderBy;
use halo_token::vesting::VestingInfo;

const KEY_CONFIG: &[u8] = b"config";
const PREFIX_KEY_VESTING_INFO: &[u8] = b"vesting_info";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub halo_token: Addr,
    pub genesis_time: u64,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton::<Config>(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read::<Config>(storage, KEY_CONFIG).load()
}

pub fn read_vesting_info(storage: &dyn Storage, address: &Addr) -> StdResult<VestingInfo> {
    bucket_read::<VestingInfo>(storage, PREFIX_KEY_VESTING_INFO).load(address.as_bytes())
}

pub fn store_vesting_info(
    storage: &mut dyn Storage,
    address: &Addr,
    vesting_info: &VestingInfo,
) -> StdResult<()> {
    bucket::<VestingInfo>(storage, PREFIX_KEY_VESTING_INFO).save(address.as_bytes(), vesting_info)
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_vesting_infos<'a>(
    deps: Deps,
    storage: &'a dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(Addr, VestingInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
    };

    let vesting_accounts: ReadonlyBucket<'a, VestingInfo> =
        ReadonlyBucket::new(storage, PREFIX_KEY_VESTING_INFO);

    vesting_accounts
        .range(start.as_deref(), end.as_deref(), order_by.into())
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((deps.api.addr_validate(&String::from_utf8_lossy(&k))?, v))
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    match start_after {
        Some(addr) => {
            let mut v = addr.as_bytes().to_vec();
            v.push(1);
            Some(v)
        }
        _ => None,
    }
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    match start_after {
        Some(addr) => Some(addr.as_bytes().to_vec()),
        _ => None,
    }
}
