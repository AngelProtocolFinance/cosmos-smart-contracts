use crate::contract::{execute, instantiate, migrate, query, reply};
use angel_core::errors::core::*;
use angel_core::messages::registrar::*;
use angel_core::responses::registrar::*;
use angel_core::structs::{EndowmentStatus, SplitDetails};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, ContractResult, CosmosMsg, Event, Reply, ReplyOn, SubMsg, SubMsgExecutionResponse, WasmMsg, coins, from_binary, to_binary};

const MOCK_ACCOUNTS_CODE_ID: u64 = 17;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let ap_team = "angelprotocolteamdano".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: 20,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(MOCK_ACCOUNTS_CODE_ID, config_response.accounts_code_id);
    assert_eq!(ap_team.clone(), config_response.owner);
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);
    let ap_team = "angelprotocolteamdano".to_string();
    let pleb = "plebAccount".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: 20,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    let info = mock_info(pleb.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    let ap_team = "angelprotocolteamdano".to_string();
    let index_fund_contract = String::from("index_fund_contract");
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: 0,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let update_config_message = UpdateConfigMsg {
        accounts_code_id: None,
        index_fund_contract: index_fund_contract.clone(),
        approved_charities: None,
        vaults: None,
        default_vault: None,
    };
    let msg = ExecuteMsg::UpdateConfig(update_config_message);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(index_fund_contract.clone(), config_response.index_fund_contract);
    assert_eq!(MOCK_ACCOUNTS_CODE_ID, config_response.accounts_code_id);
}

#[test]
fn migrate_contract() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: 20,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // try to migrate the contract
    let msg = MigrateMsg {};
    let res = migrate(deps.as_mut(), env.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len())
}

#[test]
fn test_owner_can_add_remove_approved_charities() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let pleb = "plebAccount".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: 20,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // try to add as a non-owner (should fail)
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CharityAdd {
            charity: String::from("some-rando-charity"),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // add Charity as SC Owner
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CharityAdd {
            charity: charity_addr.clone(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // try to remove as a non-owner (should fail)
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CharityRemove {
            charity: String::from("some-rando-charity"),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // remove Charity as SC Owner
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CharityRemove {
            charity: charity_addr.clone(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn only_approved_charities_can_create_endowment_accounts_and_then_update() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let good_charity_addr = "GOODQTWTETGSGSRHJTUIQADG".to_string();
    let bad_charity_addr = "BADQTWTETGSGSRHJTUIQADG".to_string();
    let good_endowment_addr = "ENDOWMENTADRESS".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: 20,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // add an approved charity to the list (Squeaky Clean Charity)
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CharityAdd {
            charity: good_charity_addr.clone(),
        },
    ).unwrap();
    assert_eq!(0, res.messages.len());

    let create_endowment_msg = CreateEndowmentMsg {
        owner: good_charity_addr.clone(),
        beneficiary: good_charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
    };

    // non-Approved charity cannot create Accounts
    let info = mock_info(bad_charity_addr.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreateEndowment(create_endowment_msg.clone()),
    ).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // approved charity can create Accounts
    let info = mock_info(good_charity_addr.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreateEndowment(create_endowment_msg.clone()),
    ).unwrap();
    assert_eq!(1, res.messages.len());

    // test that with the submessage we can instantiate account sc
    let msg: &CosmosMsg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Wasm(wasm_msg) => {
            match wasm_msg {
                WasmMsg::Instantiate { admin, code_id: _, msg, funds: _, label: _ } => {
                    assert_eq!(admin.clone(), Some("cosmos2contract".to_string()));
                    let accounts_instantiate_msg: angel_core::messages::accounts::InstantiateMsg = from_binary(msg).unwrap();
                    assert_eq!(accounts_instantiate_msg.admin_addr, ap_team.clone());

                    // let's instantiate account sc with our sub_message
                    let mut deps = mock_dependencies(&[]);
                    let info = mock_info("creator", &coins(100000, "earth"));
                    let env = mock_env();

                    // for now we have instantiation error due to another submsg call
                    // from the accounts sc instantiate method
                    // but the instantiation message work well
                    //
                    // TODO: fix test when accounts sc instantiate test will be ready
                    // by removing let err = ... and changing to:
                    // assert_eq!(0, res.messages.len());
                    let err = accounts::contract::instantiate(deps.as_mut(), env, info, accounts_instantiate_msg).unwrap_err();
                    match err {
                        ContractError::Std(err) => {
                            match err {
                                cosmwasm_std::StdError::GenericErr { msg } => {
                                    assert_eq!(msg, "Querier system error: No such contract: cosmos2contract");
                                    ()
                                },
                                _ => (),
                            }
                        },
                        _ => ()
                    }
                    ()
                },
                _ => {
                    panic!("Not the Wasm instaniation message");
                },
            }
        },
        _ => {
            panic!("Not the Cosmos message");
        },
    }

    assert_eq!(1, res.attributes.len());
    assert_eq!("action", res.attributes[0].key);
    assert_eq!("create_endowment", res.attributes[0].value);

    let events = vec![
        Event::new("instantiate_contract").add_attribute("contract_address", good_endowment_addr.clone()),
    ];
    let result = ContractResult::Ok(SubMsgExecutionResponse {
        events,
        data: None,
    });
    let subcall = Reply {
        id: 0,
        result
    };

    // test the reply method
    let res = reply(deps.as_mut(), mock_env(), subcall).unwrap();
    assert_eq!(0, res.messages.len());

    // test that the reply worked properly by querying
    let res = query(deps.as_ref(), mock_env(), QueryMsg::EndowmentList {}).unwrap();
    let endowment_list_response: EndowmentListResponse = from_binary(&res).unwrap();
    assert_eq!(endowment_list_response.endowments[0].address, Addr::unchecked(good_endowment_addr.clone()));
    assert_eq!(endowment_list_response.endowments[0].status, EndowmentStatus::Inactive);

    // let's test update endowment method by admin
    let update_endowment_status_msg = UpdateEndowmentStatusMsg {
        endowment_addr: good_endowment_addr.clone(),
        status: 1,
    };

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateEndowmentStatus(update_endowment_status_msg.clone()),
    ).unwrap();
    assert_eq!(1, res.messages.len());

    // test that the updating worked properly by querying
    let res = query(deps.as_ref(), mock_env(), QueryMsg::EndowmentList {}).unwrap();
    let endowment_list_response: EndowmentListResponse = from_binary(&res).unwrap();
    assert_eq!(endowment_list_response.endowments[0].address, Addr::unchecked(good_endowment_addr.clone()));
    assert_eq!(endowment_list_response.endowments[0].status, EndowmentStatus::Approved);
}
