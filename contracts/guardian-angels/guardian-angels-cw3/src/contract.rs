use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Threshold, UpdateConfigMsg};
use crate::state::{
    next_id, parse_id, Ballot, Config, Proposal, Votes, BALLOTS, CONFIG, GUARDIAN_PROPOSALS,
    PROPOSALS,
};
use angel_core::messages::accounts::QueryMsg as EndowmentQueryMsg;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::responses::accounts::EndowmentDetailsResponse;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, BlockInfo, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo,
    Order, QueryRequest, Response, StdResult, WasmMsg, WasmQuery,
};
use cw0::{maybe_addr, Expiration};
use cw2::set_contract_version;
use cw3::{
    ProposalListResponse, ProposalResponse, Status, ThresholdResponse, Vote, VoteInfo,
    VoteListResponse, VoteResponse, VoterDetail, VoterListResponse, VoterResponse,
};
use cw4::{Cw4Contract, MemberChangedHookMsg, MemberDiff};
use cw_storage_plus::Bound;
use std::cmp::Ordering;

// version info for migration info
const CONTRACT_NAME: &str = "guardian-angels-multisig";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let ap_team_group = Cw4Contract(deps.api.addr_validate(&msg.ap_team_group).map_err(|_| {
        ContractError::InvalidGroup {
            addr: msg.ap_team_group.clone(),
        }
    })?);
    let total_weight = ap_team_group.total_weight(&deps.querier)?;
    msg.threshold.validate(total_weight)?;

    let endowment_owners_group = Cw4Contract(
        deps.api
            .addr_validate(&msg.endowment_owners_group)
            .map_err(|_| ContractError::InvalidGroup {
                addr: msg.endowment_owners_group.clone(),
            })?,
    );
    let registrar_contract = deps.api.addr_validate(&msg.registrar_contract)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        threshold: msg.threshold,
        max_voting_period: msg.max_voting_period,
        max_voting_period_guardians: msg.max_voting_period_guardians,
        ap_team_group,
        endowment_owners_group,
        registrar_contract,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<Empty>, ContractError> {
    match msg {
        ExecuteMsg::Propose {
            endowment_addr,
            title,
            description,
            msgs,
            latest,
        } => execute_propose(
            deps,
            env,
            info,
            title,
            description,
            msgs,
            latest,
            endowment_addr,
            None,
        ),
        ExecuteMsg::ProposeOwnerChange {
            endowment_addr,
            new_owner_addr,
        } => execute_propose_owner_change(deps, env, info, endowment_addr, new_owner_addr),
        ExecuteMsg::ProposeGuardianChange {
            endowment_addr,
            add,
            remove,
        } => execute_propose_guardian_change(deps, env, info, endowment_addr, add, remove),
        ExecuteMsg::Vote { proposal_id, vote } => execute_vote(deps, env, info, proposal_id, vote),
        ExecuteMsg::VoteGuardian { proposal_id, vote } => {
            execute_vote_guardian(deps, env, info, proposal_id, vote)
        }
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, env, info, msg),
        ExecuteMsg::Execute { proposal_id } => execute_execute(deps, env, info, proposal_id),
        ExecuteMsg::Close { proposal_id } => execute_close(deps, env, info, proposal_id),
        ExecuteMsg::MemberChangedHook(MemberChangedHookMsg { diffs }) => {
            execute_membership_hook(deps, env, info, diffs)
        }
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response<Empty>, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;

    // get the owner of the Registrar config
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cfg.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    // only the owner/admin of the Registrar contract can update the configs
    if info.sender != registrar_config.owner {
        return Err(ContractError::Unauthorized {});
    }

    cfg.threshold = msg.threshold;
    cfg.max_voting_period = msg.max_voting_period;
    cfg.max_voting_period_guardians = msg.max_voting_period_guardians;

    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::default())
}

pub fn execute_propose_owner_change(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_addr: String,
    new_owner_addr: String,
) -> Result<Response<Empty>, ContractError> {
    execute_propose(
        deps,
        env,
        info,
        "Change Endowment Owner".to_string(),
        format!(
            "Changes Endowment Owner\n- endowment: {}\n- new owner: {}",
            endowment_addr, new_owner_addr
        ),
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: endowment_addr.clone(),
            msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::UpdateOwner {
                new_owner: new_owner_addr,
            })
            .unwrap(),
            funds: vec![],
        })],
        None,
        endowment_addr,
        Some("owner_change".to_string()),
    )
}
pub fn execute_propose_guardian_change(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_addr: String,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<Response<Empty>, ContractError> {
    execute_propose(
        deps,
        env,
        info,
        "Change Endowment Guardians".to_string(),
        format!(
            "Changes Endowment Guardians set\n- endowment: {}\n- add: {:?}\n- remove: {:?}",
            endowment_addr, add, remove
        ),
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: endowment_addr.clone(),
            msg: to_binary(
                &angel_core::messages::accounts::ExecuteMsg::UpdateGuardians { add, remove },
            )
            .unwrap(),
            funds: vec![],
        })],
        None,
        endowment_addr,
        Some("guardian_change".to_string()),
    )
}

