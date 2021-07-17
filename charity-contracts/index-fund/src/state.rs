use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};

use cw20::{Balance, Cw20CoinVerified};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // DANO Address
    pub account_ledgers_sc: Addr, // Address of Account Ledgers SC
    pub terra_alliance: Vec<Addr>, // vec of terra charity alliance approved addresses
    pub active_fund_index: Uint128, // index ID of the Active IndexFund
    pub fund_rotation_limit: Uint128, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32, // limit to number of members an IndexFund can have
    pub funding_goal: Option<Balance>, // donation funding limit to trigger early cycle of the Active IndexFund
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IndexFund {
    pub name: String,
    pub description: String,
    pub members: Vec<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CURRENT_ROUND_DONATIONS: Map<String, GenericBalance> = Map::new("current_round_donations"); // EID mapped to Balances of donations
pub const FUNDS: Map<String, IndexFund> = Map::new("index_fund");
