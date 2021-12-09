#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    attr, to_binary, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::state::{read_config, store_config, Config};
use crate::querier::query_pair_info;

use cw20::Cw20ExecuteMsg;
use halo_token::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use astroport_lbp::asset::{Asset, AssetInfo, PairInfo};
use astroport_lbp::factory::FactoryPairInfo;
use astroport_lbp::pair::ExecuteMsg as LBPExecuteMsg;
use astroport_lbp::querier::{query_balance, query_factory_pair_info, query_token_balance};

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
            lbp_factory: deps.api.addr_validate(&msg.lbp_factory)?,
            halo_token: deps.api.addr_validate(&msg.halo_token)?,
            distributor_contract: deps.api.addr_validate(&msg.distributor_contract)?,
            reward_factor: msg.reward_factor,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { reward_factor, gov_contract } => update_config(deps, info, reward_factor, gov_contract),
        ExecuteMsg::Sweep { denom } => sweep(deps, env, denom),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    reward_factor: Option<Decimal>,
    gov_contract: Option<String>,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if info.sender != config.gov_contract {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(reward_factor) = reward_factor {
        config.reward_factor = reward_factor;
    }

    if let Some(gov_contract) = gov_contract {
        config.gov_contract = deps.api.addr_validate(&gov_contract)?;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::default())
}

const SWEEP_REPLY_ID: u64 = 1;

/// Sweep
/// Anyone can execute sweep function to swap
/// asset token => HALO token and distribute
/// result HALO token to gov contract
pub fn sweep(deps: DepsMut, env: Env, denom: String) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    let pair_info: FactoryPairInfo = query_factory_pair_info(
        deps.as_ref(),
        &config.lbp_factory,
        &[
            AssetInfo::NativeToken {
                denom: denom.to_string(),
            },
            AssetInfo::Token {
                contract_addr: config.halo_token,
            },
        ],
    )?;

    let amount = query_balance(deps.as_ref(), &env.contract.address, denom.to_string())?;

    let swap_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: denom.to_string(),
        },
        amount,
    };

    // deduct tax first
    let amount = (swap_asset.deduct_tax(deps.as_ref())?).amount;
    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_info.contract_addr.to_string(),
                msg: to_binary(&LBPExecuteMsg::Swap {
                    offer_asset: Asset {
                        amount,
                        ..swap_asset
                    },
                    max_spread: None,
                    belief_price: None,
                    to: None,
                })?,
                funds: vec![Coin {
                    denom: denom.to_string(),
                    amount,
                }],
            }),
            SWEEP_REPLY_ID,
        ))
        .add_attributes(vec![
            attr("action", "sweep"),
            attr(
                "collected_rewards",
                format!("{:?}{:?}", amount.to_string(), denom),
            ),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    if msg.id == SWEEP_REPLY_ID {
        // send tokens on successful callback
        return distribute(deps, env);
    }

    Err(StdError::generic_err("not supported reply"))
}

// Only contract itself can execute distribute function
pub fn distribute(deps: DepsMut, env: Env) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    let amount = query_token_balance(
        deps.as_ref(),
        &config.halo_token,
        &env.contract.address,
    )?;

    let distribute_amount = amount * config.reward_factor;
    let left_amount = amount.checked_sub(distribute_amount)?;

    let mut messages: Vec<CosmosMsg> = vec![];

    if !distribute_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.halo_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.gov_contract.to_string(),
                amount: distribute_amount,
            })?,
            funds: vec![],
        }));
    }

    if !left_amount.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.halo_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.distributor_contract.to_string(),
                amount: left_amount,
            })?,
            funds: vec![],
        }));
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "distribute"),
        ("distribute_amount", &distribute_amount.to_string()),
        ("distributor_payback_amount", &left_amount.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pair {} => to_binary(&query_pair(deps)?)
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        gov_contract: state.gov_contract.to_string(),
        lbp_factory: state.lbp_factory.to_string(),
        halo_token: state.halo_token.to_string(),
        distributor_contract: state.distributor_contract.to_string(),
        reward_factor: state.reward_factor,
    };

    Ok(resp)
}

pub fn query_pair(deps: Deps) -> StdResult<PairInfo> {
    let config: Config = read_config(deps.storage)?;

    let pair_info: PairInfo = query_pair_info(
        deps,
        &config.lbp_factory,
    )?;

    Ok(pair_info)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
