use crate::contract::{DEFAULT_CLAIM_LIMIT, SECONDS_PER_WEEK};
use crate::querier::{
    query_address_voting_balance_at_timestamp, query_total_voting_balance_at_timestamp,
};
use crate::state::{Config, USER_LAST_CLAIMED_FEE_TIMESTAMP, WEEKLY_TOKEN_DISTRIBUTION};
use angel_core::utils::{calc_range_end, calc_range_start};
use cosmwasm_std::{Addr, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

pub fn compute_claimable(
    deps: Deps,
    env: Env,
    config: &Config,
    user: &Addr,
    limit: Option<u32>,
    start_after: Option<u64>,
) -> StdResult<(u64, u64, Uint128)> {
    // Set the initlal last claimed fee timestamp
    // if the user has never claimed a fee before, it will be unwrapped to default (0).
    let initial_last_claimed_fee_timestamp = start_after.unwrap_or(
        USER_LAST_CLAIMED_FEE_TIMESTAMP
            .may_load(deps.storage, user.clone())?
            .unwrap_or_default(),
    );

    // Copy the initial_last_claimed_fee_timestamp.
    // We don't want to mutate the initial_last_claimed_fee_timestamp
    // so that we can send it back unchanged in the response.
    let mut last_claimed_fee_timestamp = initial_last_claimed_fee_timestamp;

    // Increaes the start_time by SECONDS_PER_WEEK to get to the next week.
    // If the user has never collected a fee, this will be set to
    // SECONDS_PER_WEEK
    let start_time = last_claimed_fee_timestamp + SECONDS_PER_WEEK;

    // If env.block.time.seconds is divisible by SECONDS_PER_WEEK
    // (which means right at the cut off)
    // go to the previous week.
    let end_time =
        env.block.time.seconds() / SECONDS_PER_WEEK * SECONDS_PER_WEEK - SECONDS_PER_WEEK;

    // Set limit, or DEFAULT_CLAIM_LIMIT if undefined.
    let limit = limit.unwrap_or(DEFAULT_CLAIM_LIMIT) as usize;

    // Do a range query over WEEKLY_TOKEN_DISTRIBUTION
    let token_distributions = WEEKLY_TOKEN_DISTRIBUTION
        .range(
            deps.storage,
            calc_range_start(Some(start_time)).map(|v| Bound::exclusive(v)),
            calc_range_end(Some(end_time)).map(|v| Bound::inclusive(v)),
            Order::Ascending,
        )
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((k, v))
        })
        .collect::<StdResult<Vec<_>>>()?;

    // Initialize claim_amount as set to 0
    let mut claim_amount = Uint128::zero();

    for (timestamp, distributed_amount) in token_distributions {
        // For each pair of timestamp and distributed_amount in token_distributions,
        // - update last_claimed_fee_timestamp.
        // - get the total voting balance at the corresponding time.
        // - get the uer's voting balance at the corresponding time.
        // - increase claim_amount by distributed_amount * (user_voting_balance / total_voting_balance)

        // Update last_claimed_fee_timestamp
        last_claimed_fee_timestamp = timestamp;

        // Get the total voting balance at this point in time
        let total_voting_balance = query_total_voting_balance_at_timestamp(
            &deps.querier,
            &config.ve_token,
            Some(timestamp),
        )?;

        // Get the user's voting balance at this point in time
        let user_voting_balance = query_address_voting_balance_at_timestamp(
            &deps.querier,
            &config.ve_token,
            Some(timestamp),
            user,
        )?;

        // Increment claim_ammount accordingly.
        claim_amount +=
            distributed_amount.multiply_ratio(user_voting_balance, total_voting_balance);
    }

    // Return the initial_last_claimed_fee_timestamp,
    // the last_claimed_fee_timestamp
    // and the claimed_amount
    Ok((
        initial_last_claimed_fee_timestamp,
        last_claimed_fee_timestamp,
        claim_amount,
    ))
}
