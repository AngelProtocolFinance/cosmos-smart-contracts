use angel_core::structs::{
    AccountStrategies, BalanceInfo, Beneficiary, Categories, DonationsReceived, EndowmentFee,
    EndowmentStatus, EndowmentType, OneOffVaults, RebalanceDetails, SettingsController,
    SplitDetails,
};
use cosmwasm_std::{Addr, Env, Order, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: u64 = 15;
const MAX_LIMIT: u64 = 80;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub accounts_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ENDOWMENTS: Map<u32, SettingsController> = Map::new("endowments");
