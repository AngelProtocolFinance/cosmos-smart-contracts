use crate::error::ContractError;
use crate::staking::{
    claim_voting_tokens, query_staker, stake_voting_tokens, withdraw_voting_tokens,
};
use crate::state::{
    bank_read, bank_store, config_read, config_store, poll_indexer_store, poll_read, poll_store,
    poll_voter_read, poll_voter_store, read_poll_voters, read_polls, read_tmp_poll_id, state_read,
    state_store, store_tmp_poll_id, Config, ExecuteData, Poll, State, CLAIMS,
};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw0::Duration;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use halo_token::common::OrderBy;
use halo_token::gov::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, PollExecuteMsg,
    PollResponse, PollStatus, PollsResponse, QueryMsg, StateResponse, VoteOption, VoterInfo,
    VotersResponse, VotersResponseItem,
};
use terraswap::querier::query_token_balance;

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const POLL_EXECUTE_REPLY_ID: u64 = 1;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let quorum = Decimal::percent(msg.quorum);
    let threshold = Decimal::percent(msg.threshold);
    validate_decimal(quorum)?;
    validate_decimal(threshold)?;

    let config = Config {
        halo_token: deps.api.addr_validate(&msg.halo_token)?,
        owner: info.sender,
        quorum,
        threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        unbonding_period: Duration::Time(msg.unbonding_period), // secconds of unbonding
        gov_hodler: deps.api.addr_validate(&msg.gov_hodler)?,
    };

    let state = State {
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };

    config_store(deps.storage).save(&config)?;
    state_store(deps.storage).save(&state)?;

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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ExecutePollMsgs { poll_id } => execute_poll_messages(deps, env, info, poll_id),
        ExecuteMsg::RegisterContracts { halo_token } => register_contracts(deps, info, halo_token),
        ExecuteMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period,
            unbonding_period,
            gov_hodler,
        } => update_config(
            deps,
            info,
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period,
            unbonding_period,
            gov_hodler,
        ),
        ExecuteMsg::WithdrawVotingTokens { amount } => {
            withdraw_voting_tokens(deps, env, info, amount)
        }
        ExecuteMsg::ClaimVotingTokens {} => claim_voting_tokens(deps, env, info),
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => cast_vote(deps, env, info, poll_id, vote, amount),
        ExecuteMsg::EndPoll { poll_id } => end_poll(deps, env, poll_id),
        ExecuteMsg::ExecutePoll { poll_id } => execute_poll(deps, env, poll_id),
        ExecuteMsg::SnapshotPoll { poll_id } => snapshot_poll(deps, env, poll_id),
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        POLL_EXECUTE_REPLY_ID => {
            let poll_id: u64 = read_tmp_poll_id(deps.storage)?;
            fail_poll(deps, poll_id)
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

pub fn register_contracts(
    deps: DepsMut,
    info: MessageInfo,
    halo_token: String,
) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.halo_token = deps.api.addr_validate(&halo_token)?;
    config_store(deps.storage).save(&config)?;

    Ok(Response::default())
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // only asset contract can execute this message
    let config: Config = config_read(deps.storage).load()?;
    if config.halo_token != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::StakeVotingTokens {}) => {
            let api = deps.api;
            stake_voting_tokens(
                deps,
                env,
                api.addr_validate(&cw20_msg.sender)?,
                cw20_msg.amount,
            )
        }
        Ok(Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            proposal_type,
            options,
        }) => create_poll(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            title,
            description,
            link,
            proposal_type,
            options,
        ),
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    quorum: Option<u64>,
    threshold: Option<u64>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
    unbonding_period: Option<u64>,
    gov_hodler: Option<String>,
) -> Result<Response, ContractError> {
    let api = deps.api;
    let mut config: Config = config_read(deps.storage).load()?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = api.addr_validate(&owner)?;
    }

    if let Some(quorum) = quorum {
        config.quorum = Decimal::percent(quorum);
    }

    if let Some(threshold) = threshold {
        config.threshold = Decimal::percent(threshold);
    }

    if let Some(voting_period) = voting_period {
        config.voting_period = voting_period;
    }

    if let Some(timelock_period) = timelock_period {
        config.timelock_period = timelock_period;
    }

    if let Some(proposal_deposit) = proposal_deposit {
        config.proposal_deposit = proposal_deposit;
    }

    if let Some(period) = snapshot_period {
        config.snapshot_period = period;
    }

    if let Some(gov_hodler) = gov_hodler {
        config.gov_hodler = deps.api.addr_validate(&gov_hodler)?;
    }

    if let Some(unbonding_period) = unbonding_period {
        // unbonding calculated in seconds
        config.unbonding_period = Duration::Time(unbonding_period)
    }

    config_store(deps.storage).save(&config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

/// validate_title returns an error if the title is invalid
fn validate_title(title: &str) -> StdResult<()> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(StdError::generic_err("Title too short"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(StdError::generic_err("Title too long"))
    } else {
        Ok(())
    }
}

/// validate_description returns an error if the description is invalid
fn validate_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Description too long"))
    } else {
        Ok(())
    }
}

