use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub gov_contract: Addr, // HALO Gov Contract address
    pub halo_token: Addr,   // HALO token address
}

pub const CONFIG: Item<Config> = Item::new("config");

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
