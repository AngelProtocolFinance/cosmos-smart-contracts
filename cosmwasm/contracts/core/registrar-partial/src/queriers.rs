use crate::state::{CONFIG, FEES, NETWORK_CONNECTIONS, STRATEGIES};
use angel_core::msgs::registrar::{
    ConfigResponse, NetworkConnectionResponse, StrategyDetailResponse,
};
use cosmwasm_std::{Decimal, Deps, StdError, StdResult};
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

pub fn query_strategy(deps: Deps, strategy_key: String) -> StdResult<StrategyDetailResponse> {
    // check the strategy key provided is correct length of bytes (4)
    let strat_bytes = strategy_key.as_bytes();
    if strat_bytes.len() != 4 {
        return Err(StdError::InvalidDataSize {
            expected: 4_u64,
            actual: strat_bytes.len() as u64,
        });
    }
    // this fails if no strategy is found
    Ok(StrategyDetailResponse {
        strategy: STRATEGIES.load(deps.storage, &strat_bytes)?,
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
