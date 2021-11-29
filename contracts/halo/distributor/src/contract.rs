#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{read_config, store_config, Config};

use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};

use halo_token::distributor::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use cw20::Cw20ExecuteMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let whitelist = msg
        .whitelist
        .into_iter()
        .map(|w| deps.api.addr_validate(&w))
        .collect::<StdResult<Vec<Addr>>>()?;

    store_config(
        deps.storage,
        &Config {
            gov_contract: deps.api.addr_validate(&msg.gov_contract)?,
            halo_token: deps.api.addr_validate(&msg.halo_token)?,
            whitelist,
            spend_limit: msg.spend_limit,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            spend_limit,
            gov_contract,
        } => update_config(deps, info, spend_limit, gov_contract),
        ExecuteMsg::Spend { recipient, amount } => spend(deps, info, recipient, amount),
        ExecuteMsg::AddDistributor { distributor } => add_distributor(deps, info, distributor),
        ExecuteMsg::RemoveDistributor { distributor } => {
            remove_distributor(deps, info, distributor)
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    spend_limit: Option<Uint128>,
    gov_contract: Option<String>,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(spend_limit) = spend_limit {
        config.spend_limit = spend_limit;
    }

    if let Some(gov_contract) = gov_contract {
        config.gov_contract = deps.api.addr_validate(&gov_contract)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn add_distributor(
    deps: DepsMut,
    info: MessageInfo,
    distributor: String,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let distributor_raw = deps.api.addr_validate(&distributor)?;
    if config
        .whitelist
        .clone()
        .into_iter()
        .any(|w| w == distributor_raw)
    {
        return Err(StdError::generic_err("Distributor already registered"));
    }

    config.whitelist.push(distributor_raw);
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "add_distributor"),
        ("distributor", distributor.as_str()),
    ]))
}

pub fn remove_distributor(
    deps: DepsMut,
    info: MessageInfo,
    distributor: String,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if config.gov_contract != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let distributor_raw = deps.api.addr_validate(&distributor)?;
    let whitelist_len = config.whitelist.len();
    let whitelist: Vec<Addr> = config
        .whitelist
        .into_iter()
        .filter(|w| *w != distributor_raw)
        .collect();

    if whitelist_len == whitelist.len() {
        return Err(StdError::generic_err("Distributor not found"));
    }

    config.whitelist = whitelist;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "remove_distributor"),
        ("distributor", distributor.as_str()),
    ]))
}

/// Spend
/// Owner can execute spend operation to send
/// `amount` of HALO token to `recipient` for community purpose
pub fn spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    if !config.whitelist.into_iter().any(|w| w == info.sender) {
        return Err(StdError::generic_err("unauthorized"));
    }

    if config.spend_limit < amount {
        return Err(StdError::generic_err("Cannot spend more than spend_limit"));
    }

    let halo_token = config.halo_token.to_string();
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: halo_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "spend"),
            ("recipient", recipient.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        gov_contract: state.gov_contract.to_string(),
        halo_token: state.halo_token.to_string(),
        whitelist: state
            .whitelist
            .into_iter()
            .map(|w| w.to_string())
            .collect::<Vec<String>>(),
        spend_limit: state.spend_limit,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
