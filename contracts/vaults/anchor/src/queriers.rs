use crate::config;
use crate::config::{BALANCES, TOKEN_INFO};
use angel_core::responses::vault::VaultConfigResponse;
use angel_core::structs::{BalanceInfo, BalanceResponse};
use cosmwasm_std::Deps;
use cw20::TokenInfoResponse;

pub fn query_balance(deps: Deps, address: String) -> BalanceResponse {
    let address = deps.api.addr_validate(&address).unwrap();
    let balances = BALANCES
        .load(deps.storage, &address)
        .unwrap_or(BalanceInfo::default());
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

pub fn query_vault_config(deps: Deps) -> VaultConfigResponse {
    let config = config::read(deps.storage).unwrap();
    VaultConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        moneymarket: config.moneymarket.to_string(),
        input_denom: config.input_denom,
        yield_token: config.yield_token.to_string(),
        tax_per_block: config.tax_per_block,
        last_harvest: config.last_harvest,
        harvest_to_liquid: config.harvest_to_liquid,
    }
}
