use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MetaApplicationsProposalListResponse,
    MetaApplicationsProposalResponse, MigrateMsg,
};
use crate::state::{
    next_id, Ballot, Config, OldConfig, Proposal, ProposalType, Votes, BALLOTS, CONFIG, PROPOSALS,
};
use angel_core::errors::multisig::ContractError;
use angel_core::messages::accounts::QueryMsg::Endowment as EndowmentDetails;
use angel_core::messages::accounts::{
    CreateEndowmentMsg, DepositMsg, ExecuteMsg as AccountsExecuteMsg,
};
use angel_core::messages::cw3_multisig::QueryMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::accounts::EndowmentDetailsResponse;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::EndowmentType;
use angel_core::utils::validate_deposit_fund;
use cosmwasm_std::{
    entry_point, from_slice, to_binary, BankMsg, Binary, BlockInfo, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Empty, Env, MessageInfo, Order, QueryRequest, Reply, Response, StdError, StdResult,
    SubMsg, SubMsgResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw3::{
    Cw3QueryMsg, Status, Vote, VoteInfo, VoteListResponse, VoteResponse, VoterDetail,
    VoterListResponse, VoterResponse,
};
use cw4::{Cw4Contract, MemberChangedHookMsg, MemberDiff};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use cw_storage_plus::Bound;
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};
use std::cmp::Ordering;

// version info for migration info
const CONTRACT_NAME: &str = "cw3-applications";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const APPLICATION_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let group_addr = Cw4Contract(deps.api.addr_validate(&msg.group_addr).map_err(|_| {
        ContractError::InvalidGroup {
            addr: msg.group_addr.clone(),
        }
    })?);
    let total_weight = group_addr.total_weight(&deps.querier)?;
    msg.threshold.validate(total_weight)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        threshold: msg.threshold,
        max_voting_period: msg.max_voting_period,
        group_addr,
        require_execution: false,
        seed_asset: None,
        seed_split_to_liquid: Decimal::percent(0), // all to LOCKED
        new_endow_gas_money: None,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        APPLICATION_REPLY_ID => application_reply(deps, env, msg.result),
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("Invalid Submessage Reply ID: {}", msg.id),
        })),
    }
}

