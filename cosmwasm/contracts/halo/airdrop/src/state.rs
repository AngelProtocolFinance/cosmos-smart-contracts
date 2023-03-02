use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::{Item, Map};

// "Config" Storage Item & its utils.
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub halo_token: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

// "latest_stage" Storage item & its utils.
pub const LATEST_STAGE: Item<u8> = Item::new("latest_stage");
pub fn store_latest_stage(storage: &mut dyn Storage, stage: u8) -> StdResult<()> {
    LATEST_STAGE.save(storage, &stage)
}

pub fn read_latest_stage(storage: &dyn Storage) -> StdResult<u8> {
    LATEST_STAGE.load(storage)
}

// "merkle_root" storage map & its utils. (stage -> merkle_root)
pub const MERKLE_ROOT: Map<&[u8], String> = Map::new("merkle_root");
pub fn store_merkle_root(
    storage: &mut dyn Storage,
    stage: u8,
    merkle_root: String,
) -> StdResult<()> {
    MERKLE_ROOT.save(storage, &[stage], &merkle_root)
}

pub fn read_merkle_root(storage: &dyn Storage, stage: u8) -> StdResult<String> {
    MERKLE_ROOT.load(storage, &[stage])
}

// "claim_index" storage map & its utils: ((address, stage) -> bool)
pub const CLAIM_INDEX: Map<(&str, &str), bool> = Map::new("claim_index");
pub fn store_claimed(storage: &mut dyn Storage, user: &Addr, stage: u8) -> StdResult<()> {
    CLAIM_INDEX.save(storage, (user.as_str(), stage.to_string().as_str()), &true)
}

pub fn read_claimed(storage: &dyn Storage, user: &Addr, stage: u8) -> StdResult<bool> {
    match CLAIM_INDEX.may_load(storage, (user.as_str(), stage.to_string().as_str()))? {
        Some(v) => Ok(v),
        None => Ok(false),
    }
}
