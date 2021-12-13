use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,             // AP Team owner
    pub gov_contract: Addr,      // collected rewards receiver
    pub terraswap_factory: Addr, // terraswap factory contract
    pub halo_token: Addr,        // HALO token address
    pub treasury_addr: Addr,     // treasury address contract/wallet to send back rewards
    pub reward_factor: Decimal, // reward distribution rate to gov contract, left rewards sent back to distributor contract
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
