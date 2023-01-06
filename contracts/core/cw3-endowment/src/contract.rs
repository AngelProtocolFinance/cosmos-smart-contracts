use crate::msg::{
    ConfigResponse, ExecuteMsg, MetaProposalListResponse, MetaProposalResponse, MigrateMsg,
};
use crate::state::{
    next_id, Ballot, Config, Proposal, TempConfig, Votes, BALLOTS, CONFIG, GUARDIAN_BALLOTS,
    GUARDIAN_PROPOSALS, PROPOSALS, TEMP_CONFIG,
};
use angel_core::errors::multisig::ContractError;
use angel_core::messages::cw3_multisig::{EndowmentInstantiateMsg as InstantiateMsg, QueryMsg};
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::utils::event_contains_attr;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, BlockInfo, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, QueryRequest, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    SubMsgResult, WasmMsg, WasmMsg::Execute, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw3::{
    Status, Vote, VoteInfo, VoteListResponse, VoteResponse, VoterDetail, VoterListResponse,
    VoterResponse,
};
use cw4::{Cw4Contract, Member, MemberChangedHookMsg, MemberDiff};
use cw_asset::AssetUnchecked;
use cw_storage_plus::Bound;
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};
use std::cmp::Ordering;

// version info for migration info
const CONTRACT_NAME: &str = "cw3-endowment";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_CW4_REPLY_ID: u64 = 0;
const APTEAM_CW3_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // store config in a temp config item until after CW4 phones home to complete the setup
    TEMP_CONFIG.save(
        deps.storage,
        &TempConfig {
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            threshold: msg.threshold,
            max_voting_period: msg.max_voting_period,
        },
    )?;

    Ok(Response::default()
        .add_attribute("endow_id", msg.id.to_string())
        .add_attribute("multisig_addr", env.contract.address.to_string())
        // Fire a submessage to create the CW4 Group to be linked to this CW3 on reply
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: msg.cw4_code,
                admin: None,
                label: format!("v2 endowment cw4 group - {}", msg.id),
                msg: to_binary(&angel_core::messages::cw4_group::InstantiateMsg {
                    admin: Some(env.contract.address.to_string()),
                    members: msg.cw4_members,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INIT_CW4_REPLY_ID => cw4_group_reply(deps, env, msg.result),
        APTEAM_CW3_REPLY_ID => apteam_cw3_reply(deps, env, msg.result),
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("Invalid Submessage Reply ID: {}", msg.id),
        })),
    }
}

pub fn apteam_cw3_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            // filter out relevent event from the reply logs
            let apteam_event = subcall
                .events
                .iter()
                .find(|event| event_contains_attr(event, "action", "locked_withdraw_proposal"))
                .ok_or_else(|| {
                    StdError::generic_err("cannot find `locked_withdraw_proposal` event")
                })?;

            // find & parse the proposal IDs from event
            let cw3_apteam_proposal: u64 = apteam_event
                .attributes
                .iter()
                .cloned()
                .find(|attr| attr.key == "proposal_id")
                .ok_or_else(|| {
                    StdError::generic_err(
                        "Cannot find `proposal_id` attribute within the `locked_withdraw_proposal` event",
                    )
                })?
                .value
                .parse()
                .unwrap();

            let orig_proposal_id: u64 = apteam_event
                .attributes
                .iter()
                .cloned()
                .find(|attr| attr.key == "endowment_cw3_proposal_id")
                .ok_or_else(|| {
                    StdError::generic_err(
                        "Cannot find `endowment_cw3_proposal_id` attribute within the `locked_withdraw_proposal` event",
                    )
                })?
                .value
                .parse()
                .unwrap();

            // update the original proposal with the new confirmation CW3 AP Team proposal ID
            let mut proposal = PROPOSALS.load(deps.storage, orig_proposal_id)?;
            proposal.confirmation_proposal = Some(cw3_apteam_proposal);
            PROPOSALS.save(deps.storage, orig_proposal_id, &proposal)?;

            Ok(Response::default())
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

