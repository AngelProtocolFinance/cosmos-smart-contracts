use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal, Env, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use cw20::{Balance, Cw20CoinVerified};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,                      // DANO Address
    pub account_ledgers_sc: Addr,         // Address of Account Ledgers SC
    pub terra_alliance: Vec<Addr>,        // vec of terra charity alliance approved addresses
    pub active_fund_index: Uint128,       // index ID of the Active IndexFund
    pub fund_rotation_limit: Uint128, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,       // limit to number of members an IndexFund can have
    pub funding_goal: Option<Balance>, // donation funding limit to trigger early cycle of the Active IndexFund
    pub split_to_liquid_max: Decimal, // maximum % can split off into liquid account for an incoming donor deposit
    pub split_to_liquid_default: Decimal, // default % to split off into liquid account, if donor provided split is not present
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
    // Fund Specific: over-riding SC level setting to handle a fixed split value
    // Defines the % to split off into liquid account, and if defined overrides all other split
    pub split_to_liquid_default: Option<Decimal>,
    // Used for one-off funds that have an end date (ex. disaster recovery funds)
    pub expiry_time: Option<u64>,   // datetime int of index fund expiry
    pub expiry_height: Option<u64>, // block equiv of the expiry_datetime
}

impl IndexFund {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(expiry_height) = self.expiry_height {
            if env.block.height > expiry_height {
                return true;
            }
        }
        if let Some(expiry_time) = self.expiry_time {
            if env.block.time > Timestamp::from_seconds(expiry_time) {
                return true;
            }
        }
        false
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CURRENT_ROUND_DONATIONS: Map<String, GenericBalance> =
    Map::new("current_round_donations"); // EID mapped to Balances of donations
pub const FUNDS: Map<String, IndexFund> = Map::new("index_fund");
