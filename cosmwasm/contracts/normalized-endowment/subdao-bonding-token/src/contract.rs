use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw20_base::allowances::{
    deduct_allowance, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{query_balance, query_token_info};
use cw_utils::Duration;

use crate::state::{
    Config, CurveState, MinterData, TokenInfo, BALANCES, CLAIMS, CONFIG, CURVE_STATE, CURVE_TYPE,
    TOKEN_INFO,
};
use angel_core::curves::DecimalPlaces;
use angel_core::errors::core::ContractError;
use angel_core::msgs::subdao_bonding_token::{
    CurveFn, CurveInfoResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use angel_core::utils::{must_pay, nonpayable};

// version info for migration info
const CONTRACT_NAME: &str = "dao-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // store token info using cw20 standard format
    TOKEN_INFO.save(
        deps.storage,
        &TokenInfo {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            total_supply: Uint128::zero(),
            // set self as minter, so we can properly execute mint and burn
            mint: Some(MinterData {
                minter: env.contract.address,
                cap: None,
            }),
        },
    )?;

    CONFIG.save(
        deps.storage,
        &Config {
            unbonding_period: Duration::Time(msg.unbonding_period), // secconds of unbonding
        },
    )?;

    let places = DecimalPlaces::new(msg.decimals, msg.reserve_decimals);
    let supply = CurveState::new(msg.reserve_denom, places);
    CURVE_STATE.save(deps.storage, &supply)?;
    CURVE_TYPE.save(deps.storage, &msg.curve_type)?;

    Ok(Response::default())
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // only reserve asset contract can execute these messages
    let curve = CURVE_STATE.load(deps.storage)?;
    if curve.reserve_denom != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if cw20_msg.amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let token_holder_address = deps.api.addr_validate(&cw20_msg.sender)?;
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Buy {}) => execute_buy_cw20(
            deps,
            env,
            token_holder_address, // addr of HALO holder who's purchasing
            cw20_msg.amount,      // how much HALO sending
        ),
        Ok(Cw20HookMsg::DonorMatch {
            amount,
            donor,
            accounts_contract,
            endowment_id,
        }) => execute_donor_match(
            deps,
            env,
            info,
            cw20_msg.amount,
            amount,
            donor,
            accounts_contract,
            endowment_id,
        ),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // default implementation stores curve info as enum, you can do something else in a derived
    // contract and just pass in your custom curve to do_execute
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    do_execute(deps, env, info, msg, curve_fn)
}

/// We pull out logic here, so we can import this from another contract and set a different Curve.
/// This contacts sets a curve with an enum in InstantiateMsg and stored in state, but you may want
/// to use custom math not included - mexecute_burnake this easily reusable
pub fn do_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
    curve_fn: CurveFn,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_receive_msg) => {
            Ok(receive_cw20(deps, env, info, cw20_receive_msg)?)
        }
        // We only accept CW20 tokens for reseve asset curve at this time (see receive_cw20 > ExecuteMsg::Buy).
        // ExecuteMsg::Buy {} => execute_buy(deps, env, info, curve_fn),
        ExecuteMsg::Burn { amount } => Ok(execute_sell(deps, env, info, curve_fn, amount)?),
        ExecuteMsg::BurnFrom { owner, amount } => {
            Ok(execute_sell_from(deps, env, info, curve_fn, owner, amount)?)
        }
        ExecuteMsg::ClaimTokens {} => Ok(claim_tokens(deps, env, info)?),

        // these all come from cw20-base to implement the cw20 standard
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
    }
}

pub fn execute_buy_cw20(
    deps: DepsMut,
    env: Env,
    buyer_addr: Addr,
    buyer_amount: Uint128,
) -> Result<Response, ContractError> {
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    let mut state = CURVE_STATE.load(deps.storage)?;

    // calculate how many tokens can be purchased with this and mint them
    let curve = curve_fn(state.decimals);
    state.reserve += buyer_amount;
    let new_supply = curve.supply(state.reserve);
    let minted = new_supply
        .checked_sub(state.supply)
        .map_err(StdError::overflow)?;
    state.supply = new_supply;
    CURVE_STATE.save(deps.storage, &state)?;

    // call into cw20-base to mint the token, call as self as no one else is allowed
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    execute_mint(deps, env, sub_info, buyer_addr.to_string(), minted)?;

    // bond them to the validator
    let res = Response::new()
        .add_attribute("action", "buy")
        .add_attribute("from", buyer_addr.to_string())
        .add_attribute("reserve", buyer_amount)
        .add_attribute("supply", minted);
    Ok(res)
}

pub fn execute_donor_match(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sent_reserve_token_amount: Uint128,
    amount: Uint128,
    donor: String,
    accounts_contract: String,
    _endowment_id: u32, // FIXME: Should be used for minting the subdao token
) -> Result<Response, ContractError> {
    // Validation: Check if the correct amount of tokens are sent
    if sent_reserve_token_amount != amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // Calculate the amounts of dao-token to be minted and burned
    // Minted for the "donor": 40%
    // Minted for the "endowment_contract": 40%
    // Burned: 20% (This action is just needed for calculation purposes, no real action is needed.)
    let donor_amount = amount.multiply_ratio(40_u128, 100_u128);
    let endowment_amount = amount.multiply_ratio(40_u128, 100_u128);
    let burn_amount = amount.multiply_ratio(20_u128, 100_u128);

    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };

    // Mint to "donor": 40%
    execute_mint(
        deps.branch(),
        env.clone(),
        sub_info.clone(),
        donor,
        donor_amount,
    )?;
    // Mint to "endowment_contract": 40%
    execute_mint(deps, env, sub_info, accounts_contract, endowment_amount)?; // FIXME: How to handle `accounts_contract` & `endowment_id` together

    Ok(Response::default().add_attributes(vec![
        attr("method", "donor_match"),
        attr("donor_amount", donor_amount),
        attr("endowment_amount", endowment_amount),
        attr("burnt_amount", burn_amount),
    ]))
}

