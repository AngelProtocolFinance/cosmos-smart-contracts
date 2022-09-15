use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cw20::{Denom, Expiration};
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

    pub next_pending_id: u32, // (Incrementing) ID used for indexing the PendingInfo
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PendingInfo {
    pub typ: String, // type of pending transaction ('typ', because 'type' is protected keyword in Rust...)
    pub endowment_id: u32, // ID of org. sending Accounts SC
    pub beneficiary: Addr, // return to the beneficiary
    pub amount: Uint128,
    pub release_at: Expiration,
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

pub const PENDING: Map<u32, PendingInfo> = Map::new("pending");
pub const REMNANTS: Map<String, Uint128> = Map::new("remnants");

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const BALANCES: Map<u32, Uint128> = Map::new("balance");
