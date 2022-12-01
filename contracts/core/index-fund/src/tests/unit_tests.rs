use angel_core::errors::core::*;
use angel_core::messages::index_fund::*;
use angel_core::responses::index_fund::*;
use angel_core::structs::{AllianceMember, IndexFund, SplitDetails};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, Timestamp, Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;

use crate::contract::{execute, instantiate, migrate, query};
use crate::executers::{calculate_split, rotate_fund};

use super::mock_querier::mock_dependencies;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let _pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.active_fund, 0);
}

#[test]
fn only_sc_owner_can_change_owner() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // non-owner cannot change the SC owner
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateOwner {
            new_owner: String::from("some-rando-owner"),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // change the admin to some pleb
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateOwner {
            new_owner: String::from("some-rando-owner"),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check that the configs are set in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(String::from("some-rando-owner"), value.owner);
}

#[test]
fn only_registrar_can_change_registrar_contract() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // non-registrar cannot change the registrar SC addr
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateRegistrar {
            new_registrar: String::from("some-rando-registrar"),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // change the registrar SC to some pleb from the Registrar SC
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateRegistrar {
            new_registrar: String::from("some-legit-registrar"),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check that the configs are set in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        String::from("some-legit-registrar"),
        value.registrar_contract
    );
}

#[test]
fn only_owner_can_update_config() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // non-owner cannot update the config
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateConfig(UpdateConfigMsg {
            fund_member_limit: None,
            fund_rotation: None,
            funding_goal: None,
        }),
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // New config.funding_goal should meet some conditions
    let info = mock_info(&ap_team, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(UpdateConfigMsg {
            fund_member_limit: Some(40),
            fund_rotation: Some(100_u64),
            funding_goal: Some(Uint128::zero()),
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // only owner can update the config
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateConfig(UpdateConfigMsg {
            fund_member_limit: Some(40),
            fund_rotation: Some(100_u64),
            funding_goal: Some(Uint128::from(123_u128)),
        }),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check that the configs are set in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(40, config.fund_member_limit);
    assert_eq!(Some(100_u64), config.fund_rotation);
    assert_eq!(Some(Uint128::from(123_u128)), config.funding_goal);
}

#[test]
fn only_owner_can_update_alliance_member_list() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the alliance member
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AllianceMember {
            address: Addr::unchecked("address"),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        cosmwasm_std::StdError::GenericErr {
            msg: "Cannot find member".to_string()
        }
    );

    // non-owner cannot update the alliance member list
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateAllianceMemberList {
            address: Addr::unchecked("address"),
            member: AllianceMember {
                name: "new alliance member".to_string(),
                logo: None,
                website: None,
            },
            action: "add".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // only owner can add the alliance member list
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateAllianceMemberList {
            address: Addr::unchecked("address"),
            member: AllianceMember {
                name: "new alliance member".to_string(),
                logo: None,
                website: None,
            },
            action: "add".to_string(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check the result of update
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AllianceMembers {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let alliance_members: AllianceMemberListResponse = from_binary(&res).unwrap();
    assert_eq!(alliance_members.alliance_members.len(), 1);

    // Cannot update since invalid action
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateAllianceMemberList {
            address: Addr::unchecked("address"),
            member: AllianceMember {
                name: "new alliance member".to_string(),
                logo: None,
                website: None,
            },
            action: "remvoe".to_string(),
        },
    )
    .unwrap_err();

    // only owner can remove the alliance member list
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateAllianceMemberList {
            address: Addr::unchecked("address"),
            member: AllianceMember {
                name: "new alliance member".to_string(),
                logo: None,
                website: None,
            },
            action: "remove".to_string(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check the result of update
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AllianceMembers {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let alliance_members: AllianceMemberListResponse = from_binary(&res).unwrap();
    assert_eq!(alliance_members.alliance_members.len(), 0);
}

#[test]
fn sc_owner_can_add_remove_funds() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let new_fund_msg = ExecuteMsg::CreateFund {
        name: String::from("Ending Hunger"),
        description: String::from("Some fund of charities"),
        members: vec![],
        rotating_fund: Some(true),
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
    };

    let new_fund_msg1 = ExecuteMsg::CreateFund {
        name: String::from("Ending Hunger"),
        description: String::from("Some fund of charities"),
        members: vec![],
        rotating_fund: Some(true),
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
    };
    let remove_fund_msg = ExecuteMsg::RemoveFund { fund_id: 1 };

    // pleb cannot add funds (only SC owner should be able to)
    let info = mock_info(&pleb.clone(), &coins(1000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, new_fund_msg.clone()).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // real SC owner adds a fund
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        new_fund_msg.clone(),
    )
    .unwrap();
    let _res = execute(deps.as_mut(), mock_env(), info, new_fund_msg1).unwrap();
    assert_eq!(0, res.messages.len());

    // check that the fund can be fetched in a query to FundsList
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::FundsList {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let value: FundListResponse = from_binary(&res).unwrap();
    assert_eq!(2, value.funds.len());

    // pleb cannot remove funds (only SC owner should be able to)
    let info = mock_info(&pleb.clone(), &coins(1000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, remove_fund_msg.clone()).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // real SC owner removes a fund
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, remove_fund_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // check that the fund in FundsList is expired
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::FundsList {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let value: FundListResponse = from_binary(&res).unwrap();
    assert_eq!(1, value.funds.len());
    assert_eq!(value.funds[0].expiry_height, None);
    // assert_eq!(value.funds[0].expiry_height, Some(mock_env().block.height));

    // check active fund after remove current fund
    let res = query(deps.as_ref(), mock_env(), QueryMsg::ActiveFundDetails {}).unwrap();
    let value: FundDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(2, value.fund.unwrap().id);
}

#[test]
fn sc_owner_can_update_fund_members() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let _charity_addr = "charity-address".to_string();
    let registrar_contract = "registrar-account".to_string();
    let pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let new_fund_msg = ExecuteMsg::CreateFund {
        name: String::from("Ending Hunger"),
        description: String::from("Some fund of charities"),
        members: vec![],
        rotating_fund: Some(true),
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
    };
    let update_members_msg = ExecuteMsg::UpdateMembers {
        fund_id: 1,
        add: vec![1, 3],
        remove: vec![2],
    };

    // real SC owner adds a fund
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, new_fund_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // pleb cannot update fund members (only SC owner should be able to)
    let info = mock_info(&pleb.clone(), &coins(1000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, update_members_msg.clone()).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // real SC owner updates fund members
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, update_members_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // check that the fund members are accurate in query results
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::FundDetails { fund_id: 1 },
    )
    .unwrap();
    let value: FundDetailsResponse = from_binary(&res).unwrap();
    let f = value.fund.unwrap();
    assert_eq!(2, f.members.len());
}

#[test]
fn sc_owner_can_update_alliance_member() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Try to update the alliance member
    // Fails since non-owner calls the entry
    let info = mock_info("anyone", &[]);
    let update_alliance_member_msg = ExecuteMsg::UpdateAllianceMember {
        address: Addr::unchecked("address"),
        member: AllianceMember {
            name: "new alliance member".to_string(),
            logo: None,
            website: None,
        },
    };
    let err = execute(deps.as_mut(), mock_env(), info, update_alliance_member_msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the alliance member
    let info = mock_info(ap_team.as_ref(), &[]);
    let update_alliance_member_msg = ExecuteMsg::UpdateAllianceMember {
        address: Addr::unchecked("address"),
        member: AllianceMember {
            name: "new alliance member".to_string(),
            logo: None,
            website: None,
        },
    };
    let res = execute(deps.as_mut(), mock_env(), info, update_alliance_member_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the added alliance member
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AllianceMember {
            address: Addr::unchecked("address"),
        },
    )
    .unwrap();
    let alliance_member: AllianceMemberResponse = from_binary(&res).unwrap();
    assert_eq!(alliance_member.name, "new alliance member");
    assert_eq!(alliance_member.wallet, "address");
    assert_eq!(alliance_member.logo, None);
    assert_eq!(alliance_member.website, None);
}

#[test]
fn sc_owner_can_remove_member() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let _charity_addr = "charity-address".to_string();
    let registrar_contract = "registrar-account".to_string();
    let accounts_contract = "accounts_contract_addr".to_string();
    let _pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Add the fund
    let new_fund_msg = ExecuteMsg::CreateFund {
        name: String::from("Ending Hunger"),
        description: String::from("Some fund of charities"),
        members: vec![],
        rotating_fund: Some(true),
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, new_fund_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // Cannot add members which exceeds the number of limit
    let update_members_msg = ExecuteMsg::UpdateMembers {
        fund_id: 1,
        add: (1..30).collect::<Vec<u32>>(),
        remove: vec![2],
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, update_members_msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::IndexFundMembershipExceeded {});

    // Update the fund members
    let update_members_msg = ExecuteMsg::UpdateMembers {
        fund_id: 1,
        add: vec![1, 3],
        remove: vec![2],
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, update_members_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the result of addition
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::FundDetails { fund_id: 1 },
    )
    .unwrap();
    let fund_detail: FundDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(fund_detail.fund.unwrap().members.len(), 2);

    // Try to remove the member
    // Fails since non-accounts_contract calls the entry
    let remove_member_msg = RemoveMemberMsg { member: 1 };
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RemoveMember(remove_member_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to remove the member
    let remove_member_msg = RemoveMemberMsg { member: 1 };
    let info = mock_info(accounts_contract.as_ref(), &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RemoveMember(remove_member_msg),
    )
    .unwrap();

    // Check the result of removal
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::FundDetails { fund_id: 1 },
    )
    .unwrap();
    let fund_detail: FundDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(fund_detail.fund.unwrap().members.len(), 1);
}

#[test]
fn test_receive_cw20() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();
    let _pleb = "pleb-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // First, create fund
    let create_fund_msg = ExecuteMsg::CreateFund {
        name: "test fund".to_string(),
        description: "test fund desc".to_string(),
        members: vec![3],
        rotating_fund: None,
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
    };
    let info = mock_info(&ap_team.clone(), &[]);
    let _ = execute(deps.as_mut(), mock_env(), info, create_fund_msg).unwrap();

    // Deposit cw20 token
    let deposit_msg = DepositMsg {
        fund_id: Some(1),
        split: None,
    };
    let real_deposit_msg = Cw20ReceiveMsg {
        sender: ap_team.clone(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&ReceiveMsg::Deposit(deposit_msg)).unwrap(),
    };
    let info = mock_info("test-cw20", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(real_deposit_msg),
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
}

#[test]
fn rotate_funds() {
    let index_fund_1 = IndexFund {
        id: 1,
        name: "Fund #1".to_string(),
        description: "Fund number 1 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
        rotating_fund: Some(true),
    };
    let index_fund_2 = IndexFund {
        id: 2,
        name: "Fund #2".to_string(),
        description: "Fund number 2 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
        rotating_fund: Some(true),
    };

    let new_fund_1 = rotate_fund(
        vec![index_fund_1.clone()],
        1,
        10,
        Timestamp::from_seconds(100),
    );
    assert_eq!(new_fund_1, 1);
    let new_fund_2 = rotate_fund(
        vec![index_fund_1.clone(), index_fund_2.clone()],
        1,
        10,
        Timestamp::from_seconds(100),
    );
    assert_eq!(new_fund_2, 2);
    let new_fund_3 = rotate_fund(
        vec![index_fund_1, index_fund_2],
        2,
        10,
        Timestamp::from_seconds(100),
    );
    assert_eq!(new_fund_3, 1);
}

#[test]
fn rotate_funds_with_expired_funds() {
    let index_fund_1 = IndexFund {
        id: 1,
        name: "Fund #1".to_string(),
        description: "Fund number 1 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
        rotating_fund: Some(true),
    };
    let index_fund_2 = IndexFund {
        id: 2,
        name: "Fund #2".to_string(),
        description: "Fund number 2 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: Some(10),
        rotating_fund: Some(false),
    };
    let index_fund_3 = IndexFund {
        id: 3,
        name: "Fund #3".to_string(),
        description: "Fund number 3 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: Some(1000),
        expiry_height: Some(1000),
        rotating_fund: Some(true),
    };

    let new_fund_1 = rotate_fund(
        vec![index_fund_1.clone()],
        1,
        100,
        Timestamp::from_seconds(10000),
    );
    assert_eq!(new_fund_1, 1);

    let new_fund_2 = rotate_fund(
        vec![index_fund_2.clone(), index_fund_1.clone()],
        1,
        100,
        Timestamp::from_seconds(10000),
    );
    assert_eq!(new_fund_2, 1);

    let new_fund_3 = rotate_fund(
        vec![index_fund_3, index_fund_1, index_fund_2],
        1,
        100,
        Timestamp::from_seconds(10000),
    );
    assert_eq!(new_fund_3, 1);
}

#[test]
fn test_tca_without_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(calculate_split(true, sc_split, None, None), Decimal::zero());
}

#[test]
fn test_tca_with_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(
        calculate_split(true, sc_split, None, Some(Decimal::percent(42))),
        Decimal::zero()
    );
}

#[test]
fn test_non_tca_with_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(
        calculate_split(false, sc_split, None, Some(Decimal::percent(23))),
        Decimal::percent(23)
    );
}

#[test]
fn test_non_tca_without_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(
        calculate_split(false, sc_split.clone(), None, None),
        sc_split.default
    );
}

#[test]
fn test_deposit() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Cannot deposit several tokens at once
    let info = mock_info(
        &ap_team.clone(),
        &[
            Coin {
                denom: "ujuno".to_string(),
                amount: Uint128::from(100_u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100_u128),
            },
        ],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: None,
            split: None,
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Deposit fund should be one of accepted tokens
    let info = mock_info(
        &ap_team.clone(),
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: None,
            split: None,
        }),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "Not accepted token: uusd".to_string()
        })
    );

    // Cannot deposit zero balance
    let info = mock_info(
        &ap_team.clone(),
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::zero(),
        }],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: None,
            split: None,
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // There SHOULD be active fund before any deposit
    let info = mock_info(
        &ap_team.clone(),
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let _err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: None,
            split: None,
        }),
    )
    .unwrap_err();

    // Create fund
    let new_fund_msg = ExecuteMsg::CreateFund {
        name: String::from("Ending Hunger"),
        description: String::from("Some fund of charities"),
        members: vec![],
        rotating_fund: Some(true),
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        new_fund_msg.clone(),
    )
    .unwrap();

    // fund should have the members
    let info = mock_info(
        &ap_team.clone(),
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: None,
            split: None,
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::IndexFundEmpty {});

    let info = mock_info(
        &ap_team.clone(),
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: Some(1),
            split: None,
        }),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::IndexFundEmpty {});

    // Add the fund members
    let info = mock_info(&ap_team.clone(), &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateMembers {
            fund_id: 1,
            add: vec![1],
            remove: vec![],
        },
    )
    .unwrap();

    // Succeed to deposit funds
    let info = mock_info(
        &ap_team.clone(),
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit(DepositMsg {
            fund_id: None,
            split: None,
        }),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    // Same logic applies to cw20 token deposit
    let info = mock_info("test-cw20", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: ap_team.clone(),
            msg: to_binary(&ReceiveMsg::Deposit(DepositMsg {
                fund_id: Some(1),
                split: None,
            }))
            .unwrap(),
            amount: Uint128::from(100_u128),
        }),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    // Check the fund state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::ActiveFundDonations {}).unwrap();
    let donate_list: DonationListResponse = from_binary(&res).unwrap();
    assert_eq!(donate_list.donors.len(), 0);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::InvolvedFunds { endowment_id: 1 },
    )
    .unwrap();
    let involved_funds: FundListResponse = from_binary(&res).unwrap();
    assert_eq!(involved_funds.funds.len(), 1);
}

#[test]
fn test_migrate() {
    let mut deps = mock_dependencies(&[]);
    // Instantiate the contract
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Migrate
    let _err = migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap_err();
}

#[test]
fn test_query_msg_builder() {
    let mut deps = mock_dependencies(&[]);
    // Instantiate the contract
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "registrar-account".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
    };
    let info = mock_info(&ap_team.clone(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Query
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Deposit {
            token_denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
            fund_id: Some(1),
            split: None,
        },
    )
    .unwrap();

    let msg: CosmosMsg = from_binary(&res).unwrap();
    assert_eq!(
        msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&ExecuteMsg::Deposit(DepositMsg {
                fund_id: Some(1),
                split: None
            }))
            .unwrap(),
            funds: vec![Coin {
                denom: "ujuno".to_string(),
                amount: Uint128::from(100_u128)
            }]
        })
    )
}
