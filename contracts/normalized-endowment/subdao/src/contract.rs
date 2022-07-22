use crate::querier::{
    query_address_voting_balance_at_timestamp, query_total_voting_balance_at_timestamp,
};
use crate::state::{
    config_read, config_store, poll_indexer_store, poll_read, poll_store, poll_voter_read,
    poll_voter_store, read_poll_voters, read_polls, state_read, state_store, Config, ExecuteData,
    Poll, State, DONATION_MATCH,
};
use angel_core::common::OrderBy;
use angel_core::errors::dao::ContractError;
use angel_core::messages::accounts::QueryMsg::GetProfile;
use angel_core::messages::donation_match::InstantiateMsg as DonationMatchInstantiateMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::messages::subdao::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, PollExecuteMsg,
    PollResponse, PollStatus, PollsResponse, QueryMsg, StateResponse, VoteOption, VoterInfo,
    VotersResponse, VotersResponseItem,
};
use angel_core::messages::subdao_token::InstantiateMsg as DaoTokenInstantiateMsg;
use angel_core::responses::accounts::ProfileResponse as EndowProfileResponse;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::{DaoToken, DonationMatch, EndowmentType};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128,
    WasmMsg, WasmQuery,
};
use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg};

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;

    let config = Config {
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        owner: info.sender,
        dao_token: Addr::unchecked(""),
        ve_token: Addr::unchecked(""),
        swap_factory: Addr::unchecked(""),
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: msg.expiration_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let state = State {
        contract_addr: deps.api.addr_validate(env.contract.address.as_str())?,
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };

    config_store(deps.storage).save(&config)?;
    state_store(deps.storage).save(&state)?;

    Ok(Response::default()
        .add_attribute("dao_addr", env.contract.address.to_string())
        .add_submessages(
            build_dao_token_messages(deps, msg.token, msg.endow_type, msg.endow_owner).unwrap(),
        ))
}

