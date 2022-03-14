use crate::error::ContractError;
use crate::state::{
    read_bank, store_bank, read_config, read_poll, read_state,
    store_state, Config, Poll, State, TokenManager, CLAIMS, remove_poll_voter,
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use halo_token::gov::{PollStatus, StakerResponse};
use terraswap::querier::query_token_balance;

pub fn stake_voting_tokens(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InsufficientFunds {});
    }
    let key = sender.as_bytes();

    let mut token_manager = read_bank(deps.storage, &key)?.unwrap_or_default();
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    // balance already increased, so subtract deposit amount
    let total_balance =
        query_token_balance(&deps.querier, config.halo_token, env.contract.address)?
            .checked_sub(state.total_deposit + amount)?;

    let share = if total_balance.is_zero() || state.total_share.is_zero() {
        amount
    } else {
        amount.multiply_ratio(state.total_share, total_balance)
    };

    token_manager.share += share;
    state.total_share += share;

    store_state(deps.storage, &state)?;
    store_bank(deps.storage, &key, &token_manager)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "staking"),
        ("sender", sender.as_str()),
        ("share", share.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

// Withdraw amount if not staked. By default all funds will be withdrawn.
pub fn withdraw_voting_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let sender_address_raw = info.sender.clone();
    let key = sender_address_raw.as_bytes();

    if let Some(mut token_manager) = read_bank(deps.storage, &key)? {
        let config: Config = read_config(deps.storage)?;
        let mut state: State = read_state(deps.storage)?;

        // Load total share & total balance except proposal deposit amount
        let total_share = state.total_share.u128();
        let total_balance = query_token_balance(
            &deps.querier,
            config.halo_token.clone(),
            env.contract.address.clone(),
        )?
        .checked_sub(state.total_deposit)?
        .u128();

        let locked_balance =
            compute_locked_balance(deps.storage, &mut token_manager, &sender_address_raw);
        let locked_share = locked_balance * total_share / total_balance;
        let user_share = token_manager.share.u128();

        let withdraw_share = amount
            .map(|v| std::cmp::max(v.multiply_ratio(total_share, total_balance).u128(), 1u128))
            .unwrap_or_else(|| user_share - locked_share);
        let withdraw_amount = amount
            .map(|v| v.u128())
            .unwrap_or_else(|| withdraw_share * total_balance / total_share);

        if locked_share + withdraw_share > user_share {
            Err(ContractError::InvalidWithdrawAmount {})
        } else {
            let share = user_share - withdraw_share;
            token_manager.share = Uint128::from(share);

            store_bank(deps.storage, &key, &token_manager)?;

            state.total_share = Uint128::from(total_share - withdraw_share);
            store_state(deps.storage , &state)?;

            // create claim on withdrawn HALO tokens
            CLAIMS.create_claim(
                deps.storage,
                &info.sender,
                Uint128::from(withdraw_amount),
                config.unbonding_period.after(&env.block),
            )?;

            // withdraw HALO tokens to the Gov Claims Hodler contract
            withdraw_tokens(
                deps,
                Uint128::from(withdraw_amount),
                config.gov_hodler.to_string(),
            )
        }
    } else {
        Err(ContractError::NothingStaked {})
    }
}

// Claim all tokens that are past the unbonding period for a user. By default all claimable funds will be withdrawn.
pub fn claim_voting_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check how much to send - min(balance, claims[sender]), and reduce the claim
    // Ensure we have enough balance to cover this and only send some claims if that is all we can cover
    let to_send = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;
    if to_send == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    // send message to the Gov Claims Hodler to transfer HALO tokens to the sender
    claim_tokens(
        deps,
        info.sender.to_string(),
        to_send,
        config.gov_hodler.to_string(),
    )
}

// removes not in-progress poll voter info & unlock tokens
// and returns the largest locked amount in participated polls.
fn compute_locked_balance(
    storage: &mut dyn Storage,
    token_manager: &mut TokenManager,
    voter: &Addr,
) -> u128 {
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll: Poll = read_poll(storage, &poll_id.to_be_bytes()).unwrap();

        if poll.status != PollStatus::InProgress {
            // remove voter info from the poll
            remove_poll_voter(storage, *poll_id, voter);
        }

        poll.status == PollStatus::InProgress
    });

    token_manager
        .locked_balance
        .iter()
        .map(|(_, v)| v.balance.u128())
        .max()
        .unwrap_or_default()
}

fn withdraw_tokens(
    deps: DepsMut,
    amount: Uint128,
    gov_hodler: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.halo_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: gov_hodler,
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "withdraw_to_gov_hodler"),
            ("amount", amount.to_string().as_str()),
        ]))
}

fn claim_tokens(
    _deps: DepsMut,
    recipient: String,
    amount: Uint128,
    gov_hodler: String,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: gov_hodler,
            msg: to_binary(&halo_token::gov_hodler::ExecuteMsg::ClaimHalo {
                recipient: recipient.clone(),
                amount,
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "claim_from_gov_hodler")
        .add_attribute("recipient", recipient.clone())
        .add_attribute("amount", amount.to_string()))
}

pub fn query_staker(deps: Deps, env: Env, address: String) -> StdResult<StakerResponse> {
    let addr_raw = deps.api.addr_validate(&address)?;
    let config: Config = read_config(deps.storage)?;
    let state: State = read_state(deps.storage)?;
    let mut token_manager = read_bank(deps.storage, &addr_raw.as_bytes())?
        .unwrap_or_default();

    // filter out not in-progress polls
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll: Poll = read_poll(deps.storage, &poll_id.to_be_bytes()).unwrap();
        poll.status == PollStatus::InProgress
    });

    let total_balance =
        query_token_balance(&deps.querier, config.halo_token, env.contract.address)?
            .checked_sub(state.total_deposit)?;

    Ok(StakerResponse {
        balance: if !state.total_share.is_zero() {
            token_manager
                .share
                .multiply_ratio(total_balance, state.total_share)
        } else {
            Uint128::zero()
        },
        share: token_manager.share,
        locked_balance: token_manager.locked_balance,
        claims: CLAIMS
            .query_claims(deps, &deps.api.addr_validate(&address)?)?
            .claims,
    })
}
