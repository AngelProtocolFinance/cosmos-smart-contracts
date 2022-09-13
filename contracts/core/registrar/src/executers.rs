use crate::state::{CONFIG, ENDOWTYPE_FEES, NETWORK_CONNECTIONS, VAULTS};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::structs::{EndowmentType, NetworkInfo, YieldVault};
use angel_core::utils::{percentage_checks, split_checks};

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};

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
    config.tax_rate = match msg.tax_rate {
        Some(tax_rate) => percentage_checks(tax_rate),
        None => Ok(config.tax_rate),
    }
    .unwrap();
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
    config.donation_match_charites_contract = match msg.donation_match_charites_contract {
        Some(v) => Some(deps.api.addr_validate(v.as_str())?),
        None => config.donation_match_charites_contract,
    };
    config.accepted_tokens = match msg.accepted_tokens {
        Some(tokens) => tokens,
        None => config.accepted_tokens,
    };
    config.fundraising_contract = match msg.fundraising_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => config.fundraising_contract,
    };
    config.collector_addr = msg
        .collector_addr
        .map(|addr| deps.api.addr_validate(&addr).unwrap());
    config.collector_share = match msg.collector_share {
        Some(share) => share,
        None => config.collector_share,
    };
    config.subdao_gov_code = match msg.subdao_gov_code {
        Some(u64) => Some(u64),
        None => config.subdao_gov_code,
    };
    config.subdao_bonding_token_code = match msg.subdao_bonding_token_code {
        Some(u64) => Some(u64),
        None => config.subdao_bonding_token_code,
    };
    config.subdao_cw20_token_code = match msg.subdao_cw20_token_code {
        Some(u64) => Some(u64),
        None => config.subdao_cw20_token_code,
    };
    config.subdao_cw900_code = match msg.subdao_cw900_code {
        Some(u64) => Some(u64),
        None => config.subdao_cw900_code,
    };
    config.subdao_distributor_code = match msg.subdao_distributor_code {
        Some(u64) => Some(u64),
        None => config.subdao_distributor_code,
    };
    config.donation_match_code = match msg.donation_match_code {
        Some(u64) => Some(u64),
        None => config.donation_match_code,
    };
    config.swap_factory = match msg.swap_factory {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => config.swap_factory,
    };
    config.halo_token = match msg.halo_token {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => config.halo_token,
    };
    config.halo_token_lp_contract = match msg.halo_token_lp_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr).unwrap()),
        None => config.halo_token_lp_contract,
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

    // save the new vault to storage
    VAULTS.save(
        deps.storage,
        addr.as_bytes(),
        &YieldVault {
            network: msg.network.unwrap_or(env.block.chain_id),
            address: addr.to_string(),
            input_denom: msg.input_denom,
            yield_token: deps.api.addr_validate(&msg.yield_token)?.to_string(),
            approved: true,
            restricted_from: msg.restricted_from,
            acct_type: msg.acct_type,
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

pub fn update_endowtype_fees(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowTypeFeesMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // Update the "fees"
    ENDOWTYPE_FEES.save(deps.storage, "charity".to_string(), &msg.endowtype_charity)?;
    ENDOWTYPE_FEES.save(deps.storage, "normal".to_string(), &msg.endowtype_normal)?;

    Ok(Response::new().add_attribute("action", "update_endowtype_fees"))
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

    if action == *"add" {
        // Add the network_info to NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.save(deps.storage, &network_info.chain_id, &network_info)?;
    } else if action == *"remove" {
        // Remove the network_info from NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.remove(deps.storage, &network_info.chain_id);
    } else {
        return Err(ContractError::InvalidInputs {});
    }

    Ok(Response::default().add_attribute("action", "update_network_connections"))
}
