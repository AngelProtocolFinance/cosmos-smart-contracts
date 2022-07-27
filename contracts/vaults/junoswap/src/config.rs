use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cw20::Denom;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub registrar_contract: Addr,

    pub last_harvest: u64,
    pub last_harvest_fx: Option<Uint128>,

    pub pool_addr: Addr, // swap pool address(eg. JunoSwap USDC-JUNO pool address)
    pub input_denoms: Vec<Denom>, // swap input tokens(denoms) list
    pub pool_lp_token_addr: Addr, // swap lp token address
    pub routes: Vec<Addr>, // list of swap pools(eg. list of junoswap pools)
    pub staking_addr: Addr, // contract address, to where we can stake the LP token

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
pub struct PendingInfo {
    pub typ: String, // type of pending transaction ('typ', because 'type' is protected keyword in Rust...)
    pub accounts_address: Addr, // Addr of org. sending Accounts SC
    pub beneficiary: Option<Addr>, // return to the beneficiary
    pub fund: Option<u64>, // return to the active fund
    pub amount: Uint128,
}

pub const PENDING: Map<&[u8], PendingInfo> = Map::new("pending");
pub const REMNANTS: Map<String, Uint128> = Map::new("remnants");
