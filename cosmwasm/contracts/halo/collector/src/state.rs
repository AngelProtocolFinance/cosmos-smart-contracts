use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_storage_plus::Item;

use cosmwasm_std::{Addr, Decimal, StdResult, Storage};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub gov_contract: Addr,         // collected rewards receiver
    pub swap_factory: Addr,         // terraswap factory contract
    pub halo_token: Addr,           // HALO token address
    pub distributor_contract: Addr, // distributor contract to sent back rewards
    pub reward_factor: Decimal, // reward distribution rate to gov contract, left rewards sent back to distributor contract
}

pub const CONFIG: Item<Config> = Item::new("config");

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