/// This where the init logic is moved so that we can move forward only
/// with the the newly created CW4 contract's information
pub fn cw4_group_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut cw4_group_addr: Option<String> = None;
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        if attrb.key == "group_addr" {
                            cw4_group_addr = Some(attrb.value);
                        }
                    }
                }
            }

            // pull the original init msg values from the TempConfig item
            let temp = TEMP_CONFIG.load(deps.storage)?;

            // set up the CW3 config using the new CW4 group contract
            let group_addr = Cw4Contract(
                deps.api
                    .addr_validate(&cw4_group_addr.clone().unwrap())
                    .map_err(|_| ContractError::InvalidGroup {
                        addr: cw4_group_addr.unwrap(),
                    })?,
            );

            let total_weight = group_addr.total_weight(&deps.querier)?;
            temp.threshold.validate(total_weight)?;

            // validated and checked, we can not save the offical CW3 config
            CONFIG.save(
                deps.storage,
                &Config {
                    registrar_contract: temp.registrar_contract,
                    threshold: temp.threshold,
                    max_voting_period: temp.max_voting_period,
                    group_addr,
                    guardians: vec![],
                    require_execution: false, // default to auto-executing passing proposals
                },
            )?;

            Ok(Response::default())
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
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
            title,
            description,
            msgs,
            latest,
            meta,
        } => execute_propose(deps, env, info, title, description, msgs, latest, meta),
        ExecuteMsg::ProposeLockedWithdraw {
            endowment_id,
            description,
            beneficiary,
            assets,
            latest,
            meta,
        } => execute_propose_locked_withdraw(
            deps,
            env,
            info,
            endowment_id,
            description,
            beneficiary,
            assets,
            latest,
            meta,
        ),
        ExecuteMsg::Vote { proposal_id, vote } => execute_vote(deps, env, info, proposal_id, vote),
        ExecuteMsg::UpdateConfig {
            threshold,
            max_voting_period,
            require_execution,
            guardians,
        } => execute_update_config(
            deps,
            env,
            info,
            threshold,
            max_voting_period,
            require_execution,
            guardians,
        ),
        ExecuteMsg::Execute { proposal_id } => execute_execute(deps, env, info, proposal_id),
        ExecuteMsg::Close { proposal_id } => execute_close(deps, env, info, proposal_id),
        ExecuteMsg::MemberChangedHook(MemberChangedHookMsg { diffs }) => {
            execute_membership_hook(deps, env, info, diffs)
        }
        // GUARDIAN ENDPOINTS
        ExecuteMsg::GuardianPropose {
            title,
            description,
            old_member,
            new_member,
            latest,
            meta,
        } => execute_guardian_propose(
            deps,
            env,
            info,
            title,
            description,
            old_member,
            new_member,
            latest,
            meta,
        ),
        ExecuteMsg::GuardianVote { proposal_id, vote } => {
            execute_guardian_vote(deps, env, info, proposal_id, vote)
        }
        ExecuteMsg::GuardianExecute { proposal_id } => {
            execute_guardian_execute(deps, env, info, proposal_id)
        }
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    threshold: Threshold,
    max_voting_period: Duration,
    require_execution: Option<bool>,
    guardians: Option<Vec<String>>,
) -> Result<Response<Empty>, ContractError> {
    // only the contract can update own configs
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let mut cfg = CONFIG.load(deps.storage)?;
    cfg.threshold = threshold;
    cfg.max_voting_period = max_voting_period;

    if require_execution != None {
        cfg.require_execution = require_execution.unwrap();
    }

    if guardians != None {
        // check that all guardians passed are valid addresses
        let validated_guardians: Vec<Addr> = guardians
            .unwrap()
            .iter()
            .map(|a| deps.api.addr_validate(&a).unwrap())
            .collect();
        cfg.guardians = validated_guardians;
    }

    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::default())
}

