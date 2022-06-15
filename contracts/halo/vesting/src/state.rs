use cosmwasm_std::{Addr, Deps, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use halo_token::common::OrderBy;
use halo_token::vesting::VestingInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub halo_token: Addr,
    pub genesis_time: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub const VESTING_INFO: Map<&[u8], VestingInfo> = Map::new("vesting_info");
pub fn read_vesting_info(storage: &dyn Storage, address: &Addr) -> StdResult<VestingInfo> {
    VESTING_INFO.load(storage, address.as_bytes())
}

pub fn store_vesting_info(
    storage: &mut dyn Storage,
    address: &Addr,
    vesting_info: &VestingInfo,
) -> StdResult<()> {
    VESTING_INFO.save(storage, address.as_bytes(), vesting_info)
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_vesting_infos(
    deps: Deps,
    storage: &dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(Addr, VestingInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (start_after, None, OrderBy::Asc),
        _ => (None, start_after, OrderBy::Desc),
    };

    VESTING_INFO
        .range(
            storage,
            start.map(|s| Bound::ExclusiveRaw(s.as_bytes().to_vec())),
            end.map(|s| Bound::InclusiveRaw(s.as_bytes().to_vec())),
            order_by.into(),
        )
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((deps.api.addr_validate(&String::from_utf8_lossy(&k))?, v))
        })
        .collect()
}
