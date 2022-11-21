use angel_core::structs::{
    AccountStrategies, BalanceInfo, Beneficiary, Categories, DonationsReceived, EndowmentFee,
    EndowmentSettings, EndowmentStatus, EndowmentType, OneOffVaults, RebalanceDetails,
    SettingsController, SplitDetails,
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
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ENDOWMENTSETTINGS: Map<u32, EndowmentSettings> = Map::new("endowmentsettings");