pub fn execute_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    msgs: Vec<CosmosMsg>,
    latest: Option<Expiration>, // we ignore earliest
    meta: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Only members of the multisig can create a proposal
    // Non-voting members are special - they are allowed to create a proposal and
    // therefore "vote", but they aren't allowed to vote otherwise.
    // Such vote is also special, because despite having 0 weight it still counts when
    // counting threshold passing
    let vote_power = cfg
        .group_addr
        .is_member(&deps.querier, &info.sender, Some(env.block.height))?
        .ok_or(ContractError::Unauthorized {})?;

    // max expires also used as default
    let max_expires = cfg.max_voting_period.after(&env.block);
    let mut expires = latest.unwrap_or(max_expires);
    let comp = expires.partial_cmp(&max_expires);
    if let Some(Ordering::Greater) = comp {
        expires = max_expires;
    } else if comp.is_none() {
        return Err(ContractError::WrongExpiration {});
    }

    // create a proposal
    let id = next_id(deps.storage)?;
    let mut prop = Proposal {
        title,
        description,
        start_height: env.block.height,
        expires,
        msgs,
        confirmation_proposal: None,
        status: Status::Open,
        votes: Votes::new(vote_power),
        threshold: cfg.threshold,
        total_weight: cfg.group_addr.total_weight(&deps.querier)?,
        meta,
    };
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, id, &prop)?;

    // add the first yes vote from voter
    let ballot = Ballot {
        weight: vote_power,
        vote: Vote::Yes,
    };
    BALLOTS.save(deps.storage, (id, &info.sender), &ballot)?;

    // If Proposal's status is Passed, then execute it immediately (if execution is not explicitly required)
    let mut direct_execute_msgs = vec![];
    let auto_execute = !cfg.require_execution && prop.status == Status::Passed;
    if auto_execute {
        direct_execute_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Execute { proposal_id: id }).unwrap(),
            funds: vec![],
        }));
    };

    Ok(Response::new()
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("auto-executed", auto_execute.to_string())
        .add_messages(direct_execute_msgs))
}

pub fn execute_propose_locked_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: u32,
    description: String,
    beneficiary: String,
    assets: Vec<AssetUnchecked>,
    latest: Option<Expiration>, // we ignore earliest
    meta: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Only members of the multisig can create a proposal
    // Non-voting members are special - they are allowed to create a proposal and
    // therefore "vote", but they aren't allowed to vote otherwise.
    // Such vote is also special, because despite having 0 weight it still counts when
    // counting threshold passing
    let vote_power = cfg
        .group_addr
        .is_member(&deps.querier, &info.sender, Some(env.block.height))?
        .ok_or(ContractError::Unauthorized {})?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cfg.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // max expires also used as default
    let max_expires = cfg.max_voting_period.after(&env.block);
    let mut expires = latest.unwrap_or(max_expires);
    let comp = expires.partial_cmp(&max_expires);
    if let Some(Ordering::Greater) = comp {
        expires = max_expires;
    } else if comp.is_none() {
        return Err(ContractError::WrongExpiration {});
    }

    // create a proposal
    let id = next_id(deps.storage)?;
    let mut prop = Proposal {
        title: format!("Locked Withdraw Request - Endowment #{}", endowment_id),
        description: format!("Reason for request:\n{}", description),
        start_height: env.block.height,
        expires,
        msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registrar_config.owner,
            msg: to_binary(
                &angel_core::messages::cw3_apteam::ExecuteMsg::ProposeLockedWithdraw {
                    orig_proposal: id,
                    endowment_id,
                    description,
                    beneficiary,
                    assets,
                    latest,
                    meta: meta.clone(),
                },
            )
            .unwrap(),
            funds: vec![],
        })],
        confirmation_proposal: None,
        status: Status::Open,
        votes: Votes::new(vote_power),
        threshold: cfg.threshold,
        total_weight: cfg.group_addr.total_weight(&deps.querier)?,
        meta,
    };
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, id, &prop)?;

    // add the first yes vote from voter
    let ballot = Ballot {
        weight: vote_power,
        vote: Vote::Yes,
    };
    BALLOTS.save(deps.storage, (id, &info.sender), &ballot)?;

    // If Proposal's status is Passed, then execute it immediately (if execution is not explicitly required)
    let mut direct_execute_msgs = vec![];
    let auto_execute = !cfg.require_execution && prop.status == Status::Passed;
    if auto_execute {
        direct_execute_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Execute { proposal_id: id }).unwrap(),
            funds: vec![],
        }));
    };

    Ok(Response::new()
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("auto-executed", auto_execute.to_string())
        .add_messages(direct_execute_msgs))
}

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: Vote,
) -> Result<Response<Empty>, ContractError> {
    // only members of the multisig can vote
    let cfg = CONFIG.load(deps.storage)?;

    // ensure proposal exists and can be voted on
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    if prop.status != Status::Open {
        return Err(ContractError::NotOpen {});
    }
    if prop.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // Only voting members of the multisig can vote
    // Additional check if weight >= 1
    // use a snapshot of "start of proposal"
    let vote_power = cfg
        .group_addr
        .member_at_height(&deps.querier, info.sender.clone(), Some(prop.start_height))?
        .ok_or(ContractError::Unauthorized {})?;

    // cast vote if no vote previously cast
    BALLOTS.update(deps.storage, (proposal_id, &info.sender), |bal| match bal {
        Some(_) => Err(ContractError::AlreadyVoted {}),
        None => Ok(Ballot {
            weight: vote_power,
            vote,
        }),
    })?;

    // update vote tally
    prop.votes.add_vote(vote, vote_power);
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;

    // If Proposal's status is Passed, then execute it immediately (if execution is not explicitly required)
    let mut direct_execute_msgs = vec![];
    let auto_execute = !cfg.require_execution && prop.status == Status::Passed;
    if auto_execute {
        direct_execute_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Execute { proposal_id }).unwrap(),
            funds: vec![],
        }));
    }

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("auto-executed", auto_execute.to_string())
        .add_messages(direct_execute_msgs))
}

