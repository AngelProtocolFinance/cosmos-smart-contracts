use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub halo_token: Addr,
    pub staking_token: Addr,
    pub distribution_schedule: Vec<(u64, u64, Uint128)>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_distributed: u64,
    pub total_bond_amount: Uint128,
    pub global_reward_index: Decimal,
}

pub const STATE: Item<State> = Item::new("state");
pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
}

pub const STAKER_INFO: Map<&[u8], StakerInfo> = Map::new("reward");

/// returns return staker_info of the given owner
pub fn store_staker_info(
    storage: &mut dyn Storage,
    owner: &Addr,
    staker_info: &StakerInfo,
) -> StdResult<()> {
    STAKER_INFO.save(storage, owner.as_bytes(), staker_info)
}

/// remove staker_info of the given owner
pub fn remove_staker_info(storage: &mut dyn Storage, owner: &Addr) {
    STAKER_INFO.remove(storage, owner.as_bytes())
}

/// returns rewards owned by this owner
/// (read-only version for queries)
pub fn read_staker_info(storage: &dyn Storage, owner: &Addr) -> StdResult<StakerInfo> {
    match STAKER_INFO.may_load(storage, owner.as_bytes())? {
        Some(staker_info) => Ok(staker_info),
        None => Ok(StakerInfo {
            reward_index: Decimal::zero(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
        }),
    }
}