/// validate_link returns an error if the link is invalid
fn validate_link(link: &Option<String>) -> StdResult<()> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(StdError::generic_err("Link too short"))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(StdError::generic_err("Link too long"))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// validate_decimal returns an error if it is invalid
/// (we require 0-1)
fn validate_decimal(d: Decimal) -> StdResult<()> {
    if d > Decimal::one() {
        Err(StdError::generic_err("decimal must be 0 to 1"))
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
/// create a new poll
pub fn create_poll(
    deps: DepsMut,
    env: Env,
    proposer: String,
    deposit_amount: Uint128,
    title: String,
    description: String,
    link: Option<String>,
    proposal_type: Option<String>,
    options: Option<Vec<PollExecuteMsg>>,
) -> Result<Response, ContractError> {
    validate_title(&title)?;
    validate_description(&description)?;
    validate_link(&link)?;

    let config: Config = config_store(deps.storage).load()?;
    if deposit_amount < config.proposal_deposit {
        return Err(ContractError::InsufficientProposalDeposit(
            config.proposal_deposit.u128(),
        ));
    }

    let mut state: State = state_store(deps.storage).load()?;
    let poll_id = state.poll_count + 1;

    // Increase poll count & total deposit amount
    state.poll_count += 1;
    state.total_deposit += deposit_amount;
    let contract = if proposal_type == Some("registrar".to_string()) {
        config.registrar_contract
    } else {
        env.contract.address.clone()
    };

    let mut data_list: Vec<ExecuteData> = vec![];
    let all_execute_data = if let Some(exe_msgs) = options {
        for msgs in exe_msgs {
            let execute_data = ExecuteData {
                order: msgs.order,
                contract: contract.clone(),
                msg: msgs.msg,
                funding_goal: msgs.funding_goal,
                fund_rotation: msgs.fund_rotation,
                split_to_liquid: msgs.split_to_liquid,
                treasury_tax_rate: msgs.treasury_tax_rate,
            };
            data_list.push(execute_data)
        }
        Some(data_list)
    } else {
        None
    };

    let sender_address_raw = deps.api.addr_validate(&proposer)?;
    // store the current staked amount for quorum calculation
    let staked_amount =
        query_token_balance(&deps.querier, config.halo_token, env.contract.address)?
            .checked_sub(state.total_deposit)?;

    let new_poll = Poll {
        id: poll_id,
        creator: sender_address_raw,
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        end_height: env.block.height + config.voting_period,
        title,
        description,
        link,
        proposal_type,
        execute_data: all_execute_data,
        deposit_amount,
        total_balance_at_end_poll: None,
        staked_amount: Some(staked_amount),
    };

    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &new_poll)?;
    poll_indexer_store(deps.storage, &PollStatus::InProgress)
        .save(&poll_id.to_be_bytes(), &true)?;

    state_store(deps.storage).save(&state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "create_poll"),
        ("creator", new_poll.creator.as_str()),
        ("poll_id", &poll_id.to_string()),
        ("end_height", new_poll.end_height.to_string().as_str()),
    ]))
}

/*
 * Ends a poll.
 */
pub fn end_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if a_poll.end_height > env.block.height {
        return Err(ContractError::PollVotingPeriod {});
    }

    let no = a_poll.no_votes.u128();
    let yes = a_poll.yes_votes.u128();

    let tallied_weight = yes + no;

    let mut poll_status = PollStatus::Rejected;
    let mut rejected_reason = "";
    let mut passed = false;

    let mut messages: Vec<CosmosMsg> = vec![];
    let config: Config = config_read(deps.storage).load()?;
    let mut state: State = state_read(deps.storage).load()?;

    let (quorum, staked_weight) = if state.total_share.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else if let Some(staked_amount) = a_poll.staked_amount {
        (
            Decimal::from_ratio(tallied_weight, staked_amount),
            staked_amount,
        )
    } else {
        let staked_weight = query_token_balance(
            &deps.querier,
            config.halo_token.clone(),
            env.contract.address,
        )?
        .checked_sub(state.total_deposit)?;

        (
            Decimal::from_ratio(tallied_weight, staked_weight),
            staked_weight,
        )
    };

    if tallied_weight == 0 || quorum < config.quorum {
        // Quorum: More than quorum of the total staked tokens at the end of the voting
        // period need to have participated in the vote.
        rejected_reason = "Quorum not reached";
    } else {
        if Decimal::from_ratio(yes, tallied_weight) > config.threshold {
            //Threshold: More than 50% of the tokens that participated in the vote
            // (after excluding “Abstain” votes) need to have voted in favor of the proposal (“Yes”).
            poll_status = PollStatus::Passed;
            passed = true;
        } else {
            rejected_reason = "Threshold not reached";
        }

        // Refunds deposit only when quorum is reached
        if !a_poll.deposit_amount.is_zero() {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.halo_token.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: a_poll.creator.to_string(),
                    amount: a_poll.deposit_amount,
                })?,
            }))
        }
    }

    // Decrease total deposit amount
    state.total_deposit = state.total_deposit.checked_sub(a_poll.deposit_amount)?;
    state_store(deps.storage).save(&state)?;

    // Update poll indexer
    poll_indexer_store(deps.storage, &PollStatus::InProgress).remove(&a_poll.id.to_be_bytes());
    poll_indexer_store(deps.storage, &poll_status).save(&a_poll.id.to_be_bytes(), &true)?;

    // Update poll status
    a_poll.status = poll_status;
    a_poll.total_balance_at_end_poll = Some(staked_weight);
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "end_poll"),
        ("poll_id", &poll_id.to_string()),
        ("rejected_reason", rejected_reason),
        ("passed", &passed.to_string()),
    ]))
}

