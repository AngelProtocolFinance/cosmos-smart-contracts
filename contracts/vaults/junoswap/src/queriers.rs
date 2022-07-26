use crate::config::{self, CONFIG};
use crate::config::{BALANCES, TOKEN_INFO};
use crate::wasmswap::{self, InfoResponse};
use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};
use angel_core::structs::{BalanceInfo, BalanceResponse};
use cosmwasm_std::Deps;
use cw20::{Denom, TokenInfoResponse};

pub fn query_balance(deps: Deps, address: String) -> BalanceResponse {
    let address = deps.api.addr_validate(&address).unwrap();
    let balances = BALANCES
        .load(deps.storage, &address)
        .unwrap_or_else(|_| BalanceInfo::default());
    BalanceResponse {
        locked_native: balances.clone().locked_balance.native,
        liquid_native: balances.clone().liquid_balance.native,
        locked_cw20: balances.locked_balance.cw20_list(),
        liquid_cw20: balances.liquid_balance.cw20_list(),
    }
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
    let config = config::read(deps.storage).unwrap();
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
