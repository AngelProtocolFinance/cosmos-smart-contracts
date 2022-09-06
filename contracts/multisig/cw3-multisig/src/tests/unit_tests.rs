use crate::msg::ExecuteMsg;
use angel_core::messages::cw3_multisig::{InstantiateMsg, QueryMsg};
use cosmwasm_std::{coin, coins, Addr, BankMsg, Coin, Decimal, Timestamp};
use cosmwasm_std::{BlockInfo, CosmosMsg, Empty};
use cw2::{query_contract_info, ContractVersion};
use cw3::{
    ProposalListResponse, ProposalResponse, Status, Vote, VoteInfo, VoteListResponse, VoteResponse,
    VoterDetail, VoterListResponse, VoterResponse,
};
use cw4::{Cw4ExecuteMsg, Member, MemberChangedHookMsg, MemberDiff};
use cw4_group::helpers::Cw4GroupContract;
use cw_multi_test::{next_block, App, Contract, ContractWrapper, Executor};
use cw_utils::{Duration, Expiration, Threshold, ThresholdResponse};

const CONTRACT_NAME: &str = "cw3-generic";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const OWNER: &str = "admin0001";
const APTEAM1: &str = "voter0001";
const APTEAM2: &str = "voter0002";
const APTEAM3: &str = "voter0003";
const APTEAM4: &str = "voter0004";
const APTEAM5: &str = "voter0005";
const ENDOWMENTOWNER1: &str = "owner0001";
const ENDOWMENTOWNER2: &str = "owner0002";
const PLEB: &str = "somebody";

fn member<T: Into<String>>(addr: T, weight: u64) -> Member {
    Member {
        addr: addr.into(),
        weight,
    }
}

pub fn contract_flex() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_group() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw4_group::contract::execute,
        cw4_group::contract::instantiate,
        cw4_group::contract::query,
    );
    Box::new(contract)
}

fn mock_app() -> App {
    App::default()
}

// uploads code and returns address of group contract
fn instantiate_group(app: &mut App, members: Vec<Member>) -> Addr {
    let group_id = app.store_code(contract_group());
    let msg = cw4_group::msg::InstantiateMsg {
        admin: Some(OWNER.into()),
        members,
    };
    app.instantiate_contract(group_id, Addr::unchecked(OWNER), &msg, &[], "group", None)
        .unwrap()
}

fn instantiate_flex(
    app: &mut App,
    group_addr: Addr,
    threshold: Threshold,
    max_voting_period: Duration,
) -> Addr {
    let flex_id = app.store_code(contract_flex());
    let msg = InstantiateMsg {
        registrar_contract: "registrar-contract".to_string(),
        group_addr: group_addr.to_string(),
        threshold,
        max_voting_period,
    };
    app.instantiate_contract(flex_id, Addr::unchecked(OWNER), &msg, &[], "flex", None)
        .unwrap()
}

// this will set up both contracts, instantiating the group with
// all voters defined above, and the multisig pointing to it and given threshold criteria.
// Returns (multisig address, group address).
fn setup_test_case_fixed(
    app: &mut App,
    weight_needed: u64,
    max_voting_period: Duration,
    init_funds: Vec<Coin>,
    multisig_as_group_admin: bool,
) -> (Addr, Addr, Addr) {
    setup_test_case(
        app,
        Threshold::AbsoluteCount {
            weight: weight_needed,
        },
        max_voting_period,
        init_funds,
        multisig_as_group_admin,
    )
}

