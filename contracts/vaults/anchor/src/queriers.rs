use crate::config::{BALANCES, TOKEN_INFO};
use cosmwasm_std::Deps;
use cw20::{BalanceResponse, MinterResponse, TokenInfoResponse};

pub fn query_balance(deps: Deps, address: String) -> BalanceResponse {
    let address = deps.api.addr_validate(&address).unwrap();
    let balance = BALANCES
        .may_load(deps.storage, &address)
        .unwrap()
        .unwrap_or_default();
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

pub fn query_minter(deps: Deps) -> Option<MinterResponse> {
    let meta = TOKEN_INFO.load(deps.storage).unwrap();
    match meta.mint {
        Some(m) => Some(MinterResponse {
            minter: m.minter.into(),
            cap: m.cap,
        }),
        None => None,
    }
}
