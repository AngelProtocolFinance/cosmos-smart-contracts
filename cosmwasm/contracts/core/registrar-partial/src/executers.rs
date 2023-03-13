use crate::state::{CONFIG, FEES, NETWORK_CONNECTIONS, STRATEGIES};
use angel_core::errors::core::ContractError;
use angel_core::msgs::registrar::*;
use angel_core::structs::{NetworkInfo, StrategyApprovalState, StrategyParams};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner_addr = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed owner
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner_addr;
        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_owner"))
}

pub fn update_fees(
    deps: DepsMut,
    info: MessageInfo,
    fees: Vec<(String, Decimal)>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    for fee in fees.iter() {
        // check percentage is valid
        percentage_checks(fee.1)?;
        // save|update fee set in storage
        FEES.save(deps.storage, &fee.0, &fee.1)?;
    }
    Ok(Response::new().add_attribute("action", "update_fees"))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed configs
    config.treasury = deps
        .api
        .addr_validate(&msg.treasury.unwrap_or_else(|| config.treasury.to_string()))?;
    config.rebalance = match msg.rebalance {
        Some(details) => details,
        None => config.rebalance,
    };
    let max = match msg.split_max {
        Some(max) => percentage_checks(max),
        None => Ok(config.split_to_liquid.max),
    };
    let min = match msg.split_min {
        Some(min) => percentage_checks(min),
        None => Ok(config.split_to_liquid.min),
    };
    let default = match msg.split_default {
        Some(default) => percentage_checks(default),
        None => Ok(config.split_to_liquid.default),
    };
    config.split_to_liquid = split_checks(max.unwrap(), min.unwrap(), default.unwrap()).unwrap();
    config.accepted_tokens = match msg.accepted_tokens {
        Some(tokens) => tokens,
        None => config.accepted_tokens,
    };
    config.axelar_gateway = match msg.axelar_gateway {
        Some(gateway) => gateway,
        None => config.axelar_gateway,
    };
    config.axelar_ibc_channel = match msg.axelar_ibc_channel {
        Some(channel) => channel,
        None => config.axelar_ibc_channel,
    };
    config.axelar_chain_id = match msg.axelar_chain_id {
        Some(chain) => chain,
        None => config.axelar_chain_id,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn strategy_add(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_key: String,
    strategy: StrategyParams,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // check that an approved network connection was set for the StrategyParams
    let _network = NETWORK_CONNECTIONS.load(deps.storage, &strategy.chain)?;

    // check that the vault does not already exist for a given address in storage
    // add new strategy if all looks good
    STRATEGIES.update(
        deps.storage,
        &strategy_key.as_bytes(),
        |existing| match existing {
            Some(_) => Err(ContractError::StrategyAlreadyExists {}),
            None => Ok(strategy),
        },
    )?;
    Ok(Response::default())
}

pub fn strategy_remove(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_key: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // remove the Strategy from storage
    STRATEGIES.remove(deps.storage, &strategy_key.as_bytes());
    Ok(Response::default())
}

pub fn strategy_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    strategy_key: String,
    approval_state: StrategyApprovalState,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given strategy in Storage
    let mut strategy = STRATEGIES.load(deps.storage, strategy_key.as_bytes())?;

    // update strategy with approval state from passed arg
    strategy.approval_state = approval_state;
    STRATEGIES.save(deps.storage, strategy_key.as_bytes(), &strategy)?;

    Ok(Response::default())
}

pub fn update_network_connections(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    chain_id: String,
    network_info: NetworkInfo,
    action: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    if action == *"post" {
        // Add/Overwrite the network_info to NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.save(deps.storage, &chain_id, &network_info)?;
    } else if action == *"delete" {
        // Remove the network_info from NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.remove(deps.storage, &chain_id);
    } else {
        return Err(ContractError::InvalidInputs {});
    }

    Ok(Response::default().add_attribute("action", "update_network_connections"))
}
