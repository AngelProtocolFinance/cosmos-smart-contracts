use cosmwasm_std::{DepsMut, StdResult, Uint128};
use cw20::Denom;

/// Returns a `Denom` balance for a specific account.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **account_addr** is an object of type [`String`].
///
/// * **denom** is an object of type [`Denom`] used to specify the denomination used to return the balance.
pub fn query_denom_balance(deps: &DepsMut, denom: &Denom, account_addr: String) -> Uint128 {
    match denom {
        Denom::Native(denom) => {
            query_balance(&deps, account_addr, denom.to_string()).unwrap_or(Uint128::zero())
        }
        Denom::Cw20(contract_addr) => {
            query_token_balance(&deps, contract_addr.to_string(), account_addr)
                .unwrap_or(Uint128::zero())
        }
    }
}

/// Returns a native token's balance for a specific account.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **account_addr** is an object of type [`String`].
///
/// * **denom** is an object of type [`String`] used to specify the denomination used to return the balance (e.g uluna).
pub fn query_balance(deps: &DepsMut, account_addr: String, denom: String) -> StdResult<Uint128> {
    Ok(deps
        .querier
        .query_balance(account_addr, denom)
        .map(|c| c.amount)
        .unwrap_or(Uint128::zero()))
}

/// Returns a token balance for an account.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **contract_addr** is an object of type [`String`]. This is the token contract for which we return a balance.
///
/// * **account_addr** is an object of type [`String`] for which we query the token balance for.
pub fn query_token_balance(
    deps: &DepsMut,
    contract_addr: String,
    account_addr: String,
) -> StdResult<Uint128> {
    // load balance from the token contract
    let res: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        contract_addr,
        &cw20::Cw20QueryMsg::Balance {
            address: account_addr,
        },
    )?;
    Ok(res.balance)
}
