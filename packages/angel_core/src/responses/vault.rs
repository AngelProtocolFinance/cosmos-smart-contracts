use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub input_denom: String,
    pub yield_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    pub exchange_rate: Decimal256,
    pub yield_token_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultConfigResponse {
    pub owner: String,
    pub registrar_contract: String,
    pub moneymarket: String,
    pub input_denom: String,
    pub yield_token: String,
    pub tax_per_block: Decimal,
    pub last_harvest: u64,
    pub treasury_withdraw_threshold: Uint128,
}
