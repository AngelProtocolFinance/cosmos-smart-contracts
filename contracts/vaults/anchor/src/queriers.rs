use crate::config;
use crate::config::{BALANCES, TOKEN_INFO};
use angel_core::responses::vault::VaultConfigResponse;
use cosmwasm_std::{Deps, Uint128};
use cw20::TokenInfoResponse;

pub fn query_balance(deps: Deps, endowment_id: u32) -> Uint128 {
    BALANCES
        .load(deps.storage, &endowment_id)
        .unwrap_or_else(|_| Uint128::zero())
}

// pub fn query_tvl(deps: Deps,  env: Env) -> TvlResponse {
//     let mut totals: (u64, u64) = (0, 0);

//     let accounts: Result<Vec<_>, _> = BALANCES
//             .keys(deps.storage, None, None, Order::Ascending)
//             .map(String::from_utf8)
//             .collect();
//     for account in accounts.unwrap().iter() {
//         let account_address = deps.api.addr_validate(account)?;
//         let balances = BALANCES.load(deps.storage, &account_address)?;
//         totals[0] += balances.locked_balance.get_token_amount(env.contract.address.clone())
//         totals[1] += balances.liquid_balance.get_token_amount(env.contract.address.clone())
//     }

//     TvlResponse {
//         locked: totals[0],
//         liquid: totals[1],
//     };
// }

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
        harvest_to_liquid: config.harvest_to_liquid,
    }
}