fn setup_test_case(
    app: &mut App,
    threshold: Threshold,
    max_voting_period: Duration,
    init_funds: Vec<Coin>,
    multisig_as_group_admin: bool,
) -> (Addr, Addr, Addr) {
    // 1.   Instantiate Guardian Group contract with AP Team members (and OWNER as admin)
    //      Instantiate Endowment Owners contract with 2 endowment owners (and OWNER as admin)
    let ap_members = vec![
        member(OWNER, 0),
        member(APTEAM1, 1),
        member(APTEAM2, 2),
        member(APTEAM3, 3),
        member(APTEAM4, 4),
        member(APTEAM5, 5),
    ];
    let endowment_members = vec![
        member(OWNER, 0),
        member(ENDOWMENTOWNER1, 1),
        member(ENDOWMENTOWNER2, 2),
    ];
    let guardian_group = instantiate_group(app, ap_members);
    let endowment_group = instantiate_group(app, endowment_members);
    app.update_block(next_block);

    // 2. Set up Guardian Multisig backed by these groups
    let flex_addr = instantiate_flex(app, guardian_group.clone(), threshold, max_voting_period);
    app.update_block(next_block);

    // 3. (Optional) Set the multisig as the group owner
    if multisig_as_group_admin {
        let update_admin = Cw4ExecuteMsg::UpdateAdmin {
            admin: Some(flex_addr.to_string()),
        };
        app.execute_contract(
            Addr::unchecked(OWNER),
            guardian_group.clone(),
            &update_admin,
            &[],
        )
        .unwrap();
        app.update_block(next_block);
    }

    // Bonus: set some funds on the multisig contract for future proposals
    if !init_funds.is_empty() {
        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &flex_addr, init_funds)
                .unwrap()
        });
    }
    (flex_addr, guardian_group, endowment_group)
}

fn proposal_info() -> (Vec<CosmosMsg<Empty>>, String, String) {
    let bank_msg = BankMsg::Send {
        to_address: PLEB.into(),
        amount: coins(1, "BTC"),
    };
    let msgs = vec![bank_msg.into()];
    let title = "Pay somebody".to_string();
    let description = "Do I pay her?".to_string();
    (msgs, title, description)
}

fn pay_somebody_proposal() -> ExecuteMsg {
    let (msgs, title, description) = proposal_info();
    ExecuteMsg::Propose {
        title,
        description,
        msgs,
        latest: None,
        meta: Some("".to_string()),
    }
}

