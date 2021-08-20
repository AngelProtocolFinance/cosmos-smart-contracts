use crate::contract::{execute, instantiate, migrate, query};
use angel_core::errors::core::*;
use angel_core::messages::accounts::*;
use angel_core::responses::accounts::*;
use angel_core::structs::{GenericBalance, SplitDetails, StrategyComponent};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, coins, from_binary, Addr, Decimal, Uint128};
use cw20::{Balance, Cw20CoinVerified};

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info("creator", &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len()); // no news is good news! :)
}

#[test]
fn test_get_config() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_update_endowment_settings() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // update the endowment owner and beneficiary
    let msg = UpdateEndowmentSettingsMsg {
        owner: charity_addr.clone(),
        beneficiary: pleb.clone(),
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth "));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateEndowmentSettings(msg),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Not just anyone can update the Endowment's settings! Only Endowment owner can.
    let msg = UpdateEndowmentSettingsMsg {
        owner: charity_addr.clone(),
        beneficiary: pleb.clone(),
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::UpdateEndowmentSettings(msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_registrar_contract() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // change the owner to some pleb
    let info = mock_info(registrar_contract.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateRegistrar {
            new_registrar: pleb.clone(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(pleb.clone(), value.registrar_contract);

    // Original contract owner should not be able to update the registrar now
    let msg = ExecuteMsg::UpdateRegistrar {
        new_registrar: pleb.clone(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_admin() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // change the admin to some pleb
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateAdmin {
            new_admin: pleb.clone(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(pleb.clone(), value.admin_addr);

    // Original owner should not be able to update the configs now
    let msg = ExecuteMsg::UpdateAdmin {
        new_admin: charity_addr.clone(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn migrate_contract() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let _pleb = "plebAccount".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
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
fn test_update_strategy() {
    let mut deps = mock_dependencies(&[]);

    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

    let instantiate_msg = InstantiateMsg {
        admin_addr: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        index_fund_contract: index_fund_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        split_to_liquid: SplitDetails::default(),
    };

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    assert_eq!(0, res.messages.len());

    // sum of the invested strategy components percentages is not equal 100%
    let msg = ExecuteMsg::UpdateStrategy {
        strategies: vec![
            StrategyComponent {
                vault: Addr::unchecked("cash_strategy_component_addr"),
                locked_percentage: Decimal::percent(20),
                liquid_percentage: Decimal::percent(20),
            },
            StrategyComponent {
                vault: Addr::unchecked("tech_strategy_component_addr"),
                locked_percentage: Decimal::percent(60),
                liquid_percentage: Decimal::percent(60),
            },
        ],
    };

    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(err, ContractError::InvalidStrategyAllocation {});
    let msg = ExecuteMsg::UpdateStrategy {
        strategies: vec![
            StrategyComponent {
                vault: Addr::unchecked("cash_strategy_component_addr"),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
            StrategyComponent {
                vault: Addr::unchecked("tech_strategy_component_addr"),
                locked_percentage: Decimal::percent(20),
                liquid_percentage: Decimal::percent(20),
            },
            StrategyComponent {
                vault: Addr::unchecked("cash_strategy_component_addr"),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
        ],
    };

    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(err, ContractError::StrategyComponentsNotUnique {});
    let msg = ExecuteMsg::UpdateStrategy {
        strategies: vec![
            StrategyComponent {
                vault: Addr::unchecked("cash_strategy_component_addr"),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
            StrategyComponent {
                vault: Addr::unchecked("tech_strategy_component_addr"),
                locked_percentage: Decimal::percent(60),
                liquid_percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = ExecuteMsg::UpdateStrategy {
        strategies: vec![
            StrategyComponent {
                vault: Addr::unchecked("cash_strategy_component_addr"),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
            StrategyComponent {
                vault: Addr::unchecked("tech_strategy_component_addr"),
                locked_percentage: Decimal::percent(60),
                liquid_percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}