pub fn application_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            // filter out the newly created endowment ID from the events reply logs
            let mut endow_id: u32 = 0;
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        // This value comes from the custom attrbiute
                        match attrb.key.as_str() {
                            "endow_id" => endow_id = attrb.value.parse().unwrap(),
                            _ => (),
                        }
                    }
                }
            }
            if endow_id == 0 {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "Cannot find Application's new Endowment ID in event logs".to_string(),
                }));
            }

            let cfg = CONFIG.load(deps.storage)?;
            let mut res = Response::default();

            // query registrar config in order to get the accounts contract
            let registrar_config: RegistrarConfigResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: cfg.registrar_contract.to_string(),
                    msg: to_binary(&RegistrarConfig {})?,
                }))?;
            let accounts_contract = registrar_config.accounts_contract.unwrap();

            // Dust the Endowment CW3's first member wallet with some amount of native token to cover first few TXs gas
            match cfg.new_endow_gas_money {
                Some(gas_money) => {
                    if !gas_money.amount.is_zero() {
                        let endow: EndowmentDetailsResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: accounts_contract.to_string(),
                                msg: to_binary(&EndowmentDetails { id: endow_id })?,
                            }))?;
                        let voters: VoterListResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: endow.owner.to_string(),
                                msg: to_binary(&Cw3QueryMsg::ListVoters {
                                    limit: None,
                                    start_after: None,
                                })?,
                            }))?;
                        if !voters.voters.is_empty() {
                            res = res.add_message(CosmosMsg::Bank(BankMsg::Send {
                                to_address: voters.voters[0].addr.clone(),
                                amount: vec![gas_money],
                            }));
                        }
                    }
                }
                None => (),
            }

            // Add the seed money deposit to new endowment
            match cfg.seed_asset {
                Some(asset) => {
                    // build the responce with correct message based on Asset type
                    res = res.add_message(match &asset.info {
                        // execute of deposit w/ funds attached
                        AssetInfoBase::Native(ref denom) => CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: accounts_contract.to_string(),
                            msg: to_binary(&AccountsExecuteMsg::Deposit(DepositMsg {
                                id: endow_id,
                                locked_percentage: Decimal::one() - cfg.seed_split_to_liquid,
                                liquid_percentage: cfg.seed_split_to_liquid,
                            }))
                            .unwrap(),
                            funds: vec![Coin {
                                denom: denom.clone(),
                                amount: asset.amount,
                            }],
                        }),
                        // CW20 Send with deposit msg embedded
                        AssetInfo::Cw20(ref contract_addr) => CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: contract_addr.to_string(),
                            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                                contract: accounts_contract.to_string(),
                                amount: asset.amount,
                                msg: to_binary(&AccountsExecuteMsg::Deposit(DepositMsg {
                                    id: endow_id,
                                    locked_percentage: Decimal::one() - cfg.seed_split_to_liquid,
                                    liquid_percentage: cfg.seed_split_to_liquid,
                                }))
                                .unwrap(),
                            })
                            .unwrap(),
                            funds: vec![],
                        }),
                        _ => unreachable!(),
                    })
                }
                None => (),
            }

            Ok(res)
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
        ExecuteMsg::Vote { proposal_id, vote } => execute_vote(deps, env, info, proposal_id, vote),
        // Endowment Application review proposal & voting
        ExecuteMsg::ProposeApplication {
            ref_id,
            msg,
            latest,
            meta,
        } => execute_propose_application(deps, env, info, ref_id, msg, latest, meta),
        ExecuteMsg::VoteApplication {
            proposal_id,
            vote,
            reason,
        } => execute_vote_application(deps, env, info, proposal_id, vote, reason),
        ExecuteMsg::UpdateConfig {
            threshold,
            max_voting_period,
            require_execution,
            seed_asset,
            seed_split_to_liquid,
            new_endow_gas_money,
        } => execute_update_config(
            deps,
            env,
            info,
            require_execution,
            seed_asset,
            seed_split_to_liquid,
            new_endow_gas_money,
            threshold,
            max_voting_period,
        ),
        ExecuteMsg::Execute { proposal_id } => execute_execute(deps, env, info, proposal_id),
        ExecuteMsg::Close { proposal_id } => execute_close(deps, env, info, proposal_id),
        ExecuteMsg::MemberChangedHook(MemberChangedHookMsg { diffs }) => {
            execute_membership_hook(deps, env, info, diffs)
        }
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    require_execution: bool,
    seed_asset: Option<Asset>,
    seed_split_to_liquid: Decimal,
    new_endow_gas_money: Option<Coin>,
    threshold: Threshold,
    max_voting_period: Duration,
) -> Result<Response<Empty>, ContractError> {
    // only the contract can update own configs
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let mut cfg = CONFIG.load(deps.storage)?;
    cfg.require_execution = require_execution;
    cfg.threshold = threshold;
    cfg.max_voting_period = max_voting_period;

    // Check the seed token with "accepted_tokens"
    cfg.seed_asset = match seed_asset {
        Some(asset) => {
            let _token_check = validate_deposit_fund(
                deps.as_ref(),
                cfg.registrar_contract.as_str(),
                asset.clone(),
            )
            .unwrap();
            Some(asset)
        }
        None => None,
    };
    cfg.seed_split_to_liquid = seed_split_to_liquid;
    cfg.new_endow_gas_money = new_endow_gas_money;

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
    let mut prop = Proposal {
        proposal_type: ProposalType::Normal,
        title,
        description,
        start_height: env.block.height,
        expires,
        msgs,
        status: Status::Open,
        votes: Votes::new(vote_power),
        threshold: cfg.threshold,
        total_weight: cfg.group_addr.total_weight(&deps.querier)?,
        meta,
    };
    prop.update_status(&env.block);
    let id = next_id(deps.storage)?;
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
        .add_messages(direct_execute_msgs))
}