#[test]
fn test_instantiate_works() {
    let mut app = mock_app();

    // make a simple group
    let group_address = instantiate_group(&mut app, vec![member(OWNER, 1)]);
    let flex_id = app.store_code(contract_flex());

    let max_voting_period = Duration::Time(1234567);

    // Zero required weight fails
    let instantiate_msg = InstantiateMsg {
        registrar_contract: "registrar-contract".to_string(),
        group_addr: group_address.to_string(),
        threshold: Threshold::AbsoluteCount { weight: 0 },
        max_voting_period,
    };
    let _ = app
        .instantiate_contract(
            flex_id,
            Addr::unchecked(OWNER),
            &instantiate_msg,
            &[],
            "zero required weight",
            None,
        )
        .unwrap_err();

    // Total weight less than required weight not allowed
    let instantiate_msg = InstantiateMsg {
        registrar_contract: "registrar-contract".to_string(),
        group_addr: group_address.to_string(),
        threshold: Threshold::AbsoluteCount { weight: 100 },
        max_voting_period,
    };
    let _ = app
        .instantiate_contract(
            flex_id,
            Addr::unchecked(OWNER),
            &instantiate_msg,
            &[],
            "high required weight",
            None,
        )
        .unwrap_err();

    // All valid
    let instantiate_msg = InstantiateMsg {
        registrar_contract: "registrar-contract".to_string(),
        group_addr: group_address.to_string(),
        threshold: Threshold::AbsoluteCount { weight: 1 },
        max_voting_period,
    };
    let flex_addr = app
        .instantiate_contract(
            flex_id,
            Addr::unchecked(OWNER),
            &instantiate_msg,
            &[],
            "all good",
            None,
        )
        .unwrap();

    // Verify contract version set properly
    let version = query_contract_info(&app, flex_addr.clone()).unwrap();
    assert_eq!(
        ContractVersion {
            contract: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string(),
        },
        version,
    );

    // Get voters query
    let voters: VoterListResponse = app
        .wrap()
        .query_wasm_smart(
            &flex_addr,
            &QueryMsg::ListVoters {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(
        voters.voters,
        vec![VoterDetail {
            addr: OWNER.into(),
            weight: 1
        }]
    );
}

#[test]
fn test_propose_works() {
    let mut app = mock_app();

    let required_weight = 4;
    let voting_period = Duration::Time(2000000);
    let (flex_addr, _guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        false,
    );

    let proposal = pay_somebody_proposal();
    // Only voters can propose
    let _ = app
        .execute_contract(Addr::unchecked(PLEB), flex_addr.clone(), &proposal, &[])
        .unwrap_err();

    // Wrong expiration option fails
    let msgs = match proposal.clone() {
        ExecuteMsg::Propose { msgs, .. } => msgs,
        _ => panic!("Wrong variant"),
    };
    let proposal_wrong_exp = ExecuteMsg::Propose {
        title: "Rewarding somebody".to_string(),
        description: "Do we reward her?".to_string(),
        msgs,
        latest: Some(Expiration::AtHeight(123456)),
        meta: Some("".to_string()),
    };
    let _ = app
        .execute_contract(
            Addr::unchecked(OWNER),
            flex_addr.clone(),
            &proposal_wrong_exp,
            &[],
        )
        .unwrap_err();

    // Proposal from voter works
    let res = app
        .execute_contract(Addr::unchecked(APTEAM1), flex_addr.clone(), &proposal, &[])
        .unwrap();
    assert_eq!(
        res.custom_attrs(1),
        [
            ("action", "propose"),
            ("sender", APTEAM1),
            ("proposal_id", "1"),
            ("status", "Open"),
        ],
    );
}

fn get_tally(app: &App, flex_addr: &str, proposal_id: u64) -> u64 {
    // Get all the voters on the proposal
    let voters = QueryMsg::ListVotes {
        proposal_id,
        start_after: None,
        limit: None,
    };
    let votes: VoteListResponse = app.wrap().query_wasm_smart(flex_addr, &voters).unwrap();
    // Sum the weights of the Yes votes to get the tally
    votes
        .votes
        .iter()
        .filter(|&v| v.vote == Vote::Yes)
        .map(|v| v.weight)
        .sum()
}

fn expire(voting_period: Duration) -> impl Fn(&mut BlockInfo) {
    move |block: &mut BlockInfo| {
        match voting_period {
            Duration::Time(duration) => block.time = block.time.plus_seconds(duration + 1),
            Duration::Height(duration) => block.height += duration + 1,
        };
    }
}

fn unexpire(voting_period: Duration) -> impl Fn(&mut BlockInfo) {
    move |block: &mut BlockInfo| {
        match voting_period {
            Duration::Time(duration) => {
                block.time = Timestamp::from_nanos(block.time.nanos() - (duration * 1_000_000_000));
            }
            Duration::Height(duration) => block.height -= duration,
        };
    }
}

#[test]
fn test_proposal_queries() {
    let mut app = mock_app();

    let required_weight = 3;
    let voting_period = Duration::Time(2000000);
    let (flex_addr, _guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        false,
    );

    // create proposal with 0 vote power
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(APTEAM1), flex_addr.clone(), &proposal, &[])
        .unwrap();
    let proposal_id1: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

    // expire the proposal
    app.update_block(expire(voting_period));

    // add one more open proposal, 2 votes
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &proposal, &[])
        .unwrap();
    let proposal_id2: u64 = res.custom_attrs(1)[2].value.parse().unwrap();
    let proposed_at = app.block_info();

    // next block, let's query them all... make sure status is properly updated (1 should be rejected in query)
    app.update_block(next_block);
    let list_query = QueryMsg::ListProposals {
        start_after: None,
        limit: None,
    };
    let res: ProposalListResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &list_query)
        .unwrap();
    assert_eq!(2, res.proposals.len());

    // check the id and status are properly set
    let info: Vec<_> = res.proposals.iter().map(|p| (p.id, p.status)).collect();
    let expected_info = vec![
        (proposal_id1, Status::Rejected),
        (proposal_id2, Status::Open),
    ];
    assert_eq!(expected_info, info);

    // ensure the common features are set
    let (expected_msgs, expected_title, expected_description) = proposal_info();
    for prop in res.proposals {
        assert_eq!(prop.title, expected_title);
        assert_eq!(prop.description, expected_description);
        assert_eq!(prop.msgs, expected_msgs);
    }

    // reverse query can get just proposal_id2
    let list_query = QueryMsg::ReverseProposals {
        start_before: None,
        limit: Some(1),
    };
    let res: ProposalListResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &list_query)
        .unwrap();
    assert_eq!(1, res.proposals.len());

    let (msgs, title, description) = proposal_info();
    let expected = ProposalResponse {
        id: proposal_id2,
        title,
        description,
        msgs,
        expires: voting_period.after(&proposed_at),
        status: Status::Open,
        threshold: ThresholdResponse::AbsoluteCount {
            weight: 3,
            total_weight: 15,
        },
    };
    assert_eq!(&expected, &res.proposals[0]);
}

