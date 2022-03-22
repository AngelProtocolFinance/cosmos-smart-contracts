use crate::contract::{execute, instantiate, migrate, query};
use angel_core::errors::core::*;
use angel_core::messages::index_fund::*;
use angel_core::responses::index_fund::*;
use angel_core::structs::{AllianceMember, IndexFund};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{attr, coins, from_binary, Addr};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let _pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn only_sc_owner_can_change_owner() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
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
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
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
    let info = mock_info(registrar_contract.as_ref(), &coins(100000, "earth"));
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
fn migrate_contract() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let _pleb = "plebAccount".to_string();

    let instantiate_msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // try to migrate the contract
    let msg = MigrateMsg {
        active_fund: 0,
        next_fund_id: 0,
    }; // just place_holder
    let res = migrate(deps.as_mut(), env.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len())
}

#[test]
fn sc_owner_can_update_list_of_tca_members() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg1 = ExecuteMsg::UpdateTcaList {
        address: Addr::unchecked(charity_addr.as_str()),
        member: AllianceMember {
            name: "charity".to_string(),
            logo: None,
            website: None,
        },
        action: "add".to_string(),
    };

    let msg2 = ExecuteMsg::UpdateTcaList {
        address: Addr::unchecked(pleb.as_str()),
        member: AllianceMember {
            name: "pleb".to_string(),
            logo: None,
            website: None,
        },
        action: "add".to_string(),
    };
    // pleb cannot update the list (only owner should be able to)
    let info = mock_info(&pleb.clone(), &coins(1000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg1.clone()).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // real SC owner updates the list now
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg1.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("method", "update_tca_list"),
            attr("action", "add"),
            attr("address", charity_addr),
        ]
    );

    // check that the list can be fetched in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::TcaList {}).unwrap();
    let value: TcaListResponse = from_binary(&res).unwrap();
    assert_eq!(1, value.tca_members.len());

    // real SC owner updates the list again
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg2.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("method", "update_tca_list"),
            attr("action", "add"),
            attr("address", pleb.clone()),
        ]
    );

    // check that the list can be fetched in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::TcaList {}).unwrap();
    let value: TcaListResponse = from_binary(&res).unwrap();
    assert_eq!(2, value.tca_members.len());

    // real SC owner removes the member from list
    let msg3 = ExecuteMsg::UpdateTcaList {
        address: Addr::unchecked(pleb.as_str()),
        member: AllianceMember {
            name: "pleb".to_string(),
            logo: None,
            website: None,
        },
        action: "remove".to_string(),
    };

    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg3.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("method", "update_tca_list"),
            attr("action", "remove"),
            attr("address", pleb),
        ]
    );

    // check that the list can be fetched in query
    let res = query(deps.as_ref(), mock_env(), QueryMsg::TcaList {}).unwrap();
    let value: TcaListResponse = from_binary(&res).unwrap();
    assert_eq!(1, value.tca_members.len());
}

#[test]
fn sc_owner_can_add_remove_funds() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
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
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
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
        add: vec![charity_addr.clone(), String::from("CHARITYGSDRGSDRGSDRGFG")],
        remove: vec![pleb.clone()],
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
fn sc_owner_can_update_alliancemember() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let msg = InstantiateMsg {
        registrar_contract: registrar_contract.clone(),
        fund_rotation: Some(Some(1000000u64)),
        fund_member_limit: Some(20),
        funding_goal: None,
        accepted_tokens: None,
    };
    let info = mock_info(&ap_team.clone(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = ExecuteMsg::UpdateAlliancemember {
        address: Addr::unchecked("member-addr"),
        member: AllianceMember {
            name: "Alliance-1".to_string(),
            logo: Some("A1".to_string()),
            website: Some("https://alliance-1.com".to_string()),
        },
    };

    // pleb cannot update the alliance member(only owner should be able to)
    let info = mock_info(&pleb.clone(), &[]);
    let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // real SC owner can update the alliance member now
    let info = mock_info(&ap_team.clone(), &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("method", "update_alliancemember"),
            attr("member_addr", "member-addr"),
        ]
    )
}
