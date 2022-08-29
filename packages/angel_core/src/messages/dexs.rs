use cosmwasm_std::{Decimal, Uint128};
use cw20::Denom;
use cw_asset::Asset;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// JUNOSWAP SPECIFIC MESSAGES/RESPONCES/QUERIES
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenSelect {
    Token1,
    Token2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JunoSwapExecuteMsg {
    Swap {
        input_token: TokenSelect,
        input_amount: Uint128,
        min_output: Uint128,
        expiration: Option<Expiration>,
    },
    /// Chained swap converting A -> B and B -> C by leveraging two swap contracts
    PassThroughSwap {
        output_amm_address: String,
        input_token: TokenSelect,
        input_token_amount: Uint128,
        output_min_token: Uint128,
        expiration: Option<Expiration>,
    },
    SwapAndSendTo {
        input_token: TokenSelect,
        input_amount: Uint128,
        recipient: String,
        min_token: Uint128,
        expiration: Option<Expiration>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JunoSwapQueryMsg {
    Info {},
    Balance { address: String },
    Token1ForToken2Price { token1_amount: Uint128 },
    Token2ForToken1Price { token2_amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub token1_reserve: Uint128,
    pub token1_denom: Denom,
    pub token2_reserve: Uint128,
    pub token2_denom: Denom,
    pub lp_token_supply: Uint128,
    pub lp_token_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token1ForToken2PriceResponse {
    pub token2_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token2ForToken1PriceResponse {
    pub token1_amount: Uint128,
}

/// LOOP FINANCE SPECIFIC MESSAGES/RESPONCES/QUERIES
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoopExecuteMsg {
    Swap {
        offer_asset: Asset,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
    },
    IncreaseAllowance {
        amount: Uint128,
        spender: String,
    },
    ProvideLiquidity {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoopQueryMsg {
    // Get pool info for a pair
    Pool {},
    // Get a Pair's info
    Pair {},
    // simulate a swap
    Simulation { offer_asset: Asset },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}
