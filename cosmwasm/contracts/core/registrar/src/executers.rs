use crate::state::{CONFIG, CONFIG_EXTENSION, FEES, NETWORK_CONNECTIONS, STRATEGIES};
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
    config.split_to_liquid = match msg.split_to_liquid {
        Some(splits) => split_checks(splits.max, splits.min, splits.default).unwrap(),
        None => config.split_to_liquid,
    };
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

pub fn update_config_extension(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigExtensionMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut extension = CONFIG_EXTENSION.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed configs
    extension.applications_review = match msg.applications_review {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => extension.applications_review,
    };
    extension.accounts_contract = match msg.accounts_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => extension.accounts_contract,
    };
    extension.swaps_router = match msg.swaps_router {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => extension.swaps_router,
    };
    extension.cw3_code = match msg.cw3_code {
        Some(v) => Some(v),
        None => extension.cw3_code,
    };
    extension.cw4_code = match msg.cw4_code {
        Some(v) => Some(v),
        None => extension.cw4_code,
    };
    extension.charity_shares_contract = match msg.charity_shares_contract {
        Some(contract_addr) => Some(deps.api.addr_validate(&contract_addr)?),
        None => extension.charity_shares_contract,
    };
    extension.index_fund_contract = match msg.index_fund_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => extension.index_fund_contract,
    };
    extension.donation_match_charites_contract = match msg.donation_match_charites_contract {
        Some(v) => Some(deps.api.addr_validate(v.as_str())?),
        None => extension.donation_match_charites_contract,
    };
    extension.fundraising_contract = match msg.fundraising_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => extension.fundraising_contract,
    };
    extension.subdao_gov_code = match msg.subdao_gov_code {
        Some(u64) => Some(u64),
        None => extension.subdao_gov_code,
    };
    extension.subdao_bonding_token_code = match msg.subdao_bonding_token_code {
        Some(u64) => Some(u64),
        None => extension.subdao_bonding_token_code,
    };
    extension.subdao_cw20_token_code = match msg.subdao_cw20_token_code {
        Some(u64) => Some(u64),
        None => extension.subdao_cw20_token_code,
    };
    extension.subdao_cw900_code = match msg.subdao_cw900_code {
        Some(u64) => Some(u64),
        None => extension.subdao_cw900_code,
    };
    extension.subdao_distributor_code = match msg.subdao_distributor_code {
        Some(u64) => Some(u64),
        None => extension.subdao_distributor_code,
    };
    extension.donation_match_code = match msg.donation_match_code {
        Some(u64) => Some(u64),
        None => extension.donation_match_code,
    };
    extension.swap_factory = match msg.swap_factory {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => extension.swap_factory,
    };
    extension.halo_token = match msg.halo_token {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => extension.halo_token,
    };
    extension.halo_token_lp_contract = match msg.halo_token_lp_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => extension.halo_token_lp_contract,
    };
    extension.accounts_settings_controller = match msg.accounts_settings_controller {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => extension.accounts_settings_controller,
    };

    CONFIG_EXTENSION.save(deps.storage, &extension)?;

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

    // check the strategy key provided is correct length of bytes (4)
    let strat_bytes = strategy_key.as_bytes();
    if strat_bytes.len() != 4 {
        return Err(ContractError::InvalidInputs {});
    }
    // check that the vault does not already exist for a given address in storage
    // add new strategy if all looks good
    STRATEGIES.update(deps.storage, &strat_bytes, |existing| match existing {
        Some(_) => Err(ContractError::StrategyAlreadyExists {}),
        None => Ok(strategy),
    })?;
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

    // check the strategy key provided is correct length of bytes (4)
    let strat_bytes = strategy_key.as_bytes();
    if strat_bytes.len() != 4 {
        return Err(ContractError::InvalidInputs {});
    }
    // remove the Strategy from storage
    STRATEGIES.remove(deps.storage, &strat_bytes);
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

    // check the strategy key provided is correct length of bytes (4)
    let strat_bytes = strategy_key.as_bytes();
    if strat_bytes.len() != 4 {
        return Err(ContractError::InvalidInputs {});
    }
    // try to look up the given strategy in Storage
    let mut strategy = STRATEGIES.load(deps.storage, &strat_bytes)?;

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