#[test]
fn test_vote_works() {
    let mut app = mock_app();

    let required_weight = 3;
    let voting_period = Duration::Time(2000000);
    let (flex_addr, _guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        false,
    );

    // create proposal with 0 vote power
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr.clone(), &proposal, &[])
        .unwrap();

    // Get the proposal id from the logs
    let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

    // Owner cannot vote (again)
    let yes_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::Yes,
    };
    let _ = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr.clone(), &yes_vote, &[])
        .unwrap_err();

    // Only voters can vote
    let _ = app
        .execute_contract(Addr::unchecked(PLEB), flex_addr.clone(), &yes_vote, &[])
        .unwrap_err();

    // But voter1 can
    let res = app
        .execute_contract(Addr::unchecked(APTEAM1), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    assert_eq!(
        res.custom_attrs(1),
        [
            ("action", "vote"),
            ("sender", APTEAM1),
            ("proposal_id", proposal_id.to_string().as_str()),
            ("status", "Open"),
        ],
    );

    // No/Veto votes have no effect on the tally
    // Compute the current tally
    let tally = get_tally(&app, flex_addr.as_ref(), proposal_id);
    assert_eq!(tally, 1);

    // Cast a No vote
    let no_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::No,
    };
    let _ = app
        .execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &no_vote, &[])
        .unwrap();

    // Cast a Veto vote
    let veto_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::Veto,
    };
    let _ = app
        .execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &veto_vote, &[])
        .unwrap();

    // Tally unchanged
    assert_eq!(tally, get_tally(&app, flex_addr.as_ref(), proposal_id));

    let _ = app
        .execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &yes_vote, &[])
        .unwrap_err();

    // Expired proposals cannot be voted
    app.update_block(expire(voting_period));
    let _ = app
        .execute_contract(Addr::unchecked(APTEAM4), flex_addr.clone(), &yes_vote, &[])
        .unwrap_err();

    app.update_block(unexpire(voting_period));

    // Powerful voter supports it, so it passes
    let res = app
        .execute_contract(Addr::unchecked(APTEAM4), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    assert_eq!(
        res.custom_attrs(1),
        [
            ("action", "vote"),
            ("sender", APTEAM4),
            ("proposal_id", proposal_id.to_string().as_str()),
            ("status", "Passed"),
        ],
    );

    // non-Open proposals cannot be voted
    let _ = app
        .execute_contract(Addr::unchecked(APTEAM5), flex_addr.clone(), &yes_vote, &[])
        .unwrap_err();

    // query individual votes
    // initial (with 0 weight)
    let voter = OWNER.into();
    let vote: VoteResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &QueryMsg::Vote { proposal_id, voter })
        .unwrap();
    assert_eq!(
        vote.vote.unwrap(),
        VoteInfo {
            proposal_id,
            voter: OWNER.into(),
            vote: Vote::Yes,
            weight: 0
        }
    );

    // nay sayer
    let voter = APTEAM2.into();
    let vote: VoteResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &QueryMsg::Vote { proposal_id, voter })
        .unwrap();
    assert_eq!(
        vote.vote.unwrap(),
        VoteInfo {
            proposal_id,
            voter: APTEAM2.into(),
            vote: Vote::No,
            weight: 2
        }
    );

    // non-voter
    let voter = APTEAM5.into();
    let vote: VoteResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &QueryMsg::Vote { proposal_id, voter })
        .unwrap();
    assert!(vote.vote.is_none());
}