pub fn execute_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    // anyone can trigger this if the vote passed
    // try to look up the proposal from the ID given
    let mut prop: Proposal = match PROPOSALS.load(deps.storage, proposal_id) {
        Ok(p) => p,
        _ => return Err(ContractError::Unauthorized {}),
    };

    // we allow execution even after the proposal "expiration" as long as all vote come in before
    // that point. If it was approved on time, it can be executed any time.
    if prop.status != Status::Passed {
        return Err(ContractError::WrongExecuteStatus {});
    }

    // set it to executed
    prop.status = Status::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;

    // grab the ap_team cw3 address from registrar
    let cfg = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cfg.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // work into submsgs where needed (ie. going to AP Team CW3 or Gov) for catching replies
    let mut res = Response::new()
        .add_attribute("action", "execute")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string());

    // check for the single msg early withdraw proposal to
    match &prop.msgs[0] {
        cosmwasm_std::CosmosMsg::Wasm(Execute { contract_addr, .. }) => {
            res = if contract_addr.to_string() == registrar_config.owner {
                res.add_submessage(SubMsg::reply_on_success(
                    prop.msgs[0].clone(),
                    APTEAM_CW3_REPLY_ID,
                ))
            } else {
                res.add_messages(prop.msgs)
            };
        }
        _ => res = res.add_messages(prop.msgs),
    }
    // dispatch all proposed messages
    Ok(res)
}

