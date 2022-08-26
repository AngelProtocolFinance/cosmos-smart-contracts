use cosmwasm_std::{Decimal256, Uint256};
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
    pub keeper: String,
    pub pair_contract: String,
    pub lp_staking_contract: String,
}