#[test]
fn test_execute_works() {
    let mut app = mock_app();

    let required_weight = 3;
    let voting_period = Duration::Time(2000000);
    let (flex_addr, _guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        true,
    );

    // ensure we have cash to cover the proposal
    let contract_bal = app.wrap().query_balance(&flex_addr, "BTC").unwrap();
    assert_eq!(contract_bal, coin(10, "BTC"));

    // create proposal with 0 vote power
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr.clone(), &proposal, &[])
        .unwrap();

    // Get the proposal id from the logs
    let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

    // Only Passed can be executed
    let execution = ExecuteMsg::Execute { proposal_id };
    let _ = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr.clone(), &execution, &[])
        .unwrap_err();

    // Vote it, so it passes
    let vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::Yes,
    };
    let res = app
        .execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &vote, &[])
        .unwrap();
    assert_eq!(
        res.custom_attrs(1),
        [
            ("action", "vote"),
            ("sender", APTEAM3),
            ("proposal_id", proposal_id.to_string().as_str()),
            ("status", "Passed"),
        ],
    );

    // In passing: Try to close Passed fails
    let closing = ExecuteMsg::Close { proposal_id };
    let _ = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr.clone(), &closing, &[])
        .unwrap_err();

    // Execute works. Anybody can execute Passed proposals
    let res = app
        .execute_contract(Addr::unchecked(PLEB), flex_addr.clone(), &execution, &[])
        .unwrap();
    assert_eq!(
        res.custom_attrs(1),
        [
            ("action", "execute"),
            ("sender", PLEB),
            ("proposal_id", proposal_id.to_string().as_str()),
        ],
    );

    // verify money was transfered
    let some_bal = app.wrap().query_balance(PLEB, "BTC").unwrap();
    assert_eq!(some_bal, coin(1, "BTC"));
    let contract_bal = app.wrap().query_balance(&flex_addr, "BTC").unwrap();
    assert_eq!(contract_bal, coin(9, "BTC"));

    // In passing: Try to close Executed fails
    let _ = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr, &closing, &[])
        .unwrap_err();
}

#[test]
fn test_close_works() {
    let mut app = mock_app();

    let required_weight = 3;
    let voting_period = Duration::Height(2000000);
    let (flex_addr, _guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        true,
    );

    // create proposal with 0 vote power
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(OWNER), flex_addr.clone(), &proposal, &[])
        .unwrap();

    // Get the proposal id from the logs
    let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

    // Non-expired proposals cannot be closed
    let closing = ExecuteMsg::Close { proposal_id };
    let _ = app
        .execute_contract(Addr::unchecked(PLEB), flex_addr.clone(), &closing, &[])
        .unwrap_err();

    // Expired proposals can be closed
    app.update_block(expire(voting_period));
    let res = app
        .execute_contract(Addr::unchecked(PLEB), flex_addr.clone(), &closing, &[])
        .unwrap();
    assert_eq!(
        res.custom_attrs(1),
        [
            ("action", "close"),
            ("sender", PLEB),
            ("proposal_id", proposal_id.to_string().as_str()),
        ],
    );

    // Trying to close it again fails
    let closing = ExecuteMsg::Close { proposal_id };
    let _ = app
        .execute_contract(Addr::unchecked(PLEB), flex_addr, &closing, &[])
        .unwrap_err();
}

