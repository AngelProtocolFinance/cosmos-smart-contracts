use crate::msg::VaultConfigResponse;
use crate::state::{BALANCES, CONFIG, TOKEN_INFO};
use cosmwasm_std::Deps;
use cosmwasm_std::Uint128;
use cw20::TokenInfoResponse;

pub fn query_balance(deps: Deps, endowment_id: u32) -> Uint128 {
    BALANCES
        .load(deps.storage, endowment_id)
        .unwrap_or_else(|_| Uint128::zero())
}

pub fn query_total_balance(deps: Deps) -> Uint128 {
    let info = TOKEN_INFO.load(deps.storage).unwrap();
    info.total_supply
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

pub fn query_vault_config(deps: Deps) -> VaultConfigResponse {
    let config = CONFIG.load(deps.storage).unwrap();
    VaultConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        harvest_to_liquid: config.harvest_to_liquid,
    }
}