pub fn build_dao_token_messages(
    deps: DepsMut,
    token: DaoToken,
    endow_type: EndowmentType,
    endow_owner: String,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut submsgs: Vec<SubMsg> = vec![];
    let config: Config = config_read(deps.storage).load()?;
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    match (token, endow_type) {
        // Option #1. User can set an existing CW20 token as the DAO's Token
        (DaoToken::ExistingCw20(contract_addr), EndowmentType::Normal) => {
            // Check existing token is valid/accepted
            if !registrar_config
                .accepted_tokens
                .cw20_valid(contract_addr.to_string())
            {
                return Err(ContractError::NotInApprovedCoins {});
            }

            let contract_addr = deps.api.addr_validate(&contract_addr)?;
            let mut config: Config = config_store(deps.storage).load()?;
            config.dao_token = contract_addr;
            config_store(deps.storage).save(&config)?;
        }
        // Option #2. Create a basic CW20 token contract with a fixed supply
        (
            DaoToken::NewCw20 {
                initial_supply,
                name,
                symbol,
            },
            EndowmentType::Normal,
        ) => submsgs.push(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: registrar_config.subdao_token_code.unwrap(),
                admin: None,
                label: "new endowment dao token(cw20) contract".to_string(),
                msg: to_binary(&cw20_base::msg::InstantiateMsg {
                    name,
                    symbol,
                    decimals: 6,
                    initial_balances: vec![Cw20Coin {
                        address: endow_owner.to_string(),
                        amount: initial_supply,
                    }],
                    mint: None,
                    marketing: None,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }),
        // Option #3 (for all Non-Charity Endowments). Create a CW20 token with supply controlled by a bonding curve
        (
            DaoToken::BondingCurve {
                curve_type,
                name,
                symbol,
                decimals,
                reserve_denom,
                reserve_decimals,
                unbonding_period,
            },
            EndowmentType::Normal,
        ) => submsgs.push(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: registrar_config.subdao_token_code.unwrap(),
                admin: None,
                label: "new endowment dao token(bonding curve) contract".to_string(),
                msg: to_binary(&DaoTokenInstantiateMsg {
                    curve_type,
                    name,
                    symbol,
                    decimals,
                    reserve_denom,
                    reserve_decimals,
                    unbonding_period,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }),
        // Option #3 (for all Charity Endowments). Create a CW20 token with supply controlled by a bonding curve
        (
            DaoToken::BondingCurve {
                curve_type,
                name,
                symbol,
                decimals: _,
                reserve_denom: _,
                reserve_decimals: _,
                unbonding_period: _,
            },
            EndowmentType::Charity,
        ) => {
            // setup DAO token contract
            let halo_token = match registrar_config.halo_token.clone() {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "Registrar's HALO token address is empty".to_string(),
                    }))
                }
            };
            submsgs.push(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: registrar_config.subdao_token_code.unwrap(),
                    admin: None,
                    label: "new endowment dao token(bonding curve) contract".to_string(),
                    msg: to_binary(&DaoTokenInstantiateMsg {
                        curve_type,
                        name,
                        symbol,
                        decimals: 6,
                        reserve_denom: halo_token,
                        reserve_decimals: 6,
                        unbonding_period: 21,
                    })?,
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
        }
        (_, _) => return Err(ContractError::InvalidInputs {}),
    }
    Ok(submsgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::RegisterContracts {
            ve_token,
            swap_factory,
        } => register_contracts(deps, info, ve_token, swap_factory),
        ExecuteMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        } => update_config(
            deps,
            info,
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        ),
        ExecuteMsg::CastVote { poll_id, vote } => cast_vote(deps, env, info, poll_id, vote),
        ExecuteMsg::EndPoll { poll_id } => end_poll(deps, env, poll_id),
        ExecuteMsg::ExecutePoll { poll_id } => execute_poll(deps, env, poll_id),
        ExecuteMsg::ExpirePoll { poll_id } => expire_poll(deps, env, poll_id),
        _ => Err(ContractError::Unauthorized {}),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // only asset contract can execute this message
    let config: Config = config_read(deps.storage).load()?;
    if config.dao_token != deps.api.addr_validate(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execute_msgs,
        }) => create_poll(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            title,
            description,
            link,
            execute_msgs,
        ),
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => dao_token_reply(deps, env, msg.result),
        1 => donation_match_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