// uses the power from the beginning of the voting period
#[test]
fn execute_group_changes_from_external() {
    let mut app = mock_app();

    let required_weight = 4;
    let voting_period = Duration::Time(20000);
    let (flex_addr, guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        false,
    );

    // APTEAM1 starts a proposal to send some tokens (1/4 votes)
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(APTEAM1), flex_addr.clone(), &proposal, &[])
        .unwrap();
    // Get the proposal id from the logs
    let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();
    let prop_status = |app: &App, proposal_id: u64| -> Status {
        let query_prop = QueryMsg::Proposal { proposal_id };
        let prop: ProposalResponse = app
            .wrap()
            .query_wasm_smart(&flex_addr, &query_prop)
            .unwrap();
        prop.status
    };

    // 1/4 votes
    assert_eq!(prop_status(&app, proposal_id), Status::Open);

    // check current threshold (global)
    let threshold: ThresholdResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &QueryMsg::Threshold {})
        .unwrap();
    let expected_thresh = ThresholdResponse::AbsoluteCount {
        weight: 4,
        total_weight: 15,
    };
    assert_eq!(expected_thresh, threshold);

    // a few blocks later...
    app.update_block(|block| block.height += 2);

    // admin changes the group
    // updates APTEAM2 power to 7 -> with snapshot, vote doesn't pass proposal
    // adds NEWBIE with 2 power -> with snapshot, invalid vote
    // removes APTEAM3 -> with snapshot, can vote and pass proposal
    let newbie: &str = "newbie";
    let update_msg = cw4_group::msg::ExecuteMsg::UpdateMembers {
        remove: vec![APTEAM3.into()],
        add: vec![member(APTEAM2, 7), member(newbie, 2)],
    };
    app.execute_contract(Addr::unchecked(OWNER), guardian_group, &update_msg, &[])
        .unwrap();

    // check membership queries properly updated
    let query_voter = QueryMsg::Voter {
        address: APTEAM3.into(),
    };
    let power: VoterResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &query_voter)
        .unwrap();
    assert_eq!(power.weight, None);

    // proposal still open
    assert_eq!(prop_status(&app, proposal_id), Status::Open);

    // a few blocks later...
    app.update_block(|block| block.height += 3);

    // make a second proposal
    let proposal2 = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(APTEAM1), flex_addr.clone(), &proposal2, &[])
        .unwrap();
    // Get the proposal id from the logs
    let proposal_id2: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

    // APTEAM2 can pass this alone with the updated vote (newer height ignores snapshot)
    let yes_vote = ExecuteMsg::Vote {
        proposal_id: proposal_id2,
        vote: Vote::Yes,
    };
    app.execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    assert_eq!(prop_status(&app, proposal_id2), Status::Passed);

    // APTEAM2 can only vote on first proposal with weight of 2 (not enough to pass)
    let yes_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::Yes,
    };
    app.execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    assert_eq!(prop_status(&app, proposal_id), Status::Open);

    // newbie cannot vote
    let _ = app
        .execute_contract(Addr::unchecked(newbie), flex_addr.clone(), &yes_vote, &[])
        .unwrap_err();

    // previously removed APTEAM3 can still vote, passing the proposal
    app.execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    assert_eq!(prop_status(&app, proposal_id), Status::Passed);

    // check current threshold (global) is updated
    let threshold: ThresholdResponse = app
        .wrap()
        .query_wasm_smart(&flex_addr, &QueryMsg::Threshold {})
        .unwrap();
    let expected_thresh = ThresholdResponse::AbsoluteCount {
        weight: 4,
        total_weight: 19,
    };
    assert_eq!(expected_thresh, threshold);

    // TODO: check proposal threshold not changed
}

// uses the power from the beginning of the voting period
// similar to above - simpler case, but shows that one proposals can
// trigger the action
#[test]
fn execute_group_changes_from_proposal() {
    let mut app = mock_app();

    let required_weight = 4;
    let voting_period = Duration::Time(20000);
    let (flex_addr, guardian_group, _endowment_group) = setup_test_case_fixed(
        &mut app,
        required_weight,
        voting_period,
        coins(10, "BTC"),
        true,
    );

    // Start a proposal to remove APTEAM3 from the set
    let update_msg = Cw4GroupContract::new(guardian_group)
        .update_members(vec![APTEAM3.into()], vec![])
        .unwrap();
    let update_proposal = ExecuteMsg::Propose {
        title: "Kick out APTEAM3".to_string(),
        description: "He's trying to steal our money".to_string(),
        msgs: vec![update_msg],
        latest: None,
        meta: Some("".to_string()),
    };
    let res = app
        .execute_contract(
            Addr::unchecked(APTEAM1),
            flex_addr.clone(),
            &update_proposal,
            &[],
        )
        .unwrap();
    // Get the proposal id from the logs
    let update_proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

    // next block...
    app.update_block(|b| b.height += 1);

    // APTEAM1 starts a proposal to send some tokens
    let cash_proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(
            Addr::unchecked(APTEAM1),
            flex_addr.clone(),
            &cash_proposal,
            &[],
        )
        .unwrap();
    // Get the proposal id from the logs
    let cash_proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();
    assert_ne!(cash_proposal_id, update_proposal_id);

    // query proposal state
    let prop_status = |app: &App, proposal_id: u64| -> Status {
        let query_prop = QueryMsg::Proposal { proposal_id };
        let prop: ProposalResponse = app
            .wrap()
            .query_wasm_smart(&flex_addr, &query_prop)
            .unwrap();
        prop.status
    };
    assert_eq!(prop_status(&app, cash_proposal_id), Status::Open);
    assert_eq!(prop_status(&app, update_proposal_id), Status::Open);

    // next block...
    app.update_block(|b| b.height += 1);

    // Pass and execute first proposal
    let yes_vote = ExecuteMsg::Vote {
        proposal_id: update_proposal_id,
        vote: Vote::Yes,
    };
    app.execute_contract(Addr::unchecked(APTEAM4), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    let execution = ExecuteMsg::Execute {
        proposal_id: update_proposal_id,
    };
    app.execute_contract(Addr::unchecked(APTEAM4), flex_addr.clone(), &execution, &[])
        .unwrap();

    // ensure that the update_proposal is executed, but the other unchanged
    assert_eq!(prop_status(&app, update_proposal_id), Status::Executed);
    assert_eq!(prop_status(&app, cash_proposal_id), Status::Open);

    // next block...
    app.update_block(|b| b.height += 1);

    // APTEAM3 can still pass the cash proposal
    // voting on it fails
    let yes_vote = ExecuteMsg::Vote {
        proposal_id: cash_proposal_id,
        vote: Vote::Yes,
    };
    app.execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    assert_eq!(prop_status(&app, cash_proposal_id), Status::Passed);

    // but cannot open a new one
    let cash_proposal = pay_somebody_proposal();
    let _ = app
        .execute_contract(
            Addr::unchecked(APTEAM3),
            flex_addr.clone(),
            &cash_proposal,
            &[],
        )
        .unwrap_err();

    // extra: ensure no one else can call the hook
    let hook_hack = ExecuteMsg::MemberChangedHook(MemberChangedHookMsg {
        diffs: vec![MemberDiff::new(APTEAM1, Some(1), None)],
    });
    let _ = app
        .execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &hook_hack, &[])
        .unwrap_err();
}

