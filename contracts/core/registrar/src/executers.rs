use crate::state::{CONFIG, FEES, NETWORK_CONNECTIONS, VAULTS};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::structs::{AcceptedTokens, EndowmentType, NetworkInfo, VaultType, YieldVault};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

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
    config.applications_review = match msg.applications_review {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.applications_review,
    };
    config.accounts_contract = match msg.accounts_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => config.accounts_contract,
    };
    config.swaps_router = match msg.swaps_router {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => config.swaps_router,
    };
    config.cw3_code = match msg.cw3_code {
        Some(v) => Some(v),
        None => config.cw3_code,
    };
    config.cw4_code = match msg.cw4_code {
        Some(v) => Some(v),
        None => config.cw4_code,
    };
    config.charity_shares_contract = match msg.charity_shares_contract {
        Some(contract_addr) => Some(deps.api.addr_validate(&contract_addr)?),
        None => config.charity_shares_contract,
    };
    config.index_fund_contract = match msg.index_fund_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => config.index_fund_contract,
    };
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
    config.accepted_tokens = AcceptedTokens {
        native: msg
            .accepted_tokens_native
            .unwrap_or(config.accepted_tokens.native),
        cw20: msg
            .accepted_tokens_cw20
            .unwrap_or(config.accepted_tokens.cw20),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn vault_add(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: VaultAddMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // validate the passed address
    let addr = deps.api.addr_validate(&msg.vault_addr)?;

    // check that the vault does not already exist for a given address in storage
    if VAULTS.load(deps.storage, addr.as_bytes()).is_ok() {
        return Err(ContractError::VaultAlreadyExists {});
    }

    // check that valid network connection info is provided for a vault
    let vault_network = msg.network.unwrap_or(env.block.chain_id);
    let network = NETWORK_CONNECTIONS.load(deps.storage, &vault_network)?;

    // if non-native vault type, ensure the NetworkInfo has IBC configured
    if msg.vault_type != VaultType::Native && network.ibc_channel.is_none() {
        return Err(ContractError::Std(StdError::generic_err(
            "IBC Channel must be configured before adding a non-Native Vault",
        )));
    }

    // save the new vault to storage
    VAULTS.save(
        deps.storage,
        addr.as_bytes(),
        &YieldVault {
            network: vault_network,
            address: addr.to_string(),
            input_denom: msg.input_denom,
            yield_token: deps.api.addr_validate(&msg.yield_token)?.to_string(),
            approved: true,
            restricted_from: msg.restricted_from,
            acct_type: msg.acct_type,
            vault_type: msg.vault_type,
        },
    )?;
    Ok(Response::default())
}

pub fn vault_remove(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // validate the passed address
    let _addr = deps.api.addr_validate(&vault_addr)?;

    // remove the vault from storage
    VAULTS.remove(deps.storage, vault_addr.as_bytes());
    Ok(Response::default())
}

pub fn vault_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
    approved: bool,
    restricted_from: Vec<EndowmentType>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given vault in Storage
    let addr = deps.api.addr_validate(&vault_addr)?;
    let mut vault = VAULTS.load(deps.storage, addr.as_bytes())?;

    // update new vault approval status attribute from passed arg
    vault.approved = approved;
    // set any restricted endowment types
    vault.restricted_from = restricted_from;
    VAULTS.save(deps.storage, addr.as_bytes(), &vault)?;

    Ok(Response::default())
}

pub fn update_network_connections(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    network_info: NetworkInfo,
    action: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    if action == *"post" {
        // Add/Overwrite the network_info to NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.save(deps.storage, &network_info.chain_id, &network_info)?;
    } else if action == *"delete" {
        // Remove the network_info from NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.remove(deps.storage, &network_info.chain_id);
    } else {
        return Err(ContractError::InvalidInputs {});
    }

    Ok(Response::default().add_attribute("action", "update_network_connections"))
}
