use crate::anchor;
use crate::anchor::HandleMsg;
use crate::config;
use crate::msg::{InitMsg, MigrateMsg};
use angel_core::error::ContractError;
use angel_core::portals::{ConfigResponse, ExchangeRateResponse, QueryMsg};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    let moneymarket = deps.api.addr_validate(&msg.moneymarket)?;
    let anchor_config = anchor::config(deps.as_ref(), &moneymarket)?;

    let config = config::Config {
        owner: info.sender,
        moneymarket,
        input_denom: anchor_config.stable_denom.clone(),
        yield_token: deps.api.addr_validate(&anchor_config.aterra_contract)?,
    };

    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn handle(_deps: Deps, _env: Env, _msg: HandleMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    match msg {
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            input_denom: config.input_denom.clone(),
            yield_token: config.yield_token.clone().to_string(),
        }),
        QueryMsg::ExchangeRate { input_denom: _ } => {
            let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;

            to_binary(&ExchangeRateResponse {
                exchange_rate: epoch_state.exchange_rate.clone(),
                yield_token_supply: epoch_state.aterra_supply.clone(),
            })
        }
        QueryMsg::Deposit { amount } => to_binary(&anchor::deposit_stable_msg(
            deps,
            &config.moneymarket,
            &config.input_denom,
            amount.into(),
        )?),
        QueryMsg::Redeem { amount } => to_binary(&anchor::redeem_stable_msg(
            deps,
            &config.moneymarket,
            &config.yield_token,
            amount.into(),
        )?),
    }
}

pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
