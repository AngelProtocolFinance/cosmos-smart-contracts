use angel_core::curves::DecimalPlaces;
use angel_core::messages::subdao_token::CurveType;
use cosmwasm_std::{Addr, Uint128};
use cw_controllers::Claims;
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Supply is dynamic and tracks the current supply of staked and ERC20 tokens.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveState {
    /// reserve is how many native tokens exist bonded to the validator
    pub reserve: Uint128,
    /// supply is how many tokens this contract has issued
    pub supply: Uint128,
    // the denom/address of the reserve
    pub reserve_denom: String,
    // how to normalize reserve and supply
    pub decimals: DecimalPlaces,
}

impl CurveState {
    pub fn new(reserve_denom: String, decimals: DecimalPlaces) -> Self {
        CurveState {
            reserve: Uint128::zero(),
            supply: Uint128::zero(),
            reserve_denom,
            decimals,
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// This is the unbonding period of CS tokens
    /// We need this to only allow claims to be redeemed after this period
    pub unbonding_period: Duration,
}

pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const CLAIMS: Claims = Claims::new("claims");
pub const CURVE_STATE: Item<CurveState> = Item::new("curve_state");
pub const CURVE_TYPE: Item<CurveType> = Item::new("curve_type");
pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const CONFIG: Item<Config> = Item::new("config");
