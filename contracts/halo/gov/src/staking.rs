use crate::error::ContractError;
use crate::state::{
    bank_read, bank_store, config_read, config_store, poll_read, poll_voter_store, state_read,
    state_store, Config, Poll, State, TokenManager, CLAIMS,
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

    let mut token_manager = bank_read(deps.storage).may_load(key)?.unwrap_or_default();
    let config: Config = config_store(deps.storage).load()?;
    let mut state: State = state_store(deps.storage).load()?;

    // balance already increased, so subtract deposit amount
    let total_balance = query_token_balance(
        &deps.querier,
        config.halo_token,
        env.contract.address.clone(),
    )?
    .checked_sub(state.total_deposit + amount)?;

    let share = if total_balance.is_zero() || state.total_share.is_zero() {
        amount
    } else {
        amount.multiply_ratio(state.total_share, total_balance)
    };

    token_manager.share += share;
    state.total_share += share;

    state_store(deps.storage).save(&state)?;
    bank_store(deps.storage).save(key, &token_manager)?;

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

    if let Some(mut token_manager) = bank_read(deps.storage).may_load(key)? {
        let config: Config = config_store(deps.storage).load()?;
        let mut state: State = state_store(deps.storage).load()?;

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

            bank_store(deps.storage).save(key, &token_manager)?;

            state.total_share = Uint128::from(total_share - withdraw_share);
            state_store(deps.storage).save(&state)?;

            CLAIMS.create_claim(
                deps.storage,
                &info.sender.clone(),
                Uint128::from(withdraw_amount),
                config.unbonding_period.after(&env.block),
            )?;
            Ok(Response::default())
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
    let config: Config = config_store(deps.storage).load()?;
    let mut balance = deps
        .querier
        .query_balance(&env.contract.address, &config.halo_token)?;
    // check how much to send - min(balance, claims[sender]), and reduce the claim
    // Ensure we have enough balance to cover this and only send some claims if that is all we can cover
    let to_send =
        CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, Some(balance.amount))?;
    if to_send == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    // transfer tokens to the sender
    balance.amount = to_send;
    send_tokens(deps, &config.halo_token, &info.sender, to_send, "claim")
}

// removes not in-progress poll voter info & unlock tokens
// and returns the largest locked amount in participated polls.
fn compute_locked_balance(
    storage: &mut dyn Storage,
    token_manager: &mut TokenManager,
    voter: &Addr,
) -> u128 {
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll: Poll = poll_read(storage).load(&poll_id.to_be_bytes()).unwrap();

        if poll.status != PollStatus::InProgress {
            // remove voter info from the poll
            poll_voter_store(storage, *poll_id).remove(voter.as_bytes());
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

fn send_tokens(
    _deps: DepsMut,
    asset_token: &Addr,
    recipient: &Addr,
    amount: Uint128,
    action: &str,
) -> Result<Response, ContractError> {
    let contract_human = asset_token.to_string();
    let recipient_human = recipient.to_string();

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_human,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient_human.clone(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", action),
            ("recipient", recipient_human.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn query_staker(deps: Deps, env: Env, address: String) -> StdResult<StakerResponse> {
    let addr_raw = deps.api.addr_validate(&address)?;
    let config: Config = config_read(deps.storage).load()?;
    let state: State = state_read(deps.storage).load()?;
    let mut token_manager = bank_read(deps.storage)
        .may_load(addr_raw.as_bytes())?
        .unwrap_or_default();

    // filter out not in-progress polls
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll: Poll = poll_read(deps.storage)
            .load(&poll_id.to_be_bytes())
            .unwrap();

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
