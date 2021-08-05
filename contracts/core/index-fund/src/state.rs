use angel_core::structs::{GenericBalance, IndexFund, SplitDetails};
use cosmwasm_std::{Addr, Uint128};
use cw20::Balance;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,                   // DANO Address
    pub registrar_contract: Addr,      // Address of Registrar SC
    pub terra_alliance: Vec<Addr>,     // Terra Charity Alliance approved addresses
    pub active_fund_index: String,     // index ID of the Active IndexFund
    pub funds_list: Vec<String>,       // Vec of String IDs for all Funds
    pub fund_rotation_limit: Uint128, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,       // limit to number of members an IndexFund can have
    pub funding_goal: Option<Balance>, // donation funding limit to trigger early cycle of the Active IndexFund
    pub split_to_liquid: SplitDetails, // default %s to split off into liquid account, if donor provided split is not present
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DONATIONS: Map<Addr, GenericBalance> = Map::new("donation");
pub const FUNDS: Map<String, IndexFund> = Map::new("fund");
