use crate::executers;
use crate::queriers;
use crate::state::{OldConfig, CONFIG, CONFIG_EXTENSION, FEES, NETWORK_CONNECTIONS};
use angel_core::errors::core::ContractError;
use angel_core::msgs::registrar::*;
use angel_core::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, RegistrarConfigCore, RegistrarConfigExtension,
    SplitDetails,
};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    entry_point, from_slice, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};

// version info for future migration info
const CONTRACT_NAME: &str = "registrar";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
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

    let config = RegistrarConfigCore {
        owner: info.sender.clone(),
        treasury: deps.api.addr_validate(&msg.treasury)?,
        rebalance: msg.rebalance.unwrap_or_else(RebalanceDetails::default),
        split_to_liquid: splits,
        accepted_tokens: msg.accepted_tokens.unwrap_or_else(AcceptedTokens::default),
        axelar_chain_id: msg.axelar_chain_id.clone(),
        axelar_gateway: msg.axelar_gateway,
        axelar_ibc_channel: msg.axelar_ibc_channel,
    };
    CONFIG.save(deps.storage, &config)?;

    let extension = RegistrarConfigExtension {
        applications_review: info.sender.clone(),
        index_fund_contract: None,
        accounts_contract: None,
        cw3_code: None,
        cw4_code: None,
        subdao_gov_code: None,
        subdao_cw20_token_code: None,
        subdao_bonding_token_code: None,
        subdao_cw900_code: None,
        subdao_distributor_code: None,
        donation_match_code: None,
        halo_token: None,
        halo_token_lp_contract: None,
        gov_contract: None,
        donation_match_charites_contract: None,
        collector_addr: None,
        charity_shares_contract: None,
        fundraising_contract: None,
        swap_factory: msg
            .swap_factory
            .map(|v| deps.api.addr_validate(&v).unwrap()),
        swaps_router: None,
        accounts_settings_controller: None,
    };
    CONFIG_EXTENSION.save(deps.storage, &extension)?;

    // setup first basic fees
    FEES.save(deps.storage, &"vaults_harvest", &tax_rate)?;
    FEES.save(deps.storage, &"accounts_withdraw", &Decimal::permille(2))?; // Default to 0.002 or 0.2%

    // setup basic JUNO network info for native Strategies
    NETWORK_CONNECTIONS.save(
        deps.storage,
        &msg.axelar_chain_id,
        &NetworkInfo {
            accounts_contract: match extension.accounts_contract {
                Some(accounts) => Some(accounts.to_string()),
                None => None,
            },
            router_contract: None,
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
        ExecuteMsg::UpdateConfigExtension(msg) => {
            executers::update_config_extension(deps, env, info, msg)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::StrategyAdd {
            strategy_key,
            strategy,
        } => executers::strategy_add(deps, env, info, strategy_key, strategy),
        ExecuteMsg::StrategyRemove { strategy_key } => {
            executers::strategy_remove(deps, env, info, strategy_key)
        }
        ExecuteMsg::StrategyUpdate {
            strategy_key,
            approval_state,
        } => executers::strategy_update(deps, env, info, strategy_key, approval_state),
        ExecuteMsg::UpdateNetworkConnections {
            chain_id,
            network_info,
            action,
        } => executers::update_network_connections(deps, env, info, chain_id, network_info, action),
        ExecuteMsg::UpdateFees { fees } => executers::update_fees(deps, info, fees),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::ConfigExtension {} => to_binary(&queriers::query_config_extension(deps)?),
        QueryMsg::Strategy { strategy_key } => {
            to_binary(&queriers::query_strategy(deps, strategy_key)?)
        }
        QueryMsg::NetworkConnection { chain_id } => {
            to_binary(&queriers::query_network_connection(deps, chain_id)?)
        }
        QueryMsg::Fee { name } => to_binary(&queriers::query_fee(deps, name)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
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

    // Get the new addr configs from migrate msg input
    let accounts_settings_controller =
        Some(deps.api.addr_validate(&msg.accounts_settings_controller)?);

    // setup the new config struct and save to storage
    let data = deps
        .storage
        .get("config".as_bytes())
        .ok_or_else(|| StdError::not_found("Config not found"))?;
    let old_config: OldConfig = from_slice(&data)?;

    // replace old juno connection with new network connection struct
    NETWORK_CONNECTIONS.save(
        deps.storage,
        &msg.axelar_chain_id,
        &NetworkInfo {
            accounts_contract: match old_config.accounts_contract.clone() {
                Some(accounts) => Some(accounts.to_string()),
                None => None,
            },
            router_contract: None,
        },
    )?;
    // build new config struct & save
    CONFIG.save(
        deps.storage,
        &RegistrarConfigCore {
            owner: old_config.owner,
            treasury: old_config.treasury,
            split_to_liquid: old_config.split_to_liquid,
            accepted_tokens: old_config.accepted_tokens,
            rebalance: old_config.rebalance,
            axelar_gateway: msg.axelar_gateway,
            axelar_ibc_channel: msg.axelar_ibc_channel,
            axelar_chain_id: msg.axelar_chain_id,
        },
    )?;
    CONFIG_EXTENSION.save(
        deps.storage,
        &RegistrarConfigExtension {
            applications_review: old_config.applications_review,
            index_fund_contract: old_config.index_fund_contract,
            accounts_contract: old_config.accounts_contract,
            cw3_code: old_config.cw3_code,
            cw4_code: old_config.cw4_code,
            halo_token: old_config.halo_token,
            gov_contract: old_config.gov_contract,
            charity_shares_contract: old_config.charity_shares_contract,
            swaps_router: old_config.swaps_router,
            subdao_gov_code: None,
            subdao_cw20_token_code: None,
            subdao_bonding_token_code: None,
            subdao_cw900_code: None,
            subdao_distributor_code: None,
            donation_match_code: None,
            donation_match_charites_contract: None,
            halo_token_lp_contract: None,
            collector_addr: None,
            swap_factory: None,
            fundraising_contract: None,
            accounts_settings_controller,
        },
    )?;

    Ok(Response::default())
}
