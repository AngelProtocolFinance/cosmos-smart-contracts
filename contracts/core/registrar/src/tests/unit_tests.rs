use crate::contract::{execute, instantiate, migrate, query};
use angel_core::error::*;
use angel_core::registrar_msg::*;
use angel_core::registrar_rsp::*;
use angel_core::structs::TaxParameters;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Decimal};

const MOCK_ACCOUNTS_CODE_ID: u64 = 17;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let ap_team = "angelprotocolteamdano".to_string();
    let msg = InstantiateMsg {
        approved_coins: Some(vec![]),
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
    };
    let info = mock_info("creator", &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(MOCK_ACCOUNTS_CODE_ID, config_response.accounts_code_id);
    assert_eq!("creator", config_response.owner);
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);
    let ap_team = "angelprotocolteamdano".to_string();
    let msg = InstantiateMsg {
        approved_coins: Some(vec![]),
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        taxes: TaxParameters {
            exit_tax: Decimal::percent(50),
            max_tax: Decimal::one(),
            min_tax: Decimal::zero(),
            step: Decimal::percent(5),
        },
    };
    let info = mock_info("creator", &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("ill-wisher", &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg);
    assert_eq!(ContractError::Unauthorized {}, _res.unwrap_err());

    let info = mock_info("creator", &coins(1000, "earth"));
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
        approved_coins: Some(vec![]),
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
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
