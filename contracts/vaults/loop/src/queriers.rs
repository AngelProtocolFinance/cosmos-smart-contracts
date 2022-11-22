use cosmwasm_std::{Deps, Uint128};
use cw20::{BalanceResponse, TokenInfoResponse};

use angel_core::responses::vault::{ConfigResponse, StateResponse};

use crate::state::{Config, APTAX, BALANCES, CONFIG, STATE, TOKEN_INFO};

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
    let config: Config = CONFIG.load(deps.storage).unwrap();
    ConfigResponse {
        owner: config.owner.to_string(),
        acct_type: config.acct_type,
        sibling_vault: config.sibling_vault.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        keeper: config.keeper.to_string(),
        tax_collector: config.tax_collector.to_string(),

        native_token: config.native_token.to_string(),
        lp_token: config.lp_token.to_string(),
        lp_pair_token0: config.lp_pair_token0.to_string(),
        lp_pair_token1: config.lp_pair_token1.to_string(),
        lp_reward_token: config.lp_reward_token.to_string(),

        reward_to_native_rotue: config.reward_to_native_route,
        native_to_lp0_route: config.native_to_lp0_route,
        native_to_lp1_route: config.native_to_lp1_route,

        lp_factory_contract: config.lp_factory_contract.to_string(),
        lp_staking_contract: config.lp_staking_contract.to_string(),
        lp_pair_contract: config.lp_pair_contract.to_string(),

        minimum_initial_deposit: config.minimum_initial_deposit.to_string(),
    }
}

pub fn query_state(deps: Deps) -> StateResponse {
    let state = STATE.load(deps.storage).unwrap();
    StateResponse {
        total_lp_amount: state.total_lp_amount.to_string(),
        total_shares: state.total_shares.to_string(),
    }
}

pub fn query_total_balance(deps: Deps) -> BalanceResponse {
    let info = TOKEN_INFO.load(deps.storage).unwrap();
    BalanceResponse {
        balance: info.total_supply,
    }
}

pub fn query_ap_tax_balance(deps: Deps) -> BalanceResponse {
    let ap_tax = APTAX.load(deps.storage).unwrap();
    BalanceResponse { balance: ap_tax }
}
