use crate::state::{CONFIG, CONFIG_EXTENSION, FEES, NETWORK_CONNECTIONS, STRATEGIES};
use angel_core::msgs::registrar::*;
use cosmwasm_std::{Decimal, Deps, StdResult};
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
        treasury: config.treasury.to_string(),
        rebalance: config.rebalance,
        split_to_liquid: config.split_to_liquid,
        accepted_tokens: config.accepted_tokens,
        axelar_gateway: config.axelar_gateway,
        axelar_ibc_channel: config.axelar_ibc_channel,
        axelar_chain_id: config.axelar_chain_id,
    })
}

pub fn query_config_extension(deps: Deps) -> StdResult<ConfigExtensionResponse> {
    let extension = CONFIG_EXTENSION.load(deps.storage)?;
    Ok(ConfigExtensionResponse {
        accounts_contract: extension.accounts_contract.map(|addr| addr.to_string()),
        index_fund: extension.index_fund_contract.map(|addr| addr.to_string()),
        cw3_code: extension.cw3_code,
        cw4_code: extension.cw4_code,
        subdao_gov_code: extension.subdao_gov_code,
        subdao_cw20_token_code: extension.subdao_cw20_token_code,
        subdao_bonding_token_code: extension.subdao_bonding_token_code,
        subdao_cw900_code: extension.subdao_cw900_code,
        subdao_distributor_code: extension.subdao_distributor_code,
        donation_match_code: extension.donation_match_code,
        halo_token: match extension.halo_token {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
        halo_token_lp_contract: match extension.halo_token_lp_contract {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
        gov_contract: match extension.gov_contract {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
        donation_match_charites_contract: match extension.donation_match_charites_contract {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
        collector_addr: extension
            .collector_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "".to_string()),
        charity_shares_contract: extension
            .charity_shares_contract
            .map(|addr| addr.to_string()),
        swap_factory: extension.swap_factory.map(|addr| addr.to_string()),
        applications_review: extension.applications_review.to_string(),
        swaps_router: extension.swaps_router.map(|addr| addr.to_string()),
        accounts_settings_controller: match extension.accounts_settings_controller {
            Some(addr) => Some(addr.to_string()),
            None => None,
        },
    })
}

pub fn query_strategy(deps: Deps, strategy_key: String) -> StdResult<StrategyDetailResponse> {
    // this fails if no vault is found
    Ok(StrategyDetailResponse {
        strategy: STRATEGIES.load(deps.storage, strategy_key.as_bytes())?,
    })
}

pub fn query_network_connection(
    deps: Deps,
    chain_id: String,
) -> StdResult<NetworkConnectionResponse> {
    Ok(NetworkConnectionResponse {
        chain: chain_id.clone(),
        network_connection: NETWORK_CONNECTIONS.load(deps.storage, &chain_id)?,
    })
}

pub fn query_fee(deps: Deps, name: String) -> StdResult<Decimal> {
    Ok(FEES.load(deps.storage, &name).unwrap_or(Decimal::zero()))
}