pub fn execute_close(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response<Empty>, ContractError> {
    // anyone can trigger this if the vote passed
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
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
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;

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
    if info.sender != cfg.group_addr.0 {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::default())
}

// GUARDIAN RELATED FUNCTIONS
pub fn execute_guardian_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    old_member: String,
    new_member: String,
    latest: Option<Expiration>, // we ignore earliest
    meta: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Only guardian addresses can create a proposal
    if cfg.guardians.iter().position(|g| g == &info.sender) == None {
        return Err(ContractError::Unauthorized {});
    }

    // New & Old Members must be valid address
    let old_addr = deps.api.addr_validate(&old_member)?;
    let new_addr = deps.api.addr_validate(&new_member)?;

    // Old Member must be currently in the CW4 Group
    cfg.group_addr
        .member_at_height(&deps.querier, old_addr.clone(), Some(env.block.height))?
        .ok_or(ContractError::Unauthorized {})?;

    // max expires also used as default
    let max_expires = cfg.max_voting_period.after(&env.block);
    let mut expires = latest.unwrap_or(max_expires);
    let comp = expires.partial_cmp(&max_expires);
    if let Some(Ordering::Greater) = comp {
        expires = max_expires;
    } else if comp.is_none() {
        return Err(ContractError::WrongExpiration {});
    }

    // create a proposal
    let id = next_id(deps.storage)?;
    let mut prop = Proposal {
        title,
        description,
        start_height: env.block.height,
        expires,
        msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cfg.group_addr.0.to_string(),
            msg: to_binary(
                &angel_core::messages::cw4_group::ExecuteMsg::UpdateMembers {
                    remove: vec![old_member],
                    add: vec![Member {
                        addr: new_addr.to_string(),
                        weight: 1,
                    }],
                },
            )
            .unwrap(),
            funds: vec![],
        })],
        confirmation_proposal: None,
        status: Status::Open,
        votes: Votes::new(1_u64),
        threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(75),
        },
        total_weight: cfg.guardians.len() as u64, // 1 guardian, 1 vote
        meta,
    };
    prop.update_status(&env.block);
    GUARDIAN_PROPOSALS.save(deps.storage, id, &prop)?;

    // add the first yes vote from voter
    GUARDIAN_BALLOTS.save(
        deps.storage,
        (id, &info.sender),
        &Ballot {
            weight: 1_u64,
            vote: Vote::Yes,
        },
    )?;

    // If Proposal's status is Passed, then execute it immediately (if execution is not explicitly required)
    let mut direct_execute_msgs = vec![];
    let auto_execute = !cfg.require_execution && prop.status == Status::Passed;
    if auto_execute {
        direct_execute_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::GuardianExecute { proposal_id: id }).unwrap(),
            funds: vec![],
        }));
    };

    Ok(Response::new()
        .add_attribute("action", "guardian_propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("auto-executed", auto_execute.to_string())
        .add_messages(direct_execute_msgs))
}

pub fn execute_guardian_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: Vote,
) -> Result<Response<Empty>, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // only guardians of the multisig can vote
    if cfg.guardians.iter().position(|g| g == &info.sender) == None {
        return Err(ContractError::Unauthorized {});
    }

    // ensure proposal exists and can be voted on
    let mut prop = GUARDIAN_PROPOSALS.load(deps.storage, proposal_id)?;
    if prop.status != Status::Open {
        return Err(ContractError::NotOpen {});
    }
    if prop.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // cast vote if no vote previously cast
    GUARDIAN_BALLOTS.update(deps.storage, (proposal_id, &info.sender), |bal| match bal {
        Some(_) => Err(ContractError::AlreadyVoted {}),
        None => Ok(Ballot {
            weight: 1_u64,
            vote,
        }),
    })?;

    // update vote tally
    prop.votes.add_vote(vote, 1_u64);
    prop.update_status(&env.block);
    GUARDIAN_PROPOSALS.save(deps.storage, proposal_id, &prop)?;

    // If Proposal's status is Passed, then execute it immediately (if execution is not explicitly required)
    let mut direct_execute_msgs = vec![];
    let auto_execute = !cfg.require_execution && prop.status == Status::Passed;
    if auto_execute {
        direct_execute_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::GuardianExecute { proposal_id }).unwrap(),
            funds: vec![],
        }));
    }

    Ok(Response::new()
        .add_attribute("action", "guardian_vote")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("auto-executed", auto_execute.to_string())
        .add_messages(direct_execute_msgs))
}