pub fn execute_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    msgs: Vec<CosmosMsg>,
    latest: Option<Expiration>, // we ignore earliest
    endowment_addr: String,
    special_proposal: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    // look up guardian set & current owner for the Endowment
    let endowment_info: EndowmentDetailsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: endowment_addr.clone(),
            msg: to_binary(&EndowmentQueryMsg::Endowment {})?,
        }))?;
    let guardians_count = endowment_info.guardians.len() as u64;
    let cfg = CONFIG.load(deps.storage)?;
    let vote_power;
    let total_weight;
    let threshold;
    let max_expires;

    if special_proposal == None {
        // Only Endowment Owner members can create generic proposals (non-special proposals).
        // General proposals cannot be created by Guardians or by AP Team Members.
        // AP Team Group members vote to appove these generic proposals by Endowment Owners.
        vote_power = cfg
            .endowment_owners_group
            .is_member(&deps.querier, &info.sender)?
            .ok_or(ContractError::Unauthorized {})?;
        total_weight = cfg.ap_team_group.total_weight(&deps.querier)?;
        threshold = cfg.threshold;
        max_expires = cfg.max_voting_period.after(&env.block);
    } else {
        max_expires = cfg.max_voting_period_guardians.after(&env.block);
        let proposal_type = special_proposal.as_ref().unwrap();
        if proposal_type == "guardian_change" && info.sender == endowment_info.owner {
            vote_power = cfg
                .endowment_owners_group
                .is_member(&deps.querier, &info.sender)?
                .ok_or(ContractError::Unauthorized {})?;

            total_weight = guardians_count + 1;
            if guardians_count > 0 {
                threshold = Threshold::ThresholdQuorum {
                    // threshold is (N/2+1) where N is the current # of Guardians
                    threshold: Decimal::from_ratio(guardians_count / 2 + 1, guardians_count),
                    // quorum is % representation of 1 vote (ie. the owner's first vote)
                    quorum: Decimal::from_ratio(1u128, guardians_count + 1),
                };
            } else {
                threshold = Threshold::AbsoluteCount { weight: 1 };
                threshold.validate(total_weight)?;
            }
            threshold.validate(total_weight)?;
        } else if proposal_type == "owner_change"
            && endowment_info.is_guardian(info.sender.to_string())
        {
            // guardians get a default voting weight of 1
            vote_power = 1;
            total_weight = guardians_count;
            // threshold is (N/2+1) where N is the current # of Guardians
            threshold = Threshold::AbsoluteCount {
                weight: guardians_count / 2 + 1,
            };
            threshold.validate(total_weight)?;
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }
    let mut expires = latest.unwrap_or(max_expires);
    let comp = expires.partial_cmp(&max_expires);
    if let Some(Ordering::Greater) = comp {
        expires = max_expires;
    } else if comp.is_none() {
        return Err(ContractError::WrongExpiration {});
    }

    // create a proposal
    let mut prop = Proposal {
        endowment_addr,
        title,
        description,
        start_height: env.block.height,
        expires,
        msgs,
        status: Status::Open,
        votes: Votes::new(vote_power),
        threshold,
        total_weight,
    };
    prop.update_status(&env.block);
    let id = next_id(deps.storage)?;

    match special_proposal {
        // add special proposals to focused lists for easier handling/recall later
        Some(_p) => GUARDIAN_PROPOSALS.save(deps.storage, id.into(), &prop)?,
        // all other proposals added to general list
        None => PROPOSALS.save(deps.storage, id.into(), &prop)?,
    }

    // add the first yes vote from voter
    let ballot = Ballot {
        weight: vote_power,
        vote: Vote::Yes,
    };
    BALLOTS.save(deps.storage, (id.into(), &info.sender), &ballot)?;

    Ok(Response::new()
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("status", format!("{:?}", prop.status)))
}

