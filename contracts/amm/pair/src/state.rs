use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use halo_amm::asset::PairInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub factory_addr: Addr,
    pub liquidity_token: Addr,
    pub collector_addr: Addr,
    pub commission_rate: String,
}

// put the length bytes at the first for compatibility with legacy singleton store
pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");

// put the length bytes at the first for compatibility with legacy singleton store
pub const PAIR_INFO: Item<PairInfo> = Item::new("\u{0}\u{9}amm_pair_info");