// // uses the power from the beginning of the voting period
// #[test]
// fn percentage_handles_group_changes() {
//     let mut app = mock_app();

//     // 33% required, which is 5 of the initial 15
//     let voting_period = Duration::Time(20000);
//     let (flex_addr, guardian_group, _endowment_group) = setup_test_case(
//         &mut app,
//         Threshold::AbsolutePercentage {
//             percentage: Decimal::percent(33),
//         },
//         voting_period,
//         coins(10, "BTC"),
//         false,
//     );

//     // APTEAM3 starts a proposal to send some tokens (3/5 votes)
//     let proposal = pay_somebody_proposal();
//     let res = app
//         .execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &proposal, &[])
//         .unwrap();
//     // Get the proposal id from the logs
//     let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();
//     let prop_status = |app: &App| -> Status {
//         let query_prop = QueryMsg::Proposal { proposal_id };
//         let prop: ProposalResponse = app
//             .wrap()
//             .query_wasm_smart(&flex_addr, &query_prop)
//             .unwrap();
//         prop.status
//     };

//     // 3/5 votes
//     assert_eq!(prop_status(&app), Status::Open);

//     // a few blocks later...
//     app.update_block(|block| block.height += 2);

//     // admin changes the group (3 -> 0, 2 -> 7, 0 -> 15) - total = 32, require 11 to pass
//     let newbie: &str = "newbie";
//     let update_msg = cw4_group::msg::ExecuteMsg::UpdateMembers {
//         remove: vec![APTEAM3.into()],
//         add: vec![member(APTEAM2, 7), member(newbie, 15)],
//     };
//     app.execute_contract(Addr::unchecked(OWNER), guardian_group, &update_msg, &[])
//         .unwrap();

//     // a few blocks later...
//     app.update_block(|block| block.height += 3);

//     // APTEAM2 votes according to original weights: 3 + 2 = 5 / 5 => Passed
//     // with updated weights, it would be 3 + 7 = 10 / 11 => Open
//     let yes_vote = ExecuteMsg::Vote {
//         proposal_id,
//         vote: Vote::Yes,
//     };
//     app.execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &yes_vote, &[])
//         .unwrap();
//     assert_eq!(prop_status(&app), Status::Passed);

//     // new proposal can be passed single-handedly by newbie
//     let proposal = pay_somebody_proposal();
//     let res = app
//         .execute_contract(Addr::unchecked(newbie), flex_addr.clone(), &proposal, &[])
//         .unwrap();
//     // Get the proposal id from the logs
//     let proposal_id2: u64 = res.custom_attrs(1)[2].value.parse().unwrap();

//     // check proposal2 status
//     let query_prop = QueryMsg::Proposal {
//         proposal_id: proposal_id2,
//     };
//     let prop: ProposalResponse = app
//         .wrap()
//         .query_wasm_smart(&flex_addr, &query_prop)
//         .unwrap();
//     assert_eq!(Status::Passed, prop.status);
// }

