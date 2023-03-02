use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub reserve_token: Addr,
    pub lp_pair_contract: Addr,
    pub registrar_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