pub fn execute_propose_application(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ref_id: String,
    mut msg: CreateEndowmentMsg,
    latest: Option<Expiration>, // we ignore earliest
    meta: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Only charity endowments are processed through this CW3 for now
    if msg.endow_type != EndowmentType::Charity {
        return Err(ContractError::Unauthorized {});
    }

    // ensure charity specific params are set correctly (regardless of what user passes)
    msg.withdraw_before_maturity = false;
    msg.maturity_height = None;
    msg.maturity_time = None;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cfg.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;
    let accounts_contract = registrar_config.accounts_contract.unwrap().to_string();

    // check that at least 1 SDG category is set for charity endowments
    if msg.categories.sdgs.is_empty() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Invalid UN SDG inputs given".to_string(),
        }));
    }
    msg.categories.sdgs.sort();
    for item in msg.categories.sdgs.clone().into_iter() {
        if item > 17 || item == 0 {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Invalid UN SDG inputs given".to_string(),
            }));
        }
    }

    // max expires also used as default
    let max_expires = cfg.max_voting_period.after(&env.block);
    let mut expires = latest.unwrap_or(max_expires);
    let comp = expires.partial_cmp(&max_expires);
    if let Some(Ordering::Greater) = comp {
        expires = max_expires;
    } else if comp.is_none() {
        return Err(ContractError::WrongExpiration {});
    }

    // set the proposal_link so we can more easily query and endowment by it's proposal ID
    let prop_id = next_id(deps.storage)?;
    msg.proposal_link = Some(prop_id);

    // create an application for Endowment creation for review
    let mut prop = Proposal {
        proposal_type: ProposalType::Application,
        title: format!("{} Application - {}", msg.endow_type, ref_id),
        description: "".to_string(),
        start_height: env.block.height,
        expires,
        msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: accounts_contract,
            msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::CreateEndowment(msg))
                .unwrap(),
            funds: vec![],
        })],
        status: Status::Open,
        votes: Votes::new(0),
        threshold: cfg.threshold,
        total_weight: cfg.group_addr.total_weight(&deps.querier)?,
        meta,
    };
    prop.update_status(&env.block);
    PROPOSALS.save(deps.storage, prop_id, &prop)?;

    // If Proposal's status is Passed, then execute it immediately (if execution is not explicitly required)
    let mut direct_execute_msgs = vec![];
    let auto_execute = !cfg.require_execution && prop.status == Status::Passed;
    if auto_execute {
        direct_execute_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Execute {
                proposal_id: prop_id,
            })
            .unwrap(),
            funds: vec![],
        }));
    };

    Ok(Response::new()
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", prop_id.to_string())
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

    // ensure proposal exists and can be voted on and it is NOT
    // an Application ProposalType
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    if prop.status != Status::Open {
        return Err(ContractError::NotOpen {});
    }
    if prop.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }
    if prop.proposal_type == ProposalType::Application {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot vote on a Proposal that is an Application type".to_string(),
        }));
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
    };

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("auto-executed", auto_execute.to_string())
        .add_messages(direct_execute_msgs))
}

