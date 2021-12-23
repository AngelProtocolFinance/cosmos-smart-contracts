use crate::state::{read_config, store_config, Config};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use halo_token::gov_hodler::{ExecuteMsg, InstantiateMsg, MigrateMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner: info.sender,
            gov_contract: deps.api.addr_validate(&msg.gov_contract)?,
            halo_token: deps.api.addr_validate(&msg.halo_token)?,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { gov_contract } => update_config(deps, info, gov_contract),
        ExecuteMsg::ClaimHalo { recipient, amount } => claim_halo(deps, info, recipient, amount),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    gov_contract: String,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if info.sender != config.gov_contract && info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }
    config.gov_contract = deps.api.addr_validate(&gov_contract)?;
    store_config(deps.storage, &config)?;
    Ok(Response::default())
}

/// Gov Contract only may send requests for Gov HALO Hodler contract to send some amount
/// of HALO tokens to a reciepient. These tokens are tokens eligable for Claiming from Gov.
pub fn claim_halo(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    if info.sender != config.gov_contract {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.halo_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient, amount })?,
            funds: vec![],
        }))
        .add_attributes(vec![
            ("action", "hodler_claim"),
            ("amount", &amount.to_string()),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
