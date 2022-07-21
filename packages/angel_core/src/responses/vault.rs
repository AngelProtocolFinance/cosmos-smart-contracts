use cosmwasm_std::{Decimal, Decimal256, Uint256};
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    pub exchange_rate: Decimal256,
    pub yield_token_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub registrar_contract: String,
    pub target: String,
    pub input_denoms: Vec<Denom>,
    pub yield_token: String,
    pub last_harvest: u64,
    pub harvest_to_liquid: Decimal,
}
