use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_storage_plus::Item;

use cosmwasm_std::{Addr, StdResult, Storage, Uint128};

#[cw_serde]
pub struct Config {
    pub gov_contract: Addr,   // HALO gov address
    pub halo_token: Addr,     // HALO token address
    pub spend_limit: Uint128, // spend limit per each `spend` request
}

pub const CONFIG: Item<Config> = Item::new("config");

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
