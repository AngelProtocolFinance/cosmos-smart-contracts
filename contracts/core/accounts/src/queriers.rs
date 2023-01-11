use crate::state::{Allowances, Endowment, ALLOWANCES, CONFIG, ENDOWMENTS, STATES};
use angel_core::responses::accounts::*;
use angel_core::structs::EndowmentBalanceResponse;
use angel_core::utils::vault_endowment_balance;
use cosmwasm_std::{Deps, Order, StdResult};
use cw2::get_contract_version;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: format!(
            "{}-{}",
            get_contract_version(deps.storage)?.contract,
            get_contract_version(deps.storage)?.version
        ),
        registrar_contract: config.registrar_contract.to_string(),
        next_account_id: config.next_account_id,
        max_general_category_id: config.max_general_category_id,
    })
}

pub fn query_state(deps: Deps, id: u32) -> StdResult<StateResponse> {
    let state = STATES.load(deps.storage, id)?;

    Ok(StateResponse {
        donations_received: state.donations_received,
        balances: state.balances,
        closing_endowment: state.closing_endowment,
        closing_beneficiary: state.closing_beneficiary,
    })
}

pub fn query_endowment_balance(deps: Deps, id: u32) -> StdResult<EndowmentBalanceResponse> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    let state = STATES.load(deps.storage, id)?;

    // setup the basic response object w/ account's balances locked & liquid (held by this contract)
    let tokens_on_hand = state.balances;

    // process all one-off vaults
    let mut invested_locked = vec![];
    for vault in endowment.oneoff_vaults.locked.into_iter() {
        let vault_bal = vault_endowment_balance(deps, vault.clone().to_string(), id);
        invested_locked.push((vault.to_string(), vault_bal));
    }
    let mut invested_liquid = vec![];
    for vault in endowment.oneoff_vaults.liquid.into_iter() {
        let vault_bal = vault_endowment_balance(deps, vault.clone().to_string(), id);
        invested_liquid.push((vault.to_string(), vault_bal));
    }

    Ok(EndowmentBalanceResponse {
        tokens_on_hand,
        invested_locked,
        invested_liquid,
    })
}

pub fn query_endowment_by_proposal_link(
    deps: Deps,
    proposal_link: u64,
) -> StdResult<EndowmentDetailsResponse> {
    let endowments: Vec<Endowment> = ENDOWMENTS
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| e.as_ref().unwrap().1.proposal_link == Some(proposal_link))
        .map(|item| item.unwrap().1)
        .collect();
    if endowments.len() != 1 {
        return Err(cosmwasm_std::StdError::NotFound {
            kind: "endowment".to_string(),
        });
    }
    let Endowment {
        owner,
        name,
        categories,
        tier,
        endow_type,
        logo,
        image,
        status,
        deposit_approved,
        withdraw_approved,
        maturity_time,
        strategies,
        oneoff_vaults,
        rebalance,
        kyc_donors_only,
        pending_redemptions,
        proposal_link,
        referral_id,
    } = endowments[0].clone();

    Ok(EndowmentDetailsResponse {
        owner,
        name,
        categories,
        tier,
        endow_type,
        logo,
        image,
        status,
        deposit_approved,
        withdraw_approved,
        maturity_time,
        strategies,
        oneoff_vaults,
        rebalance,
        kyc_donors_only,
        pending_redemptions,
        proposal_link,
        referral_id,
    })
}

pub fn query_endowment_details(deps: Deps, id: u32) -> StdResult<EndowmentDetailsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    Ok(EndowmentDetailsResponse {
        owner: endowment.owner,
        name: endowment.name,
        tier: endowment.tier,
        categories: endowment.categories,
        logo: endowment.logo,
        image: endowment.image,
        status: endowment.status,
        endow_type: endowment.endow_type,
        maturity_time: endowment.maturity_time,
        strategies: endowment.strategies,
        oneoff_vaults: endowment.oneoff_vaults,
        rebalance: endowment.rebalance,
        kyc_donors_only: endowment.kyc_donors_only,
        deposit_approved: endowment.deposit_approved,
        withdraw_approved: endowment.withdraw_approved,
        pending_redemptions: endowment.pending_redemptions,
        proposal_link: endowment.proposal_link,
        referral_id: endowment.referral_id,
    })
}

pub fn query_allowances(deps: Deps, id: u32, spender: String) -> StdResult<Allowances> {
    let endowment = ENDOWMENTS.load(deps.storage, id)?;
    let spender = deps.api.addr_validate(&spender)?;
    let allowances = ALLOWANCES.may_load(deps.storage, (&endowment.owner, &spender))?;
    Ok(allowances.unwrap_or_default())
}
