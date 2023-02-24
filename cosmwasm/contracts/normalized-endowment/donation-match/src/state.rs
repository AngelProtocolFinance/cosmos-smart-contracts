use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub reserve_token: Addr,
    pub lp_pair_contract: Addr,
    pub registrar_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
