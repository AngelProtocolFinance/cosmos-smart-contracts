#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{read_config, store_config, Config};

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, WasmMsg,
};

use halo_token::community::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use cw20::Cw20ExecuteMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            gov_contract: deps.api.addr_validate(&msg.gov_contract)?,
            halo_token: deps.api.addr_validate(&msg.halo_token)?,
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
    if config.gov_contract != info.sender {
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
            ("amount", &amount.to_string()),
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
        spend_limit: state.spend_limit,
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    // Re-save the config because of storage switch (singleton -> Item)
    // Should be removed after v1.6 deployment

    // CONFIG
    let config_key: &[u8] = b"config";
    let prefixed_config_key: &[u8] = &cosmwasm_storage::to_length_prefixed(config_key);
    let data = deps
        .storage
        .get(prefixed_config_key)
        .ok_or_else(|| StdError::NotFound {
            kind: "Config".to_string(),
        })?;
    let config: Config = cosmwasm_std::from_slice(&data)?;
    deps.storage.set(
        config_key,
        &cosmwasm_std::to_vec(&Config {
            gov_contract: config.gov_contract,
            halo_token: config.halo_token,
            spend_limit: config.spend_limit,
        })?,
    );

    Ok(Response::default())
}