pub fn execute_vote_guardian(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: Vote,
) -> Result<Response<Empty>, ContractError> {
    // only Guardian members of the Endowment can vote on these proposals
    // ensure proposal exists and can be voted on
    let mut prop = GUARDIAN_PROPOSALS.load(deps.storage, proposal_id.into())?;
    if prop.status != Status::Open {
        return Err(ContractError::NotOpen {});
    }
    if prop.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // only Guardian members on an Endowment of the multisig can vote
    let endowment_info: EndowmentDetailsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: prop.endowment_addr.clone(),
            msg: to_binary(&EndowmentQueryMsg::Endowment {})?,
        }))?;
    if !endowment_info.is_guardian(info.sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }

    let vote_power = 1;
    // cast vote if no vote previously cast
    BALLOTS.update(
        deps.storage,
        (proposal_id.into(), &info.sender),
        |bal| match bal {
            Some(_) => Err(ContractError::AlreadyVoted {}),
            None => Ok(Ballot {
                weight: vote_power,
                vote,
            }),
        },
    )?;

    // update vote tally
    prop.votes.add_vote(vote, vote_power);
    prop.update_status(&env.block);
    GUARDIAN_PROPOSALS.save(deps.storage, proposal_id.into(), &prop)?;

    Ok(Response::new()
        .add_attribute("action", "vote_guardian")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", format!("{:?}", prop.status)))
}

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: Vote,
) -> Result<Response<Empty>, ContractError> {
    // ensure proposal exists and can be voted on
    let mut prop = PROPOSALS.load(deps.storage, proposal_id.into())?;
    if prop.status != Status::Open {
        return Err(ContractError::NotOpen {});
    }
    if prop.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    let cfg = CONFIG.load(deps.storage)?;
    // use a snapshot of "start of proposal"
    let vote_power = cfg
        .ap_team_group
        .member_at_height(&deps.querier, info.sender.clone(), prop.start_height)?
        .ok_or(ContractError::Unauthorized {})?;

    // cast vote if no vote previously cast
    BALLOTS.update(
        deps.storage,
        (proposal_id.into(), &info.sender),
        |bal| match bal {
            Some(_) => Err(ContractError::AlreadyVoted {}),
            None => Ok(Ballot {
                weight: vote_power,
                vote,
            }),
        },
    )?;

    // update vote tally
    prop.votes.add_vote(vote, vote_power);
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, proposal_id.into(), &prop)?;

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", format!("{:?}", prop.status)))
}

pub fn execute_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    // anyone can trigger this if the vote passed
    // try to look up the proposal from the ID given
    let mut prop: Proposal;
    let mut proposal_store = PROPOSALS;
    match PROPOSALS.load(deps.storage, proposal_id.into()) {
        Ok(p) => {
            prop = p;
        }
        _ => match GUARDIAN_PROPOSALS.load(deps.storage, proposal_id.into()) {
            Ok(p) => {
                proposal_store = GUARDIAN_PROPOSALS;
                prop = p;
            }
            _ => return Err(ContractError::Unauthorized {}),
        },
    };

    // we allow execution even after the proposal "expiration" as long as all vote come in before
    // that point. If it was approved on time, it can be executed any time.
    if prop.status != Status::Passed {
        return Err(ContractError::WrongExecuteStatus {});
    }

    // set it to executed
    prop.status = Status::Executed;
    proposal_store.save(deps.storage, proposal_id.into(), &prop)?;

    // dispatch all proposed messages
    Ok(Response::new()
        .add_messages(prop.msgs)
        .add_attribute("action", "execute")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string()))
}

pub fn execute_close(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response<Empty>, ContractError> {
    // try to look up the proposal from the ID given
    let mut prop: Proposal;
    let mut proposal_store = PROPOSALS;
    match PROPOSALS.load(deps.storage, proposal_id.into()) {
        Ok(p) => {
            prop = p;
        }
        _ => match GUARDIAN_PROPOSALS.load(deps.storage, proposal_id.into()) {
            Ok(p) => {
                proposal_store = GUARDIAN_PROPOSALS;
                prop = p;
            }
            _ => return Err(ContractError::Unauthorized {}),
        },
    };

    if [Status::Executed, Status::Rejected, Status::Passed]
        .iter()
        .any(|x| *x == prop.status)
    {
        return Err(ContractError::WrongCloseStatus {});
    }
    if !prop.expires.is_expired(&env.block) {
        return Err(ContractError::NotExpired {});
    }

    // set it to failed
    prop.status = Status::Rejected;
    proposal_store.save(deps.storage, proposal_id.into(), &prop)?;

    Ok(Response::new()
        .add_attribute("action", "close")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string()))
}

