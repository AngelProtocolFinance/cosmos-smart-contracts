use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub registrar_contract: Addr,
    pub keeper: Addr,

    pub lp_staking_contract: Addr, // loopswap farming contract address
    pub pair_contract: Addr,       // loopswap pair contract address
    pub loop_token: Addr,          // LOOP token address(Atm, LOOP is loopswap farming reward token)

    pub total_lp_amount: Uint128, // total amount of LP tokens in this `vault`
    pub total_shares: Uint128,    // total amount of minted vault tokens
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

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const BALANCES: Map<u32, Uint128> = Map::new("balance");