pub fn execute_vote_application(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: Vote,
    reason: Option<String>,
) -> Result<Response<Empty>, ContractError> {
    // only members of the multisig can vote
    let cfg = CONFIG.load(deps.storage)?;

    // ensure proposal exists and can be voted on
    // and that it is a Application ProposalType
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    if prop.status != Status::Open {
        return Err(ContractError::NotOpen {});
    }
    if prop.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }
    if prop.proposal_type != ProposalType::Application {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot vote on a Proposal that is not an Application type".to_string(),
        }));
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

    // If Vote == NO && Reason is given, set the Proposal description with reason for NO vote
    if vote == Vote::No {
        prop.description = reason.unwrap_or(prop.description);
    }

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
    };

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

    let mut res = Response::new()
        .add_attribute("action", "execute")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", proposal_id.to_string());

    // check if this is an Application proposal & send out as a submessage, so that we can catch the successful Endowment setup and follow
    // it up with a seed money top-up (set by CONFIG seed_asset & seed_split_to_liquid)
    // if Normal proposal treat as standard cosmos message sending
    res = match prop.proposal_type {
        ProposalType::Application => res.add_submessage(SubMsg::reply_on_success(
            prop.msgs[0].clone(),
            APPLICATION_REPLY_ID,
        )),
        _ => res.add_messages(prop.msgs),
    };

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
        registrar_contract: cfg.registrar_contract.to_string(),
        threshold: cfg.threshold,
        max_voting_period: cfg.max_voting_period,
        group_addr: cfg.group_addr.0.to_string(),
        require_execution: cfg.require_execution,
        seed_asset: cfg.seed_asset,
        seed_split_to_liquid: cfg.seed_split_to_liquid,
        new_endow_gas_money: cfg.new_endow_gas_money,
    })
}

fn query_threshold(deps: Deps) -> StdResult<ThresholdResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let total_weight = cfg.group_addr.total_weight(&deps.querier)?;
    Ok(cfg.threshold.to_response(total_weight))
}

fn query_proposal(deps: Deps, env: Env, id: u64) -> StdResult<MetaApplicationsProposalResponse> {
    let prop = PROPOSALS.load(deps.storage, id)?;
    let status = prop.current_status(&env.block);
    let threshold = prop.threshold.to_response(prop.total_weight);
    Ok(MetaApplicationsProposalResponse {
        id,
        title: prop.title,
        description: prop.description,
        msgs: prop.msgs,
        status,
        expires: prop.expires,
        threshold,
        proposal_type: prop.proposal_type,
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
) -> StdResult<MetaApplicationsProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let proposals = PROPOSALS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect::<StdResult<_>>()?;

    Ok(MetaApplicationsProposalListResponse { proposals })
}

fn reverse_proposals(
    deps: Deps,
    env: Env,
    start_before: Option<u64>,
    limit: Option<u32>,
) -> StdResult<MetaApplicationsProposalListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let end = start_before.map(Bound::exclusive);
    let props: StdResult<Vec<_>> = PROPOSALS
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|p| map_proposal(&env.block, p))
        .collect();

    Ok(MetaApplicationsProposalListResponse { proposals: props? })
}

fn map_proposal(
    block: &BlockInfo,
    item: StdResult<(u64, Proposal)>,
) -> StdResult<MetaApplicationsProposalResponse> {
    item.map(|(id, prop)| {
        let status = prop.current_status(block);
        let threshold = prop.threshold.to_response(prop.total_weight);
        MetaApplicationsProposalResponse {
            id,
            title: prop.title,
            description: prop.description,
            msgs: prop.msgs,
            status,
            expires: prop.expires,
            threshold,
            proposal_type: prop.proposal_type,
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

    // setup the new config struct and save to storage
    let data = deps
        .storage
        .get("config".as_bytes())
        .ok_or_else(|| StdError::not_found("Config not found"))?;
    let old_config: OldConfig = from_slice(&data)?;
    CONFIG.save(
        deps.storage,
        &Config {
            registrar_contract: old_config.registrar_contract,
            threshold: old_config.threshold,
            max_voting_period: old_config.max_voting_period,
            group_addr: old_config.group_addr,
            require_execution: false, // default to auto-execute
            seed_asset: None,         // default to NO seed funding program
            seed_split_to_liquid: Decimal::percent(0), // all to LOCKED
            new_endow_gas_money: None, // don't send any Juno
        },
    )?;

    Ok(Response::default())
}
