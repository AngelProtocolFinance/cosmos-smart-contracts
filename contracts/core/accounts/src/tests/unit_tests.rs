use super::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::contract::{execute, instantiate, migrate, query};
use angel_core::errors::core::*;
use angel_core::messages::accounts::*;
use angel_core::responses::accounts::*;
use angel_core::structs::{
    AccountType, Beneficiary, Categories, EndowmentBalanceResponse, EndowmentType,
    StrategyComponent, SwapOperation,
};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, coins, from_binary, to_binary, Addr, Coin, Decimal, Env, OwnedDeps, StdError, Uint128,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetUnchecked};
use cw_utils::Threshold;

const AP_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const CHARITY_ID: u32 = 1;
const CHARITY_ADDR: &str = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v";
const REGISTRAR_CONTRACT: &str = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
const DEPOSITOR: &str = "depositor";

fn create_endowment() -> (
    OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    Env,
    String,
    EndowmentDetailsResponse,
) {
    let mut deps = mock_dependencies(&[]);
    let create_endowment_msg = CreateEndowmentMsg {
        owner: CHARITY_ADDR.to_string(),
        name: "Test Endowment".to_string(),
        endow_type: EndowmentType::Normal,
        categories: Categories {
            sdgs: vec![2],
            general: vec![],
        },
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
        maturity_time: None,
        cw4_members: vec![],
        kyc_donors_only: true,
        cw3_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_max_voting_period: 60,
        proposal_link: None,
        referral_id: None,
    };

    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let env = mock_env();
    let acct_contract = env.contract.address.to_string();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    let _ = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::CreateEndowment(create_endowment_msg),
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let value: EndowmentDetailsResponse = from_binary(&res).unwrap();

    (deps, env, acct_contract, value)
}

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len()); // no news is good news! :)
}

#[test]
fn test_update_endowment_settings() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    // update the endowment "owner" & "kyc_donors_only"
    let msg = UpdateEndowmentSettingsMsg {
        id: CHARITY_ID,
        owner: Some(CHARITY_ADDR.to_string()),
        kyc_donors_only: Some(false),
        name: Some("Test Endowment".to_string()),
        endow_type: Some("normal".to_string()),
        categories: Some(Categories {
            sdgs: vec![2],
            general: vec![],
        }),
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
    };
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
        id: CHARITY_ID,
        owner: Some(CHARITY_ADDR.to_string()),
        kyc_donors_only: Some(false),
        name: Some("Test Endowment name goes here".to_string()),
        endow_type: Some("normal".to_string()),
        categories: Some(Categories {
            sdgs: vec![2],
            general: vec![],
        }),
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
    };
    let info = mock_info(PLEB, &coins(100000, "earth "));
    // This should fail with an error!
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::UpdateEndowmentSettings(msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_endowment_status() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // Fail to update the endowment status since caller is not config owner
    let update_endowment_status_msg = UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    };
    let info = mock_info("non-registrar", &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::UpdateEndowmentStatus(update_endowment_status_msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let info = mock_info(AP_TEAM, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::UpdateEndowmentStatus(update_endowment_status_msg),
    )
    .unwrap();
    assert_eq!(0, res.attributes.len());

    // Check the update status
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.deposit_approved, true);
    assert_eq!(endow.withdraw_approved, true);
}

