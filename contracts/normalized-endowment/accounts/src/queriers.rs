use crate::state::{CONFIG, ENDOWMENT, PROFILE, STATE};
use angel_core::responses::accounts::*;
use angel_core::structs::BalanceResponse;
use angel_core::{messages::vault::QueryMsg as VaultQuerier, structs::TransactionRecord};
use cosmwasm_std::{to_binary, Addr, Deps, Env, QueryRequest, StdError, StdResult, WasmQuery};
use cw2::get_contract_version;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        version: get_contract_version(deps.storage)?.contract,
        registrar_contract: config.registrar_contract.to_string(),
        deposit_approved: config.deposit_approved,
        withdraw_approved: config.withdraw_approved,
        last_earnings_harvest: config.last_earnings_harvest,
        last_harvest_fx: config
            .last_harvest_fx
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
    })
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(StateResponse {
        donations_received: state.donations_received,
    })
}

pub fn query_account_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    let endowment = ENDOWMENT.load(deps.storage)?;
    // setup the basic response object w/ accounts' balances
    let mut balances = BalanceResponse::default();
    // add stategies' balances to the object
    for strategy in endowment.strategies {
        let mut strategy_balance: BalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: strategy.vault.to_string(),
                msg: to_binary(&VaultQuerier::Balance {
                    address: env.contract.address.to_string(),
                })?,
            }))?;
        balances
            .locked_cw20
            .append(&mut strategy_balance.locked_cw20);
        balances
            .liquid_cw20
            .append(&mut strategy_balance.liquid_cw20);
        balances
            .locked_native
            .append(&mut strategy_balance.locked_native);
        balances
            .liquid_native
            .append(&mut strategy_balance.liquid_native);
    }

    Ok(balances)
}

pub fn query_endowment_details(deps: Deps) -> StdResult<EndowmentDetailsResponse> {
    // this fails if no account is found
    let endowment = ENDOWMENT.load(deps.storage)?;
    Ok(EndowmentDetailsResponse {
        owner: endowment.owner,
        name: endowment.name,
        description: endowment.description,
        withdraw_before_maturity: endowment.withdraw_before_maturity,
        maturity_time: endowment.maturity_time,
        maturity_height: endowment.maturity_height,
        strategies: endowment.strategies,
        rebalance: endowment.rebalance,
        donation_match_contract_addr: endowment
            .donation_matching_contract
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "".to_string()),
    })
}

pub fn query_profile(deps: Deps) -> StdResult<ProfileResponse> {
    let profile = PROFILE.load(deps.storage)?;
    Ok(ProfileResponse {
        name: profile.name,
        overview: profile.overview,
        un_sdg: profile.un_sdg,
        tier: profile.tier,
        logo: profile.logo,
        image: profile.image,
        url: profile.url,
        registration_number: profile.registration_number,
        country_of_origin: profile.country_of_origin,
        street_address: profile.street_address,
        contact_email: profile.contact_email,
        social_media_urls: profile.social_media_urls,
        number_of_employees: profile.number_of_employees,
        average_annual_budget: profile.average_annual_budget,
        annual_revenue: profile.annual_revenue,
        charity_navigator_rating: profile.charity_navigator_rating,
    })
}

pub fn query_transactions(
    deps: Deps,
    sender: Option<String>,
    recipient: Option<String>,
    denom: Option<String>,
) -> StdResult<TxRecordsResponse> {
    let txs = STATE.load(deps.storage)?.transactions;

    let txs = match sender {
        Some(addr) => {
            if deps.api.addr_validate(&addr).is_err() {
                return Err(StdError::GenericErr {
                    msg: "Invalid sender address".to_string(),
                });
            }
            txs.into_iter()
                .filter(|tx| tx.sender == addr)
                .collect::<Vec<TransactionRecord>>()
        }
        None => txs,
    };

    let txs = match recipient {
        Some(addr) => {
            if deps.api.addr_validate(&addr).is_err() {
                return Err(StdError::GenericErr {
                    msg: "Invalid recipient address".to_string(),
                });
            }
            txs.into_iter()
                .filter(|tx| {
                    *tx.recipient
                        .as_ref()
                        .unwrap_or(&Addr::unchecked("anonymous"))
                        == addr
                })
                .collect::<Vec<TransactionRecord>>()
        }
        None => txs,
    };

    let txs = match denom {
        Some(denom) => txs
            .into_iter()
            .filter(|tx| tx.denom == denom)
            .collect::<Vec<TransactionRecord>>(),
        None => txs,
    };

    Ok(TxRecordsResponse { txs })
}

pub fn query_endowment_fees(deps: Deps) -> StdResult<EndowmentFeesResponse> {
    let endowment = ENDOWMENT.load(deps.storage)?;
    Ok(EndowmentFeesResponse {
        earnings_fee: endowment.earnings_fee,
        deposit_fee: endowment.deposit_fee,
        withdraw_fee: endowment.withdraw_fee,
        aum_fee: endowment.aum_fee,
    })
}
