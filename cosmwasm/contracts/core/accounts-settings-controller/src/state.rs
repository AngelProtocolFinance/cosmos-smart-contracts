use angel_core::structs::{EndowmentController, EndowmentSettings};
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,              // DANO/AP Team Address
    pub registrar_contract: Addr, // Registrar contract address
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SETTINGS: Map<u32, EndowmentSettings> = Map::new("endowment-settings");
pub const CONTROLLER: Map<u32, EndowmentController> = Map::new("endowment-controller");
