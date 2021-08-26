use crate::state::{ACCOUNTS, CONFIG, ENDOWMENT};
use angel_core::responses::accounts::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        owner: config.owner.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
        deposit_approved: config.deposit_approved,
        withdraw_approved: config.withdraw_approved,
    };
    Ok(res)
}

pub fn query_account_details(
    deps: Deps,
    account_type: String,
) -> StdResult<AccountDetailsResponse> {
    // this fails if no account is found
    let account = ACCOUNTS.load(deps.storage, account_type.clone())?;
    let details = AccountDetailsResponse {
        account_type,
        ust_balance: account.ust_balance,
    };
    Ok(details)
}

pub fn query_account_list(deps: Deps) -> StdResult<AccountListResponse> {
    let list = AccountListResponse {
        locked_account: query_account_details(deps, "locked".to_string())?,
        liquid_account: query_account_details(deps, "liquid".to_string())?,
    };
    Ok(list)
}

pub fn query_endowment_details(deps: Deps) -> StdResult<EndowmentDetailsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENT.load(deps.storage)?;
    Ok(EndowmentDetailsResponse {
        owner: endowment.owner,
        beneficiary: endowment.beneficiary,
        name: endowment.name,
        description: endowment.description,
        withdraw_before_maturity: endowment.withdraw_before_maturity,
        maturity_time: endowment.maturity_time,
        maturity_height: endowment.maturity_height,
        split_to_liquid: endowment.split_to_liquid,
        strategies: endowment.strategies,
        // total_funds: Uint128 // locked total + liquid total
        // total_donations: Uint128 // all donations received
    })
}
