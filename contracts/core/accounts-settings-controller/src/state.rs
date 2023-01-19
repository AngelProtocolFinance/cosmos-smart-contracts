use angel_core::structs::{EndowmentController, EndowmentSettings};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,              // DANO/AP Team Address
    pub registrar_contract: Addr, // Registrar contract address
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SETTINGS: Map<u32, EndowmentSettings> = Map::new("endowment-settings");
pub const CONTROLLER: Map<u32, EndowmentController> = Map::new("endowment-controller");
