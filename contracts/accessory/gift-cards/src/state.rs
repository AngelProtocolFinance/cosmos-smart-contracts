use angel_core::structs::GenericBalance;
use cosmwasm_std::Addr;
use cw_asset::Asset;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM MULTISIG
    pub keeper: Addr,
    pub registrar_contract: Addr,
    pub next_deposit: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Deposit {
    pub sender: Addr,
    pub token: Asset,
    pub claimed: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DEPOSITS: Map<u64, Deposit> = Map::new("deposit");
pub const BALANCES: Map<Addr, GenericBalance> = Map::new("balance");
