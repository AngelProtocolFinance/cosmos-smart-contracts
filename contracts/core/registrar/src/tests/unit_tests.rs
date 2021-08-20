use crate::contract::{execute, instantiate, migrate, query};
use angel_core::errors::core::*;
use angel_core::messages::registrar::*;
use angel_core::responses::registrar::*;
use angel_core::structs::TaxParameters;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Decimal};

const MOCK_ACCOUNTS_CODE_ID: u64 = 17;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let ap_team = "angelprotocolteamdano".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
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
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    let info = mock_info("ill-wisher", &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg);
    assert_eq!(ContractError::Unauthorized {}, _res.unwrap_err());

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
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
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
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
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
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
fn only_approved_charities_can_create_endowment_accounts() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let good_charity_addr = "GOODQTWTETGSGSRHJTUIQADG".to_string();
    let bad_charity_addr = "BADQTWTETGSGSRHJTUIQADG".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
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
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let create_endowment_msg = CreateEndowmentMsg {
        owner: good_charity_addr.clone(),
        beneficiary: good_charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: None,
    };

    // non-Approved charity cannot create Accounts
    let info = mock_info(bad_charity_addr.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreateEndowment(create_endowment_msg.clone()),
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    // approved charity can create Accounts
    let info = mock_info(good_charity_addr.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreateEndowment(create_endowment_msg.clone()),
    )
    .unwrap();
    assert_eq!(1, res.messages.len());
}
