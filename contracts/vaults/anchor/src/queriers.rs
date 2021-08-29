use crate::config::{LIQUID_BALANCES, LOCKED_BALANCES, TOKEN_INFO};
use angel_core::responses::vault::VaultBalanceResponse;
use cosmwasm_std::Deps;
use cw20::TokenInfoResponse;

pub fn query_balance(deps: Deps, address: String) -> VaultBalanceResponse {
    let info = TOKEN_INFO.load(deps.storage).unwrap();
    let address = deps.api.addr_validate(&address).unwrap();
    VaultBalanceResponse {
        locked: LOCKED_BALANCES
            .may_load(deps.storage, &address)
            .unwrap()
            .unwrap_or_default(),
        liquid: LIQUID_BALANCES
            .may_load(deps.storage, &address)
            .unwrap()
            .unwrap_or_default(),
        denom: info.symbol,
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
