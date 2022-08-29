use angel_core::structs::AccountType;
use cosmwasm_std::{Addr, Decimal, Decimal256, StdResult, Storage, Uint128};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub registrar_contract: Addr,
    pub acct_type: AccountType,
    pub sibling_vault: Addr,
    pub moneymarket: Addr,
    pub input_denom: String,
    pub yield_token: Addr,
    pub next_pending_id: u64,
    pub tax_per_block: Decimal,
    pub last_harvest: u64,
    pub last_harvest_fx: Option<Decimal256>,
    pub harvest_to_liquid: Decimal,
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
pub struct PendingInfo {
    pub typ: String, // type of pending transaction ('typ', because 'type' is protected keyword in Rust...)
    pub endowment_id: u32, // ID of Endowment that send msg from Accounts contract
    pub beneficiary: Option<Addr>, // return to the beneficiary
    pub fund: Option<u64>, // return to the active fund
    pub amount: Uint128,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const TREASURY_TOKENS: Item<Uint128> = Item::new("treasury_tokens");
pub const BALANCES: Map<&u32, Uint128> = Map::new("balances");
pub const PENDING: Map<&[u8], PendingInfo> = Map::new("pending");
pub const DEPOSIT_TOKEN_DENOM: Item<String> = Item::new("deposit_token_denom");