#[test]
fn test_change_configs_() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // change the registrar contract to some pleb address
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateConfig {
            new_owner: None,
            new_registrar: Some(PLEB.to_string()),
            max_general_category_id: Some(2),
            ibc_controller: None,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(PLEB, value.registrar_contract);

    // Check that the "PLEB" registrar contract should not be able to affect/update the configs
    let msg = ExecuteMsg::UpdateConfig {
        new_owner: None,
        new_registrar: Some(PLEB.to_string()),
        max_general_category_id: Some(100),
        ibc_controller: None,
    };
    let info = mock_info(PLEB, &coins(100000, "earth "));
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_admin() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // change the admin to some pleb
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateConfig {
            new_owner: Some(PLEB.to_string()),
            new_registrar: None,
            max_general_category_id: None,
            ibc_controller: None,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(PLEB, value.owner);

    // Original owner should not be able to update the configs now
    let msg = ExecuteMsg::UpdateConfig {
        new_owner: Some(CHARITY_ADDR.to_string()),
        new_registrar: None,
        max_general_category_id: None,
        ibc_controller: None,
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth "));
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_strategy() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // sum of the invested strategy components percentages is over 100%
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                percentage: Decimal::percent(30),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(50),
            },
        ],
    };

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // sum of the invested strategy components percentages is over 100%
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "vault".to_string(),
                percentage: Decimal::percent(30),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(80),
            },
        ],
    };

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidStrategyAllocation {});

    // duplicated vaults passed
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "vault".to_string(),
                percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(20),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(40),
            },
        ],
    };

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StrategyComponentsNotUnique {});

    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "vault".to_string(),
                percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(PLEB, &coins(100000, "earth"));
    let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the strategies
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        strategies: vec![Strategy {
            vault: "cash_strategy_component_addr".to_string(),
            percentage: Decimal::percent(100),
        }],
    };
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Check the strategies
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endowment: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endowment.strategies.locked.len(), 2);
    assert_eq!(endowment.strategies.liquid.len(), 1);
    assert_eq!(
        endowment.strategies.liquid,
        vec![StrategyComponent {
            vault: "cash_strategy_component_addr".to_string(),
            percentage: Decimal::percent(100),
        }]
    );
}

#[test]
fn test_donate() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit" w/o "Auto Invest" turned on. No Vault deposits should take place.
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "ujuno"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    assert_eq!(0, res.messages.len());

    // Check the "STATE" for "transactions" field
    let query_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(state.donations_received.locked.u128(), donation_amt / 2);
    assert_eq!(state.donations_received.liquid.u128(), donation_amt / 2);

    // Update the Endowment settings to enable onward vault deposits
    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let _vaults_res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateStrategies {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            strategies: [
                Strategy {
                    vault: "tech_strategy_component_addr".to_string(),
                    percentage: Decimal::percent(40),
                },
                Strategy {
                    vault: "vault".to_string(),
                    percentage: Decimal::percent(40),
                },
            ]
            .to_vec(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(2, endow.strategies.locked.len());

    // Cannot deposit several tokens at once.
    let info = mock_info(
        DEPOSITOR,
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
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let err = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Try the "Deposit" w/ "Auto Invest" turned on. Two Vault deposits should now take place.
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "ujuno"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_deposit_cw20() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(donation_amt),
        msg: to_binary(&deposit_msg).unwrap(),
    });
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(0, res.messages.len());
}

#[test]
fn test_withdraw() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "ujuno"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    // Try the "Withdraw"
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary: "beneficiary".to_string(),
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native("ujuno".to_string()),
            amount: Uint128::from(100_u128),
        }],
    };
    let res = execute(deps.as_mut(), env.clone(), info, withdraw_msg).unwrap();
    assert_eq!(2, res.messages.len());

    // Try to "withdraw" cw20 tokens
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary: "beneficiary".to_string(),
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(100_u128),
        }],
    };
    let err = execute(deps.as_mut(), env.clone(), info, withdraw_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Deposit cw20 token first & withdraw
    let donation_amt = 200_u128;
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(donation_amt),
        msg: to_binary(&deposit_msg).unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary: "beneficiary".to_string(),
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(100_u128),
        }],
    };
    let res = execute(deps.as_mut(), env.clone(), info, withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_withdraw_liquid() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "ujuno"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    // Try the "WithdrawLiquid"
    // Fails since the amount is too big
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary: "beneficiary".to_string(),
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native("ujuno".to_string()),
            amount: Uint128::from(1000_u128),
        }],
    };
    let err = execute(deps.as_mut(), env.clone(), info, withdraw_liquid_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to withdraw liquid amount
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary: "beneficiary".to_string(),
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native("ujuno".to_string()),
            amount: Uint128::from(10_u128),
        }],
    };
    let res = execute(deps.as_mut(), env.clone(), info, withdraw_liquid_msg).unwrap();
    assert_eq!(2, res.messages.len());
}

