use angel_core::structs::EndowmentStatus;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM / DANO Address
    pub index_fund_contract: Addr,
    // List of all possible CW20 Token demoninations that we can accept
    // This is required to avoid a DoS attack with an invalid cw20 contract. See https://github.com/CosmWasm/cosmwasm-plus/issues/19
    pub approved_coins: Vec<Addr>,
    pub accounts_code_id: u64,
}

impl Config {
    pub fn human_approved_coins(&self) -> Vec<String> {
        self.approved_coins.iter().map(|a| a.to_string()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EndowmentEntry {
    pub name: String,
    pub description: String,
    pub status: EndowmentStatus,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const REGISTRY: Map<String, EndowmentEntry> = Map::new("endowment");
pub const VAULTS: Map<String, bool> = Map::new("vault");