// uses the power from the beginning of the voting period
#[test]
fn quorum_handles_group_changes() {
    let mut app = mock_app();

    // 33% required for quora, which is 5 of the initial 15
    // 50% yes required to pass early (8 of the initial 15)
    let voting_period = Duration::Time(20000);
    let (flex_addr, guardian_group, _endowment_group) = setup_test_case(
        &mut app,
        Threshold::ThresholdQuorum {
            threshold: Decimal::percent(50),
            quorum: Decimal::percent(33),
        },
        voting_period,
        coins(10, "BTC"),
        false,
    );

    // APTEAM3 starts a proposal to send some tokens (3 votes)
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &proposal, &[])
        .unwrap();
    // Get the proposal id from the logs
    let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();
    let prop_status = |app: &App| -> Status {
        let query_prop = QueryMsg::Proposal { proposal_id };
        let prop: ProposalResponse = app
            .wrap()
            .query_wasm_smart(&flex_addr, &query_prop)
            .unwrap();
        prop.status
    };

    // 3/5 votes - not expired
    assert_eq!(prop_status(&app), Status::Open);

    // a few blocks later...
    app.update_block(|block| block.height += 2);

    // admin changes the group (3 -> 0, 2 -> 7, 0 -> 15) - total = 32, require 11 to pass
    let newbie: &str = "newbie";
    let update_msg = cw4_group::msg::ExecuteMsg::UpdateMembers {
        remove: vec![APTEAM3.into()],
        add: vec![member(APTEAM2, 7), member(newbie, 15)],
    };
    app.execute_contract(Addr::unchecked(OWNER), guardian_group, &update_msg, &[])
        .unwrap();

    // a few blocks later...
    app.update_block(|block| block.height += 3);

    // APTEAM2 votes no, according to original weights: 3 yes, 2 no, 5 total (will pass when expired)
    // with updated weights, it would be 3 yes, 7 no, 10 total (will fail when expired)
    let yes_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::No,
    };
    app.execute_contract(Addr::unchecked(APTEAM2), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    // not expired yet
    assert_eq!(prop_status(&app), Status::Open);

    // wait until the vote is over, and see it was passed (met quorum, and threshold of voters)
    app.update_block(expire(voting_period));
    assert_eq!(prop_status(&app), Status::Passed);
}

#[test]
fn quorum_enforced_even_if_absolute_threshold_met() {
    let mut app = mock_app();

    // 33% required for quora, which is 5 of the initial 15
    // 50% yes required to pass early (8 of the initial 15)
    let voting_period = Duration::Time(20000);
    let (flex_addr, _guardian_group, _endowment_group) = setup_test_case(
        &mut app,
        // note that 60% yes is not enough to pass without 20% no as well
        Threshold::ThresholdQuorum {
            threshold: Decimal::percent(60),
            quorum: Decimal::percent(80),
        },
        voting_period,
        coins(10, "BTC"),
        false,
    );

    // create proposal
    let proposal = pay_somebody_proposal();
    let res = app
        .execute_contract(Addr::unchecked(APTEAM5), flex_addr.clone(), &proposal, &[])
        .unwrap();
    // Get the proposal id from the logs
    let proposal_id: u64 = res.custom_attrs(1)[2].value.parse().unwrap();
    let prop_status = |app: &App| -> Status {
        let query_prop = QueryMsg::Proposal { proposal_id };
        let prop: ProposalResponse = app
            .wrap()
            .query_wasm_smart(&flex_addr, &query_prop)
            .unwrap();
        prop.status
    };
    assert_eq!(prop_status(&app), Status::Open);
    app.update_block(|block| block.height += 3);

    // reach 60% of yes votes, not enough to pass early (or late)
    let yes_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::Yes,
    };
    app.execute_contract(Addr::unchecked(APTEAM4), flex_addr.clone(), &yes_vote, &[])
        .unwrap();
    // 9 of 15 is 60% absolute threshold, but less than 12 (80% quorum needed)
    assert_eq!(prop_status(&app), Status::Open);

    // add 3 weight no vote and we hit quorum and this passes
    let no_vote = ExecuteMsg::Vote {
        proposal_id,
        vote: Vote::No,
    };
    app.execute_contract(Addr::unchecked(APTEAM3), flex_addr.clone(), &no_vote, &[])
        .unwrap();
    assert_eq!(prop_status(&app), Status::Passed);
}