pub fn dao_token_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut dao_token_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate" {
                    for attrb in event.attributes {
                        if attrb.key == "_contract_address" {
                            dao_token_addr = attrb.value;
                        }
                    }
                }
            }

            // update the "dao_token" to be the new contract
            let mut config: Config = config_read(deps.storage).load()?;
            config.dao_token = deps.api.addr_validate(&dao_token_addr)?;
            config_store(deps.storage).save(&config)?;

            let mut res =
                Response::default().add_attribute("dao_token_addr", dao_token_addr.to_string());

            let registrar_config: RegistrarConfigResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.registrar_contract.to_string(),
                    msg: to_binary(&RegistrarConfig {})?,
                }))?;

            let endow_profile: EndowProfileResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.owner.to_string(),
                    msg: to_binary(&GetProfile {})?,
                }))?;

            let donation_match = DONATION_MATCH.may_load(deps.storage)?;

            // optional donation matching contract can be setup as long as a dao token is in place
            if endow_profile.endowment_type == EndowmentType::Normal {
                let match_code = match (&donation_match, registrar_config.donation_match_code) {
                    (Some(_), Some(match_code)) => Some(match_code),
                    (Some(_), None) => {
                        return Err(ContractError::Std(StdError::GenericErr {
                            msg: "No code id for donation matching contract".to_string(),
                        }))
                    }
                    (None, _) => None,
                };
                match donation_match {
                    Some(DonationMatch::HaloTokenReserve {}) => {
                        match (
                            registrar_config.halo_token,
                            registrar_config.halo_token_lp_contract,
                        ) {
                            (Some(reserve_addr), Some(lp_addr)) => {
                                res = res.add_submessage(SubMsg {
                                    id: 1,
                                    msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                                        code_id: match_code.unwrap(),
                                        admin: None,
                                        label: "new donation match contract".to_string(),
                                        msg: to_binary(&DonationMatchInstantiateMsg {
                                            reserve_token: reserve_addr,
                                            lp_pair: lp_addr,
                                            registrar_contract: config
                                                .registrar_contract
                                                .to_string(),
                                        })?,
                                        funds: vec![],
                                    }),
                                    gas_limit: None,
                                    reply_on: ReplyOn::Success,
                                });
                            }
                            (_, _) => {
                                return Err(ContractError::Std(StdError::GenericErr {
                                    msg: "HALO Token is not setup to be a reserve token"
                                        .to_string(),
                                }))
                            }
                        }
                    }
                    Some(DonationMatch::Cw20TokenReserve {
                        reserve_addr,
                        lp_addr,
                    }) => {
                        res = res.add_submessage(SubMsg {
                            id: 1,
                            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                                code_id: match_code.unwrap(),
                                admin: None,
                                label: "new donation match contract".to_string(),
                                msg: to_binary(&DonationMatchInstantiateMsg {
                                    reserve_token: reserve_addr,
                                    lp_pair: lp_addr,
                                    registrar_contract: config.registrar_contract.to_string(),
                                })?,
                                funds: vec![],
                            }),
                            gas_limit: None,
                            reply_on: ReplyOn::Success,
                        });
                    }
                    // Option #3: New CW20 Token to be added here...
                    // Some(3) => {)
                    _ => (),
                };
            }

            Ok(res)
        }
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn donation_match_reply(
    _deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut donation_match_contract_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate" {
                    for attrb in event.attributes {
                        if attrb.key == "_contract_address" {
                            donation_match_contract_addr = attrb.value;
                        }
                    }
                }
            }

            Ok(Response::default()
                .add_attribute("donation_match_contract", donation_match_contract_addr))
        }
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn register_contracts(
    deps: DepsMut,
    info: MessageInfo,
    ve_token: String,
    swap_factory: String,
) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;

    if config.owner.ne(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    config.ve_token = deps.api.addr_validate(&ve_token)?;
    config.swap_factory = deps.api.addr_validate(&swap_factory)?;
    config_store(deps.storage).save(&config)?;

    Ok(Response::default())
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    expiration_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> Result<Response, ContractError> {
    let api = deps.api;
    config_store(deps.storage).update(|mut config| {
        if config.owner != api.addr_validate(info.sender.as_str())? {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = api.addr_validate(&owner)?;
        }

        if let Some(quorum) = quorum {
            config.quorum = quorum;
        }

        if let Some(threshold) = threshold {
            config.threshold = threshold;
        }

        if let Some(voting_period) = voting_period {
            config.voting_period = voting_period;
        }

        if let Some(timelock_period) = timelock_period {
            config.timelock_period = timelock_period;
        }

        if let Some(expiration_period) = expiration_period {
            config.expiration_period = expiration_period;
        }

        if let Some(proposal_deposit) = proposal_deposit {
            config.proposal_deposit = proposal_deposit;
        }

        if let Some(period) = snapshot_period {
            config.snapshot_period = period;
        }

        Ok(config)
    })?;

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

/// validate_quorum returns an error if the quorum is invalid
/// (we require 0-1)
fn validate_quorum(quorum: Decimal) -> StdResult<()> {
    if quorum > Decimal::one() {
        Err(StdError::generic_err("quorum must be 0 to 1"))
    } else {
        Ok(())
    }
}

/// validate_threshold returns an error if the threshold is invalid
/// (we require 0-1)
fn validate_threshold(threshold: Decimal) -> StdResult<()> {
    if threshold > Decimal::one() {
        Err(StdError::generic_err("threshold must be 0 to 1"))
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
    execute_msgs: Option<Vec<PollExecuteMsg>>,
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

    let mut data_list: Vec<ExecuteData> = vec![];
    let all_execute_data = if let Some(exe_msgs) = execute_msgs {
        for msgs in exe_msgs {
            let execute_data = ExecuteData {
                order: msgs.order,
                contract: deps.api.addr_validate(&msgs.contract)?,
                msg: msgs.msg,
            };
            data_list.push(execute_data)
        }
        Some(data_list)
    } else {
        None
    };

    let staked_amount = query_total_voting_balance_at_timestamp(
        &deps.querier,
        &config.ve_token,
        Some(env.block.time.seconds()),
    )?;

    let sender_addr_raw = deps.api.addr_validate(&proposer)?;
    let new_poll = Poll {
        id: poll_id,
        creator: sender_addr_raw,
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        start_time: env.block.time.seconds(),
        end_height: env.block.height + config.voting_period,
        title,
        description,
        link,
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
        ("creator", &new_poll.creator.to_string().as_str()),
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

    let staked_amount = a_poll.staked_amount.unwrap();

    let (quorum, staked_weight) = if staked_amount.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else {
        (
            Decimal::from_ratio(tallied_weight, staked_amount),
            staked_amount,
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
                contract_addr: config.dao_token.to_string(),
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
 * Execute a msg of passed poll.
 */
pub fn execute_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if a_poll.end_height + config.timelock_period > env.block.height {
        return Err(ContractError::TimelockNotExpired {});
    }

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
            }))
        }
    } else {
        return Err(ContractError::NoExecuteData {});
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "execute_poll"),
        ("poll_id", &poll_id.to_string()),
    ]))
}

