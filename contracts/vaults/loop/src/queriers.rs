use cosmwasm_std::Deps;
use cw20::{BalanceResponse, Denom, TokenInfoResponse};

use angel_core::responses::vault::{ConfigResponse, ExchangeRateResponse};

use crate::state::{BALANCES, CONFIG, TOKEN_INFO};

pub fn query_balance(deps: Deps, id: u32) -> BalanceResponse {
    let balance = BALANCES.load(deps.storage, id).unwrap_or_default();
    BalanceResponse { balance }
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
        loop_factory_contract: config.loop_factory_contract.to_string(),
        loop_farming_contract: config.loop_farming_contract.to_string(),
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
    todo!("Implement the following query response")
    // ExchangeRateResponse {
    //     exchange_rate: Decimal256::zero(),
    //     yield_token_supply: Uint256::zero(),
    // }
}
