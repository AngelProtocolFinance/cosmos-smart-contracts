use angel_core::structs::BalanceInfo;
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub registrar_contract: Addr,
    pub moneymarket: Addr,
    pub input_denom: String,
    pub yield_token: Addr,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, CONFIG_KEY).load()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MinterData {
    pub minter: Addr,
    /// cap is how many more tokens can be issued by the minter
    pub cap: Option<Uint128>,
}

impl TokenInfo {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PendingDepositInfo {
    pub id: Uint128,
    pub locked: Uint128,
    pub liquid: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PendingRedemptionInfo {
    pub id: Uint128,
    pub account_address: Addr,
    pub locked: Uint128,
    pub liquid: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PendingWithdrawInfo {
    pub id: Uint128,
    pub beneficiary: Addr,
    pub locked: Uint128,
    pub liquid: Uint128,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const BALANCES: Map<&Addr, BalanceInfo> = Map::new("balance");
pub const PENDING_DEPOSITS: Map<&Addr, Vec<PendingDepositInfo>> = Map::new("deposit");
pub const PENDING_REDEMPTIONS: Map<&Addr, Vec<PendingRedemptionInfo>> = Map::new("redemption");
pub const PENDING_WITHDRAWS: Map<&Addr, Vec<PendingWithdrawInfo>> = Map::new("withdraw");
