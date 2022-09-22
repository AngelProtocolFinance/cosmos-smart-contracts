use crate::executers;
use crate::queriers;
use crate::state::{Config, CONFIG, NETWORK_CONNECTIONS};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::structs::{AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::{get_contract_version, set_contract_version};

// version info for future migration info
const CONTRACT_NAME: &str = "registrar";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let tax_rate = percentage_checks(msg.tax_rate).unwrap();
    let splits: SplitDetails = match msg.split_to_liquid {
        Some(splits) => split_checks(splits.max, splits.min, splits.default),
        None => Ok(SplitDetails::default()),
    }
    .unwrap();

    let configs = Config {
        owner: info.sender.clone(),
        applications_review: info.sender,
        index_fund_contract: None,
        accounts_contract: None,
        treasury: deps.api.addr_validate(&msg.treasury)?,
        tax_rate,
        cw3_code: None,
        cw4_code: None,
        subdao_gov_code: None,
        subdao_cw20_token_code: None,
        subdao_bonding_token_code: None,
        subdao_cw900_code: None,
        subdao_distributor_code: None,
        donation_match_code: None,
        rebalance: msg.rebalance.unwrap_or_else(RebalanceDetails::default),
        split_to_liquid: splits,
        halo_token: None,
        halo_token_lp_contract: None,
        gov_contract: None,
        donation_match_charites_contract: None,
        collector_addr: None,
        collector_share: Decimal::percent(50_u64),
        charity_shares_contract: None,
        accepted_tokens: msg.accepted_tokens.unwrap_or_else(AcceptedTokens::default),
        fundraising_contract: None,
        swap_factory: msg
            .swap_factory
            .map(|v| deps.api.addr_validate(&v).unwrap()),
        swaps_router: None,
    };

    CONFIG.save(deps.storage, &configs)?;

    // setup basic JUNO network info for native Vaults
    NETWORK_CONNECTIONS.save(
        deps.storage,
        &env.block.chain_id.clone(),
        &NetworkInfo {
            name: "Juno".to_string(),
            chain_id: env.block.chain_id,
            ibc_channel: None,
            ibc_host_contract: None,
            gas_limit: None,
        },
    )?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::VaultAdd(msg) => executers::vault_add(deps, env, info, msg),
        ExecuteMsg::VaultRemove { vault_addr } => {
            executers::vault_remove(deps, env, info, vault_addr)
        }
        ExecuteMsg::VaultUpdate {
            vault_addr,
            approved,
            restricted_from,
        } => executers::vault_update(deps, env, info, vault_addr, approved, restricted_from),
        ExecuteMsg::UpdateEndowTypeFees(msg) => {
            executers::update_endowtype_fees(deps, env, info, msg)
        }
        ExecuteMsg::UpdateNetworkConnections {
            network_info,
            action,
        } => executers::update_network_connections(deps, env, info, network_info, action),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::VaultList {
            network,
            endowment_type,
            acct_type,
            vault_type,
            approved,
            start_after,
            limit,
        } => to_binary(&queriers::query_vault_list(
            deps,
            network,
            endowment_type,
            acct_type,
            vault_type,
            approved,
            start_after,
            limit,
        )?),
        QueryMsg::Vault { vault_addr } => {
            to_binary(&queriers::query_vault_details(deps, vault_addr)?)
        }
        QueryMsg::Fees {} => to_binary(&queriers::query_fees(deps)?),
        QueryMsg::NetworkConnection { chain_id } => {
            to_binary(&queriers::query_network_connection(deps, chain_id)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        }));
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        }));
    }
    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
