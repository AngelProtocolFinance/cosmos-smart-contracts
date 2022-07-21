use angel_core::structs::BalanceInfo;
use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub registrar_contract: Addr,

    pub next_pending_id: u64,
    pub last_harvest: u64,
    pub last_harvest_fx: Option<Uint128>,
    pub harvest_to_liquid: Decimal,

    pub target: Addr, // swap pool address(eg. JunoSwap USDC-JUNO pool address)
    pub input_denoms: Vec<String>, // swap input tokens(denoms) list
    pub yield_token: Addr, // swap lp token address
    pub routes: Vec<Addr>, // list of swap pools(eg. list of junoswap pools)

    pub total_assets: Uint128, // total value of assets deposited from endowments (in usdc/usd)
    pub total_shares: Uint128, // total amount of minted vault tokens
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    CONFIG.save(storage, data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
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
    pub accounts_address: Addr, // Addr of org. sending Accounts SC
    pub beneficiary: Option<Addr>, // return to the beneficiary
    pub fund: Option<u64>, // return to the active fund
    pub locked: Uint128,
    pub liquid: Uint128,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const BALANCES: Map<&Addr, BalanceInfo> = Map::new("balance");
pub const PENDING: Map<&[u8], PendingInfo> = Map::new("pending");
