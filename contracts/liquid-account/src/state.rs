use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub locked_account: Option<Addr>,
    pub index_fund: Option<Addr>,
    pub investment_strategy: Option<Addr>,
    // pub cw20_approved_coins: Vec<Addr>, // TODO: should it be there or could there be some kind of a link to another SC (Locked SC or more general)?
}

pub const CONFIG: Item<Config> = Item::new("config");