pub fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    curve_fn: CurveFn,
) -> Result<Response, ContractError> {
    let mut state = CURVE_STATE.load(deps.storage)?;

    let payment = must_pay(&info, &state.reserve_denom)?;

    // calculate how many tokens can be purchased with this and mint them
    let curve = curve_fn(state.decimals);
    state.reserve += payment;
    let new_supply = curve.supply(state.reserve);
    let minted = new_supply
        .checked_sub(state.supply)
        .map_err(StdError::overflow)?;
    state.supply = new_supply;
    CURVE_STATE.save(deps.storage, &state)?;

    // call into cw20-base to mint the token, call as self as no one else is allowed
    let sub_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    execute_mint(deps, env, sub_info, info.sender.to_string(), minted)?;

    // bond them to the validator
    let res = Response::new()
        .add_attribute("action", "buy")
        .add_attribute("from", info.sender)
        .add_attribute("reserve", payment)
        .add_attribute("supply", minted);
    Ok(res)
}

pub fn execute_sell(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    curve_fn: CurveFn,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let receiver = info.sender.clone();
    // do all the work
    let mut res = do_sell(deps, env, info, curve_fn, receiver, amount)?;

    // add our custom attributes
    res.attributes.push(attr("action", "burn"));
    Ok(res)
}

pub fn execute_sell_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    curve_fn: CurveFn,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let owner_addr = deps.api.addr_validate(&owner)?;
    let spender_addr = info.sender.clone();

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &spender_addr, &env.block, amount)?;

    // do all the work in do_sell
    let receiver_addr = info.sender;
    let owner_info = MessageInfo {
        sender: owner_addr,
        funds: info.funds,
    };
    let mut res = do_sell(
        deps,
        env,
        owner_info,
        curve_fn,
        receiver_addr.clone(),
        amount,
    )?;

    // add our custom attributes
    res.attributes.push(attr("action", "burn_from"));
    res.attributes.push(attr("by", receiver_addr));
    Ok(res)
}

fn do_sell(
    mut deps: DepsMut,
    env: Env,
    // info.sender is the one burning tokens
    info: MessageInfo,
    curve_fn: CurveFn,
    // receiver is the one who gains (same for execute_sell, diff for execute_sell_from)
    _receiver: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let block = env.clone().block;
    // burn from the caller, this ensures there are tokens to cover this
    execute_burn(deps.branch(), env, info.clone(), amount)?;

    // calculate how many tokens can be purchased with this and mint them
    let mut state = CURVE_STATE.load(deps.storage)?;
    let curve = curve_fn(state.decimals);
    state.supply = state
        .supply
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    let new_reserve = curve.reserve(state.supply);
    let released = state
        .reserve
        .checked_sub(new_reserve)
        .map_err(StdError::overflow)?;
    state.reserve = new_reserve;
    CURVE_STATE.save(deps.storage, &state)?;

    // create a new claim for the released reserve Amount
    let config = CONFIG.load(deps.storage)?;
    CLAIMS.create_claim(
        deps.storage,
        &info.sender,
        amount,
        config.unbonding_period.after(&block),
    )?;

    let res = Response::new()
        .add_attribute("from", info.sender)
        .add_attribute("supply", amount)
        .add_attribute("reserve", released);
    Ok(res)
}

// Claim all reserve tokens that are past the unbonding period for a user.
pub fn claim_tokens(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // check how much to send - min(balance, claims[sender]), and reduce the claim
    // Ensure we have enough balance to cover this and only send some claims if that is all we can cover
    let to_send = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;
    if to_send == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    let state = CURVE_STATE.load(deps.storage)?;

    // now send the HALO tokens to the sender
    let res = Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.reserve_denom,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.into(),
                amount: to_send,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "claim"),
            ("amount", to_send.to_string().as_str()),
        ]);
    Ok(res)
}

pub fn execute_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::ZeroAmount {});
    }

    // lower balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    if token_info.mint.is_none() || token_info.mint.as_ref().unwrap().minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    token_info.total_supply += amount;
    if let Some(limit) = token_info.get_cap() {
        if token_info.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &token_info)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let rcpt_addr = deps.api.addr_validate(&contract)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount)
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract)?,
        );
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    // default implementation stores curve info as enum, you can do something else in a derived
    // contract and just pass in your custom curve to do_execute
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    do_query(deps, env, msg, curve_fn)
}

/// We pull out logic here, so we can import this from another contract and set a different Curve.
/// This contacts sets a curve with an enum in InstantitateMsg and stored in state, but you may want
/// to use custom math not included - make this easily reusable
pub fn do_query(deps: Deps, _env: Env, msg: QueryMsg, curve_fn: CurveFn) -> StdResult<Binary> {
    match msg {
        // custom queries
        QueryMsg::CurveInfo {} => to_binary(&query_curve_info(deps, curve_fn)?),
        QueryMsg::Claims { address } => Ok(to_binary(
            &CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)?,
        )?),
        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
    }
}

pub fn query_curve_info(deps: Deps, curve_fn: CurveFn) -> StdResult<CurveInfoResponse> {
    let CurveState {
        reserve,
        supply,
        reserve_denom,
        decimals,
    } = CURVE_STATE.load(deps.storage)?;

    // This we can get from the local digits stored in instantiate
    let curve = curve_fn(decimals);
    let spot_price = curve.spot_price(supply);

    Ok(CurveInfoResponse {
        reserve,
        supply,
        spot_price,
        reserve_denom,
    })
}