#[test]
fn test_vault_receipt() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Update the Endowment status to APPROVED
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try to run "vault_receipt"
    // Fails since no funds
    let info = mock_info("vault", &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::VaultReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Success, but no messages since "config.pending_redemptions == None"
    let info = mock_info(
        "vault",
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::VaultReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Should fail if we try to assign a vault with an acct_type that is different from the Endow acct_type
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(), // THIS IS A LIQUID ACCOUNT VAULT!
                percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();

    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        strategies: vec![
            Strategy {
                vault: "vault".to_string(),
                percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(2, endow.strategies.locked.len());

    // Success, check if the "config.redemptions" is decreased
    let info = mock_info(
        "vault",
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::VaultReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(0, endow.pending_redemptions);

    // Same logic applies to the cw20 token vault_receipt
    let info = mock_info("test-cw20", &[]);
    let msg = ReceiveMsg::VaultReceipt {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "vault".to_string(),
            msg: to_binary(&msg).unwrap(),
            amount: Uint128::from(100_u128),
        }),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_close_endowment() {
    let (mut deps, env, acct_contract, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // confirm we have true for deposit and withdraw
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.withdraw_approved, true);
    assert_eq!(endow.deposit_approved, true);

    // Fails since external address / non-accounts contract calls the entry
    let info = mock_info(AP_TEAM, &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: Beneficiary::Wallet {
                address: CHARITY_ADDR.to_string(),
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Success
    let info = mock_info(&acct_contract, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: Beneficiary::Wallet {
                address: CHARITY_ADDR.to_string(),
            },
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Check the config & state
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.withdraw_approved, true);
    assert_eq!(endow.deposit_approved, false);
    assert_eq!(endow.pending_redemptions, 0);

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.closing_endowment, true);
    assert_eq!(
        state.closing_beneficiary,
        Some(Beneficiary::Wallet {
            address: CHARITY_ADDR.to_string()
        })
    );
}

#[test]
fn test_swap_token() {
    let (mut deps, _env, _acct_contract, _endow_details) = create_endowment();

    // Should deposit some funds before swap operation
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &coins(1000000000_u128, "ujuno")),
        ExecuteMsg::Deposit(DepositMsg {
            id: CHARITY_ID,
            locked_percentage: Decimal::percent(100),
            liquid_percentage: Decimal::percent(0),
        }),
    )
    .unwrap();

    // Fail to swap token since non-authorized call
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::from(1000000_u128),
            operations: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to swap token since no operations
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::from(1000000_u128),
            operations: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Fail to swap token since no amount
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::zero(),
            operations: vec![SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Succeed to swap token
    let info = mock_info(CHARITY_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::from(1000000_u128),
            operations: vec![SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
            }],
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_swap_receipt() {
    let (mut deps, _env, _acct_contract, _endow_details) = create_endowment();

    // Fail to swap receipt since non-authorized call
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            final_asset: Asset::native("ujuno", 1000000_u128),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to swap receipt & update the state
    let info = mock_info("swaps_router_addr", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            final_asset: Asset::native("ujuno", 1000000_u128),
        },
    )
    .unwrap();

    let info = mock_info("swaps_router_addr", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Liquid,
            final_asset: Asset::native("ujuno", 2000000_u128),
        },
    )
    .unwrap();

    // Check the result(state.balances)
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::TokenAmount {
            id: CHARITY_ID,
            asset_info: AssetInfo::Native("ujuno".to_string()),
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(1000000_u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::TokenAmount {
            id: CHARITY_ID,
            asset_info: AssetInfo::Native("ujuno".to_string()),
            acct_type: AccountType::Liquid,
        },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(2000000_u128));

    // Same logic applies to "SwapReceipt" of cw20 tokens
    let info = mock_info("test-cw20", &[]);
    let msg = ReceiveMsg::SwapReceipt {
        id: CHARITY_ID,
        final_asset: Asset {
            info: AssetInfoBase::Cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(1000000_u128),
        },
        acct_type: AccountType::Liquid,
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "swaps_router_addr".to_string(),
            msg: to_binary(&msg).unwrap(),
            amount: Uint128::from(1000000_u128),
        }),
    )
    .unwrap();

    let info = mock_info("test-cw20", &[]);
    let msg = ReceiveMsg::SwapReceipt {
        id: CHARITY_ID,
        final_asset: Asset {
            info: AssetInfoBase::Cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(2000000_u128),
        },
        acct_type: AccountType::Locked,
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "swaps_router_addr".to_string(),
            msg: to_binary(&msg).unwrap(),
            amount: Uint128::from(1000000_u128),
        }),
    )
    .unwrap();

    // Check the result
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::TokenAmount {
            id: CHARITY_ID,
            asset_info: AssetInfo::cw20(Addr::unchecked("test-cw20")),
            acct_type: AccountType::Liquid,
        },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(1000000_u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::TokenAmount {
            id: CHARITY_ID,
            asset_info: AssetInfo::cw20(Addr::unchecked("test-cw20")),
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(2000000_u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance { id: CHARITY_ID },
    )
    .unwrap();
    let bal: EndowmentBalanceResponse = from_binary(&res).unwrap();
    assert_eq!(bal.tokens_on_hand.liquid.cw20.len(), 1);
}

#[test]
fn test_vaults_invest() {
    let (mut deps, _env, _acct_contract, _endow_details) = create_endowment();

    // Fail to invest to vaults since no endowment owner calls
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsInvest {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to invest to vaults since vaults are empty
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsInvest {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Fail to invest to vaults since acct_type does not match
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsInvest {
            id: CHARITY_ID,
            acct_type: AccountType::Liquid,
            vaults: vec![(
                "vault".to_string(),
                Asset::native("input-denom", 1000000_u128),
            )],
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Vault and Endowment AccountTypes do not match".to_string(),
        })
    );

    // Fail to invest to vaults since insufficient funds
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsInvest {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![(
                "vault".to_string(),
                Asset::native("input-denom", 1000000_u128),
            )],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Finally, succeed to do "vaults_invest"
    // first, need to update the "state.balances"
    let info = mock_info("swaps_router_addr", &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            final_asset: Asset::native("input-denom", 1000000_u128),
        },
    )
    .unwrap();

    // succeed to "vaults_invest"
    let info = mock_info(CHARITY_ADDR, &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsInvest {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![(
                "vault".to_string(),
                Asset::native("input-denom", 300000_u128),
            )],
        },
    )
    .unwrap();

    // Check the result(state.balances)
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::TokenAmount {
            id: CHARITY_ID,
            asset_info: AssetInfo::Native("input-denom".to_string()),
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(1000000_u128 - 300000_u128));
}

#[test]
fn test_vaults_redeem() {
    let (mut deps, _env, _acct_contract, _endow_details) = create_endowment();

    // Fail to redeem vaults since no endowment owner calls
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsRedeem {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![("vault".to_string(), Uint128::from(1000000_u128))],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to redeem vaults since vaults are empty
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsRedeem {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Fail to invest to vaults since acct_type does not match
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsRedeem {
            id: CHARITY_ID,
            acct_type: AccountType::Liquid,
            vaults: vec![("vault".to_string(), Uint128::from(1000000_u128))],
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Vault and Endowment AccountTypes do not match".to_string(),
        })
    );

    // Fail to invest to vaults since insufficient funds
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsRedeem {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![("vault".to_string(), Uint128::from(2000000_u128))],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::BalanceTooSmall {});

    // Succeed to invest to vaults
    let info = mock_info(CHARITY_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultsRedeem {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            vaults: vec![("vault".to_string(), Uint128::from(100000_u128))],
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_distribute_to_beneficiary() {
    let (mut deps, _env, _acct_contract, _endow_details) = create_endowment();

    // Only contract itself can call this entry. In other words, it is internal entry.
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::DistributeToBeneficiary { id: CHARITY_ID },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Since "state.closing_beneficiary" is None, it just defaults the "state.balances".
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::DistributeToBeneficiary { id: CHARITY_ID },
    )
    .unwrap();

    // Set the "closing_beneficiary" for the tests
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: Beneficiary::Wallet {
                address: CHARITY_ADDR.to_string(),
            },
        },
    )
    .unwrap();
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.closing_beneficiary,
        Some(Beneficiary::Wallet {
            address: CHARITY_ADDR.to_string()
        })
    );

    // Succeed to distribute to "wallet beneficiary"
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::DistributeToBeneficiary { id: CHARITY_ID },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_query_endowment_list() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Check if the query returns correct list
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentList {
            proposal_link: None,
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    let endow_lists: EndowmentListResponse = from_binary(&res).unwrap();
    assert_eq!(endow_lists.endowments.len(), 1);

    // Check the result of conidtion queries
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentList {
            proposal_link: Some(1),
            start_after: Some(100),
            limit: None,
        },
    )
    .unwrap();
    let endow_lists: EndowmentListResponse = from_binary(&res).unwrap();
    assert_eq!(endow_lists.endowments.len(), 0);
}

#[test]
fn test_migrate() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    let err = migrate(deps.as_mut(), env, MigrateMsg {}).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        })
    );
}