pub fn execute_membership_hook(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _diffs: Vec<MemberDiff>,
) -> Result<Response<Empty>, ContractError> {
    // This is now a no-op
    // But we leave the authorization check as a demo
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.ap_team_group.0 {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Threshold {} => to_binary(&query_threshold(deps)?),
        QueryMsg::Proposal { proposal_id } => to_binary(&query_proposal(deps, env, proposal_id)?),
        QueryMsg::Vote { proposal_id, voter } => to_binary(&query_vote(deps, proposal_id, voter)?),
        QueryMsg::ListProposals { start_after, limit } => {
            to_binary(&list_proposals(deps, env, start_after, limit)?)
        }
        QueryMsg::ReverseProposals {
            start_before,
            limit,
        } => to_binary(&reverse_proposals(deps, env, start_before, limit)?),
        QueryMsg::ListVotes {
            proposal_id,
            start_after,
            limit,
        } => to_binary(&list_votes(deps, proposal_id, start_after, limit)?),
        QueryMsg::Voter { address } => to_binary(&query_voter(deps, address)?),
        QueryMsg::ListVoters { start_after, limit } => {
            to_binary(&list_voters(deps, start_after, limit)?)
        }
    }
}

fn query_threshold(deps: Deps) -> StdResult<ThresholdResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let total_weight = cfg.ap_team_group.total_weight(&deps.querier)?;
    Ok(cfg.threshold.to_response(total_weight))
}

fn query_proposal(deps: Deps, env: Env, id: u64) -> StdResult<ProposalResponse> {
    // try to look up the proposal from the ID given
    let prop = PROPOSALS
        .load(deps.storage, id.into())
        .unwrap_or(GUARDIAN_PROPOSALS.load(deps.storage, id.into())?);
    let status = prop.current_status(&env.block);
    let threshold = prop.threshold.to_response(prop.total_weight);
    Ok(ProposalResponse {
        id,
        title: prop.title,
        description: prop.description,
        msgs: prop.msgs,
        status,
        expires: prop.expires,
        threshold,
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn list_proposals(
    deps: Deps,
    env: Env,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive_int);
    let props: StdResult<Vec<_>> = PROPOSALS
        .range(deps.storage, start.clone(), None, Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();
    let special_props: StdResult<Vec<_>> = GUARDIAN_PROPOSALS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();
    let mut all_props = props.unwrap();
    all_props.extend(special_props.unwrap());
    Ok(ProposalListResponse {
        proposals: all_props,
    })
}

fn reverse_proposals(
    deps: Deps,
    env: Env,
    start_before: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let end = start_before.map(Bound::exclusive_int);
    let props: StdResult<Vec<_>> = PROPOSALS
        .range(deps.storage, None, end.clone(), Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();
    let special_props: StdResult<Vec<_>> = GUARDIAN_PROPOSALS
        .range(deps.storage, None, end, Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();
    let mut all_props = props.unwrap();
    all_props.extend(special_props.unwrap());

    Ok(ProposalListResponse {
        proposals: all_props,
    })
}

fn map_proposal(
    block: &BlockInfo,
    item: StdResult<(Vec<u8>, Proposal)>,
) -> StdResult<ProposalResponse> {
    let (key, prop) = item?;
    let status = prop.current_status(block);
    let threshold = prop.threshold.to_response(prop.total_weight);
    Ok(ProposalResponse {
        id: parse_id(&key)?,
        title: prop.title,
        description: prop.description,
        msgs: prop.msgs,
        status,
        expires: prop.expires,
        threshold,
    })
}

fn query_vote(deps: Deps, proposal_id: u64, voter: String) -> StdResult<VoteResponse> {
    let voter_addr = deps.api.addr_validate(&voter)?;
    let prop = BALLOTS.may_load(deps.storage, (proposal_id.into(), &voter_addr))?;
    let vote = prop.map(|b| VoteInfo {
        voter,
        vote: b.vote,
        weight: b.weight,
    });
    Ok(VoteResponse { vote })
}

fn list_votes(
    deps: Deps,
    proposal_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VoteListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let votes: StdResult<Vec<_>> = BALLOTS
        .prefix(proposal_id.into())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (voter, ballot) = item?;
            Ok(VoteInfo {
                voter: String::from_utf8(voter)?,
                vote: ballot.vote,
                weight: ballot.weight,
            })
        })
        .collect();

    Ok(VoteListResponse { votes: votes? })
}

fn query_voter(deps: Deps, voter: String) -> StdResult<VoterResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let voter_addr = deps.api.addr_validate(&voter)?;
    let weight = cfg.ap_team_group.is_member(&deps.querier, &voter_addr)?;

    Ok(VoterResponse { weight })
}

fn list_voters(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VoterListResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let voters = cfg
        .ap_team_group
        .list_members(&deps.querier, start_after, limit)?
        .into_iter()
        .map(|member| VoterDetail {
            addr: member.addr,
            weight: member.weight,
        })
        .collect();
    Ok(VoterListResponse { voters })
}
