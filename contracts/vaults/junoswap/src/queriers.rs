use crate::state::{self, CONFIG};
use crate::wasmswap::{self, InfoResponse};
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};
use cosmwasm_std::{Deps, Uint128};
use cw20::{BalanceResponse, Denom, TokenInfoResponse};
use cw20_base::state::TOKEN_INFO;

pub fn query_balance(deps: Deps, address: String) -> BalanceResponse {
    cw20_base::contract::query_balance(deps, address).unwrap_or(BalanceResponse {
        balance: Uint128::zero(),
    })
}

pub fn query_token_info(deps: Deps) -> TokenInfoResponse {
    let info = TOKEN_INFO.load(deps.storage).unwrap();
    TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    }
}

pub fn query_config(deps: Deps) -> ConfigResponse {
    let config = state::read(deps.storage).unwrap();
    ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        pool_addr: config.pool_addr.to_string(),
        input_denoms: config.input_denoms,
        pool_lp_token_addr: config.pool_lp_token_addr.to_string(),
        staking_addr: config.staking_addr.to_string(),
        last_harvest: config.last_harvest,
    }
}

pub fn query_exchange_rate(deps: Deps, input_denom: Denom) -> ExchangeRateResponse {
    let config = CONFIG.load(deps.storage).unwrap();
    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(config.pool_addr, &wasmswap::QueryMsg::Info {})
        .unwrap();
    todo!("Implement the following query response")
    // ExchangeRateResponse {
    //     exchange_rate: Decimal256::zero(),
    //     yield_token_supply: Uint256::zero(),
    // }
}