/*
 * Execute a msgs of passed poll as one submsg to catch failures
 */
pub fn execute_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if a_poll.end_height + config.timelock_period > env.block.height {
        return Err(ContractError::TimelockNotExpired {});
    }

    store_tmp_poll_id(deps.storage, a_poll.id)?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_error(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::ExecutePollMsgs { poll_id })?,
            funds: vec![],
        }),
        POLL_EXECUTE_REPLY_ID,
    )))
}

/*
 * Execute a msgs of a poll
 */
pub fn execute_poll_messages(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    poll_indexer_store(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_store(deps.storage, &PollStatus::Executed).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Executed;
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(all_msgs) = a_poll.execute_data {
        let mut msgs = all_msgs;
        msgs.sort();
        for msg in msgs {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: msg.contract.to_string(),
                msg: msg.msg,
                funds: vec![],
            }));
        }
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "execute_poll"),
        ("poll_id", poll_id.to_string().as_str()),
    ]))
}

/*
 * Set the status of a poll to Failed if execute_poll fails
 */
pub fn fail_poll(deps: DepsMut, poll_id: u64) -> Result<Response, ContractError> {
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    poll_indexer_store(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_store(deps.storage, &PollStatus::Failed).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Failed;
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "fail_poll"),
        ("poll_id", poll_id.to_string().as_str()),
    ]))
}

/// SnapshotPoll is used to take a snapshot of the staked amount for quorum calculation
pub fn snapshot_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end > config.snapshot_period {
        return Err(ContractError::SnapshotHeight {});
    }

    if a_poll.staked_amount.is_some() {
        return Err(ContractError::SnapshotAlreadyOccurred {});
    }

    // store the current staked amount for quorum calculation
    let state: State = state_store(deps.storage).load()?;

    let staked_amount =
        query_token_balance(&deps.querier, config.halo_token, env.contract.address)?
            .checked_sub(state.total_deposit)?;

    a_poll.staked_amount = Some(staked_amount);

    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "snapshot_poll"),
        attr("poll_id", poll_id.to_string().as_str()),
        attr("staked_amount", staked_amount.to_string().as_str()),
    ]))
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;
    let state = state_read(deps.storage).load()?;
    if poll_id == 0 || state.poll_count < poll_id {
        return Err(ContractError::PollNotFound {});
    }

    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;
    if a_poll.status != PollStatus::InProgress || env.block.height > a_poll.end_height {
        return Err(ContractError::PollNotInProgress {});
    }

    // Check the voter already has a vote on the poll
    if poll_voter_read(deps.storage, poll_id)
        .load(info.sender.as_bytes())
        .is_ok()
    {
        return Err(ContractError::AlreadyVoted {});
    }

    let key = &info.sender.as_bytes();
    let mut token_manager = bank_read(deps.storage).may_load(key)?.unwrap_or_default();

    // convert share to amount
    let total_share = state.total_share;
    let total_balance =
        query_token_balance(&deps.querier, config.halo_token, env.contract.address)?
            .checked_sub(state.total_deposit)?;

    if token_manager
        .share
        .multiply_ratio(total_balance, total_share)
        < amount
    {
        return Err(ContractError::InsufficientStaked {});
    }

    // update tally info
    if VoteOption::Yes == vote {
        a_poll.yes_votes += amount;
    } else {
        a_poll.no_votes += amount;
    }

    let vote_info = VoterInfo {
        vote,
        balance: amount,
    };
    token_manager
        .locked_balance
        .push((poll_id, vote_info.clone()));
    bank_store(deps.storage).save(key, &token_manager)?;

    // store poll voter && and update poll data
    poll_voter_store(deps.storage, poll_id).save(info.sender.as_bytes(), &vote_info)?;

    // processing snapshot
    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end < config.snapshot_period && a_poll.staked_amount.is_none() {
        a_poll.staked_amount = Some(total_balance);
    }

    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "cast_vote"),
        ("poll_id", poll_id.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
        ("voter", info.sender.as_str()),
        ("vote_option", vote_info.vote.to_string().as_str()),
    ]))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_binary(&query_state(deps)?)?),
        QueryMsg::Claims { address } => Ok(to_binary(
            &CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)?,
        )?),
        QueryMsg::Staker { address } => Ok(to_binary(&query_staker(deps, env, address)?)?),
        QueryMsg::Poll { poll_id } => Ok(to_binary(&query_poll(deps, poll_id)?)?),
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_polls(
            deps,
            filter,
            start_after,
            limit,
            order_by,
        )?)?),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_voters(
            deps,
            poll_id,
            start_after,
            limit,
            order_by,
        )?)?),
    }
}

fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        halo_token: config.halo_token.to_string(),
        quorum: config.quorum,
        threshold: config.threshold,
        voting_period: config.voting_period,
        timelock_period: config.timelock_period,
        proposal_deposit: config.proposal_deposit,
        snapshot_period: config.snapshot_period,
        unbonding_period: config.unbonding_period,
    })
}

fn query_state(deps: Deps) -> Result<StateResponse, ContractError> {
    let state: State = state_read(deps.storage).load()?;
    Ok(StateResponse {
        poll_count: state.poll_count,
        total_share: state.total_share,
        total_deposit: state.total_deposit,
    })
}

fn query_poll(deps: Deps, poll_id: u64) -> Result<PollResponse, ContractError> {
    let poll = match poll_read(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let mut data_list: Vec<PollExecuteMsg> = vec![];

    Ok(PollResponse {
        id: poll.id,
        creator: poll.creator.to_string(),
        status: poll.status,
        end_height: poll.end_height,
        title: poll.title,
        description: poll.description,
        link: poll.link,
        proposal_type: poll.proposal_type,
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            for msg in exe_msgs {
                let execute_data = PollExecuteMsg {
                    order: msg.order,
                    msg: msg.msg,
                    funding_goal: msg.funding_goal,
                    fund_rotation: msg.fund_rotation,
                    split_to_liquid: msg.split_to_liquid,
                    treasury_tax_rate: msg.treasury_tax_rate,
                };
                data_list.push(execute_data)
            }
            Some(data_list)
        } else {
            None
        },
        yes_votes: poll.yes_votes,
        no_votes: poll.no_votes,
        staked_amount: poll.staked_amount,
        total_balance_at_end_poll: poll.total_balance_at_end_poll,
    })
}

fn query_polls(
    deps: Deps,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<PollsResponse, ContractError> {
    let polls = read_polls(deps.storage, filter, start_after, limit, order_by)?;

    let poll_responses: StdResult<Vec<PollResponse>> = polls
        .iter()
        .map(|poll| {
            Ok(PollResponse {
                id: poll.id,
                creator: poll.creator.to_string(),
                status: poll.status.clone(),
                end_height: poll.end_height,
                title: poll.title.to_string(),
                description: poll.description.to_string(),
                link: poll.link.clone(),
                proposal_type: poll.proposal_type.clone(),
                deposit_amount: poll.deposit_amount,
                execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
                    let mut data_list: Vec<PollExecuteMsg> = vec![];

                    for msg in exe_msgs {
                        let execute_data = PollExecuteMsg {
                            order: msg.order,
                            msg: msg.msg,
                            funding_goal: msg.funding_goal,
                            fund_rotation: msg.fund_rotation,
                            split_to_liquid: msg.split_to_liquid,
                            treasury_tax_rate: msg.treasury_tax_rate,
                        };
                        data_list.push(execute_data)
                    }
                    Some(data_list)
                } else {
                    None
                },
                yes_votes: poll.yes_votes,
                no_votes: poll.no_votes,
                staked_amount: poll.staked_amount,
                total_balance_at_end_poll: poll.total_balance_at_end_poll,
            })
        })
        .collect();

    Ok(PollsResponse {
        polls: poll_responses?,
    })
}

fn query_voters(
    deps: Deps,
    poll_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<VotersResponse, ContractError> {
    let poll: Poll = match poll_read(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else if let Some(start_after) = start_after {
        read_poll_voters(
            deps,
            deps.storage,
            poll_id,
            Some(deps.api.addr_validate(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_poll_voters(deps, deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: voter_info.0.to_string(),
                vote: voter_info.1.vote.clone(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(VotersResponse {
        voters: voters_response?,
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