pub fn execute_guardian_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    // anyone can trigger this if the vote passed
    // try to look up the proposal from the ID given
    let mut prop: Proposal = match GUARDIAN_PROPOSALS.load(deps.storage, proposal_id) {
        Ok(p) => p,
        _ => return Err(ContractError::Unauthorized {}),
    };

    // we allow execution even after the proposal "expiration" as long as all vote come in before
    // that point. If it was approved on time, it can be executed any time.
    if prop.status != Status::Passed {
        return Err(ContractError::WrongExecuteStatus {});
    }

    // set it to executed
    prop.status = Status::Executed;
    GUARDIAN_PROPOSALS.save(deps.storage, proposal_id, &prop)?;

    // dispatch all proposed messages
    Ok(Response::new()
        .add_attribute("action", "guardian_execute")
        .add_attribute("sender", info.sender)
        .add_messages(prop.msgs)
        .add_attribute("proposal_id", proposal_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
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

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        require_execution: cfg.require_execution,
        threshold: cfg.threshold,
        max_voting_period: cfg.max_voting_period,
        group_addr: cfg.group_addr.0.to_string(),
        registrar_contract: cfg.registrar_contract.to_string(),
    })
}

fn query_threshold(deps: Deps) -> StdResult<ThresholdResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let total_weight = cfg.group_addr.total_weight(&deps.querier)?;
    Ok(cfg.threshold.to_response(total_weight))
}

fn query_proposal(deps: Deps, env: Env, id: u64) -> StdResult<MetaProposalResponse> {
    let prop = PROPOSALS.load(deps.storage, id)?;
    let status = prop.current_status(&env.block);
    let threshold = prop.threshold.to_response(prop.total_weight);
    Ok(MetaProposalResponse {
        id,
        title: prop.title,
        description: prop.description,
        msgs: prop.msgs,
        status,
        confirmation_proposal: prop.confirmation_proposal,
        expires: prop.expires,
        threshold,
        meta: prop.meta,
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
) -> StdResult<MetaProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let proposals = PROPOSALS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect::<StdResult<_>>()?;

    Ok(MetaProposalListResponse { proposals })
}

fn reverse_proposals(
    deps: Deps,
    env: Env,
    start_before: Option<u64>,
    limit: Option<u32>,
) -> StdResult<MetaProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let end = start_before.map(Bound::exclusive);
    let props: StdResult<Vec<_>> = PROPOSALS
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();

    Ok(MetaProposalListResponse { proposals: props? })
}

fn map_proposal(
    block: &BlockInfo,
    item: StdResult<(u64, Proposal)>,
) -> StdResult<MetaProposalResponse> {
    item.map(|(id, prop)| {
        let status = prop.current_status(block);
        let threshold = prop.threshold.to_response(prop.total_weight);
        MetaProposalResponse {
            id,
            title: prop.title,
            description: prop.description,
            msgs: prop.msgs,
            status,
            confirmation_proposal: prop.confirmation_proposal,
            expires: prop.expires,
            threshold,
            meta: prop.meta,
        }
    })
}

fn query_vote(deps: Deps, proposal_id: u64, voter: String) -> StdResult<VoteResponse> {
    let voter_addr = deps.api.addr_validate(&voter)?;
    let prop = BALLOTS.may_load(deps.storage, (proposal_id, &voter_addr))?;
    let vote = prop.map(|b| VoteInfo {
        voter,
        proposal_id,
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
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let votes = BALLOTS
        .prefix(proposal_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, ballot)| VoteInfo {
                proposal_id,
                voter: addr.into(),
                vote: ballot.vote,
                weight: ballot.weight,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(VoteListResponse { votes })
}

fn query_voter(deps: Deps, voter: String) -> StdResult<VoterResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let voter_addr = deps.api.addr_validate(&voter)?;
    let weight = cfg.group_addr.is_member(&deps.querier, &voter_addr, None)?;

    Ok(VoterResponse { weight })
}

fn list_voters(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VoterListResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let voters = cfg
        .group_addr
        .list_members(&deps.querier, start_after, limit)?
        .into_iter()
        .map(|member| VoterDetail {
            addr: member.addr,
            weight: member.weight,
        })
        .collect();
    Ok(VoterListResponse { voters })
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        }));
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        }));
    }
    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
