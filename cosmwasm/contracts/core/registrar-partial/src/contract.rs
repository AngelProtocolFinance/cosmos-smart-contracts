use crate::executers;
use crate::queriers;
use crate::state::{CONFIG, FEES, NETWORK_CONNECTIONS};
use angel_core::errors::core::ContractError;
use angel_core::msgs::registrar::InstantiateMsg;
use angel_core::msgs::registrar_partial::{ExecuteMsg, MigrateMsg, QueryMsg};
use angel_core::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, RegistrarConfigCore, SplitDetails,
};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::{get_contract_version, set_contract_version};

// version info for future migration info
const CONTRACT_NAME: &str = "registrar-partial";
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
        axelar_gateway: msg.axelar_gateway,
        axelar_ibc_channel: msg.axelar_ibc_channel,
        axelar_chain_id: msg.axelar_chain_id.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    // setup first basic fees
    FEES.save(deps.storage, &"vaults_harvest", &tax_rate)?;
    FEES.save(deps.storage, &"accounts_withdraw", &Decimal::permille(2))?; // Default to 0.002 or 0.2%

    // setup basic JUNO network info for native Strategies
    NETWORK_CONNECTIONS.save(
        deps.storage,
        &msg.axelar_chain_id,
        &NetworkInfo {
            accounts_contract: None,
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