/// ExpirePoll is used to make the poll as expired state for querying purpose
pub fn expire_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if a_poll.execute_data.is_none() {
        return Err(ContractError::NoExecuteData {});
    }

    if a_poll.end_height + config.expiration_period > env.block.height {
        return Err(ContractError::PollNotExpired {});
    }

    poll_indexer_store(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_store(deps.storage, &PollStatus::Expired).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Expired;
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "expire_poll"),
        ("poll_id", &poll_id.to_string()),
    ]))
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
) -> Result<Response, ContractError> {
    let sender_address = deps.api.addr_validate(info.sender.as_str())?;
    let sender_addr_raw = sender_address.as_bytes();
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
        .load(sender_addr_raw)
        .is_ok()
    {
        return Err(ContractError::AlreadyVoted {});
    }

    let amount = query_address_voting_balance_at_timestamp(
        &deps.querier,
        &config.ve_token,
        Some(a_poll.start_time),
        &info.sender,
    )?;

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

    // store poll voter && and update poll data
    poll_voter_store(deps.storage, poll_id).save(sender_addr_raw, &vote_info)?;
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "cast_vote"),
        ("poll_id", &poll_id.to_string()),
        ("amount", &amount.to_string().as_str()),
        ("voter", info.sender.to_string().as_str()),
        ("vote_option", vote_info.vote.to_string().as_str()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_binary(&query_state(deps)?)?),
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
        dao_token: config.dao_token.to_string(),
        swap_factory: config.swap_factory.to_string(),
        quorum: config.quorum,
        threshold: config.threshold,
        voting_period: config.voting_period,
        timelock_period: config.timelock_period,
        expiration_period: config.expiration_period,
        proposal_deposit: config.proposal_deposit,
        snapshot_period: config.snapshot_period,
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
        start_time: poll.start_time,
        end_height: poll.end_height,
        title: poll.title,
        description: poll.description,
        link: poll.link,
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            for msg in exe_msgs {
                let execute_data = PollExecuteMsg {
                    order: msg.order,
                    contract: msg.contract.to_string(),
                    msg: msg.msg,
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
                start_time: poll.start_time,
                end_height: poll.end_height,
                title: poll.title.to_string(),
                description: poll.description.to_string(),
                link: poll.link.clone(),
                deposit_amount: poll.deposit_amount,
                execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
                    let mut data_list: Vec<PollExecuteMsg> = vec![];

                    for msg in exe_msgs {
                        let execute_data = PollExecuteMsg {
                            order: msg.order,
                            contract: msg.contract.to_string(),
                            msg: msg.msg,
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
            deps.storage,
            poll_id,
            Some(deps.api.addr_validate(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_poll_voters(deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: String::from_utf8(voter_info.0.clone())?,
                vote: voter_info.1.vote.clone(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(VotersResponse {
        voters: voters_response?,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
