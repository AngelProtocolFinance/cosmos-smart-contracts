use cosmwasm_std::{Deps, Uint128};
use cw20::{Denom, TokenInfoResponse};

use angel_core::messages::vault::WasmSwapQueryMsg;
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse, InfoResponse};

use crate::state::{BALANCES, CONFIG, TOKEN_INFO};

pub fn query_balance(deps: Deps, id: u32) -> Uint128 {
    BALANCES.load(deps.storage, id).unwrap_or_default()
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
    let config = CONFIG.load(deps.storage).unwrap();
    ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        keeper: config.keeper.to_string(),
        pool_addr: config.pool_addr.to_string(),
        input_denoms: config.input_denoms,
        pool_lp_token_addr: config.pool_lp_token_addr.to_string(),
        staking_addr: config.staking_addr.to_string(),
        last_harvest: config.last_harvest,
    }
}

pub fn query_total_balance(deps: Deps) -> BalanceResponse {
    let config = CONFIG.load(deps.storage).unwrap();
    BalanceResponse {
        balance: config.total_shares,
    }
}

pub fn query_exchange_rate(deps: Deps, input_denom: Denom) -> ExchangeRateResponse {
    let config = CONFIG.load(deps.storage).unwrap();
    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(config.pool_addr, &WasmSwapQueryMsg::Info {})
        .unwrap();
    todo!("Implement the following query response")
    // ExchangeRateResponse {
    //     exchange_rate: Decimal256::zero(),
    //     yield_token_supply: Uint256::zero(),
    // }
}
