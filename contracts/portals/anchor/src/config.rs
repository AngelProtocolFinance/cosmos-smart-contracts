use cosmwasm_std::{Addr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub deposit_token: Addr,
    pub moneymarket: Addr,
    pub input_denom: String,
    pub yield_token: Addr,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, CONFIG_KEY).load()
}
