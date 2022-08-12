use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw20::Denom;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub registrar_contract: Addr,
    pub keeper: Addr,

    pub last_harvest: u64,
    pub last_harvest_fx: Option<Uint128>,
    pub harvest_to_liquid: Decimal,

    pub pool_addr: Addr, // swap pool address(eg. JunoSwap USDC-JUNO pool address)
    pub input_denoms: Vec<Denom>, // swap input tokens(denoms) list
    pub pool_lp_token_addr: Addr, // swap lp token address
    pub routes: Vec<Addr>, // list of swap pools(eg. list of junoswap pools)
    pub staking_addr: Addr, // contract address, to where we can stake the LP token
    pub output_token_denom: Denom, // denom of output token to be used when withdraw/claim

    pub total_assets: Uint128, // total value of assets deposited from endowments (in usdc/usd)
    pub total_shares: Uint128, // total amount of minted vault tokens
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PendingInfo {
    pub typ: String, // type of pending transaction ('typ', because 'type' is protected keyword in Rust...)
    pub endowment_id: String, // ID of org. sending Accounts SC
    pub beneficiary: Addr, // return to the beneficiary
    pub amount: Uint128,
}

pub const PENDING: Map<(&str, u64), PendingInfo> = Map::new("pending");
pub const REMNANTS: Map<String, Uint128> = Map::new("remnants");
