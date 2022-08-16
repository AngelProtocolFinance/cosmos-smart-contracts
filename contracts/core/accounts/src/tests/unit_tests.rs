use super::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;
use angel_core::messages::accounts::*;
use angel_core::responses::accounts::*;
use angel_core::structs::{EndowmentType, GenericBalance, Profile, SocialMedialUrls};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{attr, coins, from_binary, to_binary, Coin, Decimal, OwnedDeps, Uint128};
use cw20::Cw20ReceiveMsg;

use cw_utils::{Duration, Threshold};

const AP_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const CHARITY_ID: u32 = 1;
const CHARITY_ADDR: &str = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v";
const REGISTRAR_CONTRACT: &str = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
const DEPOSITOR: &str = "depositor";

fn create_endowment() -> (
    OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    EndowmentDetailsResponse,
) {
    let mut deps = mock_dependencies(&[]);
    let profile: Profile = Profile {
        name: "Test Endowment".to_string(),
        overview: "Endowment to power an amazing charity".to_string(),
        un_sdg: Some(2),
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
        url: Some("nice-charity.org".to_string()),
        registration_number: Some("1234567".to_string()),
        country_of_origin: Some("GB".to_string()),
        street_address: Some("10 Downing St".to_string()),
        contact_email: Some("admin@nice-charity.org".to_string()),
        social_media_urls: SocialMedialUrls {
            facebook: None,
            twitter: Some("https://twitter.com/nice-charity".to_string()),
            linkedin: None,
        },
        number_of_employees: Some(10),
        average_annual_budget: Some("1 Million Pounds".to_string()),
        annual_revenue: Some("Not enough".to_string()),
        charity_navigator_rating: None,
        endow_type: EndowmentType::Charity,
    };

    let create_endowment_msg = CreateEndowmentMsg {
        owner: CHARITY_ADDR.to_string(),
        beneficiary: CHARITY_ADDR.to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        profile: profile,
        cw4_members: vec![],
        kyc_donors_only: true,
        cw3_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_max_voting_period: Duration::Time(60),
    };

    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(REGISTRAR_CONTRACT, &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    let _ = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreateEndowment(create_endowment_msg),
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let value: EndowmentDetailsResponse = from_binary(&res).unwrap();

    (deps, value)
}

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len()); // no news is good news! :)
}

#[test]
fn test_update_endowment_settings() {
    let (mut deps, endow_details) = create_endowment();

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    // update the endowment "owner" & "kyc_donors_only"
    let msg = UpdateEndowmentSettingsMsg {
        id: CHARITY_ID,
        owner: CHARITY_ADDR.to_string(),
        kyc_donors_only: true,
    };
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateEndowmentSettings(msg),
    )
    .unwrap();
    assert_eq!(1, res.messages.len());

    // Not just anyone can update the Endowment's settings! Only Endowment owner can.
    let msg = UpdateEndowmentSettingsMsg {
        id: CHARITY_ID,
        owner: CHARITY_ADDR.to_string(),
        kyc_donors_only: true,
    };
    let info = mock_info(PLEB, &coins(100000, "earth "));
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
fn test_update_endowment_status() {
    let (mut deps, _endow_details) = create_endowment();

    // Fail to update the endowment status since caller is not `registrar_contract`
    let update_status_msg = UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: false,
        withdraw_approved: true,
    };
    let info = mock_info("non-registrar", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentStatus(update_status_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the endowment status
    let update_status_msg = UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: false,
        withdraw_approved: true,
    };
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentStatus(update_status_msg),
    )
    .unwrap();
    assert_eq!(0, res.attributes.len());

    // Check the update status
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.deposit_approved, false);
    assert_eq!(endow.withdraw_approved, true);
}

#[test]
fn test_change_registrar_contract() {
    let (mut deps, _endow_details) = create_endowment();

    // change the owner to some pleb
    let info = mock_info(REGISTRAR_CONTRACT, &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateRegistrar {
            new_registrar: PLEB.to_string(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(PLEB, value.registrar_contract);

    // Original contract owner should not be able to update the registrar now
    let msg = ExecuteMsg::UpdateRegistrar {
        new_registrar: PLEB.to_string(),
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_admin() {
    let (mut deps, _endow_details) = create_endowment();

    // change the admin to some pleb
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateOwner {
            new_owner: PLEB.to_string(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(PLEB, value.owner);

    // Original owner should not be able to update the configs now
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: CHARITY_ADDR.to_string(),
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_strategy() {
    let (mut deps, endow_details) = create_endowment();

    // sum of the invested strategy components percentages is not equal 100%
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                percentage: Decimal::percent(30),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(60),
            },
        ],
    };

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidStrategyAllocation {});

    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                percentage: Decimal::percent(20),
            },
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                percentage: Decimal::percent(40),
            },
        ],
    };

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StrategyComponentsNotUnique {});

    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
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
    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
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
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_endowment_profile() {
    let (mut deps, endow_details) = create_endowment();
    let msg = UpdateProfileMsg {
        id: CHARITY_ID,
        name: None,
        overview: Some("Test Endowment is for just testing".to_string()),
        un_sdg: Some(1_u8),
        tier: Some(2_u8),
        logo: Some("".to_string()),
        image: Some("".to_string()),
        url: None,
        registration_number: None,
        country_of_origin: None,
        street_address: None,
        contact_email: None,
        facebook: None,
        twitter: None,
        linkedin: None,
        number_of_employees: None,
        average_annual_budget: None,
        annual_revenue: None,
        charity_navigator_rating: None,
        endow_type: None,
    };

    // Not just anyone can update the Endowment's profile! Only Endowment owner or Config owner can.
    let info = mock_info(PLEB, &[]);
    let env = mock_env();
    // This should fail with an error!
    let err = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::UpdateProfile(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Endowment owner can update the profile
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let env = mock_env();
    // This should succeed!
    let res = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::UpdateProfile(msg.clone()),
    )
    .unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update_profile"),
            attr("sender", CHARITY_ADDR.to_string())
        ]
    );
    assert_eq!(res.messages.len(), 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetProfile { id: CHARITY_ID },
    )
    .unwrap();
    let value: ProfileResponse = from_binary(&res).unwrap();
    assert_eq!(
        value.overview,
        "Test Endowment is for just testing".to_string()
    );
    assert_eq!(value.un_sdg, Some(2));
    assert_eq!(value.tier, Some(3));

    // Config owner can update certain profile
    let info = mock_info(AP_TEAM, &[]);
    let env = mock_env();
    // This should succeed!
    let _res = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::UpdateProfile(msg.clone()),
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetProfile { id: CHARITY_ID },
    )
    .unwrap();
    let value: ProfileResponse = from_binary(&res).unwrap();
    assert_eq!(value.un_sdg.unwrap(), 1);
    assert_eq!(value.tier.unwrap(), 2);
}

#[test]
fn test_donate() {
    let (mut deps, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    assert_eq!(res.attributes.len(), 4);

    // Check the "STATE" for "transactions" field
    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(state.donations_received.u128(), donation_amt);
}

#[test]
fn test_deposit_cw20() {
    let (mut deps, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

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
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 4);
}

#[test]
fn test_withdraw() {
    let (mut deps, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    // Try the "Withdraw"
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        sources: vec![],
        beneficiary: "beneficiary".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_withdraw_liquid() {
    let (mut deps, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    // Try the "WithdrawLiquid"
    // Fails since the amount is too big
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::WithdrawLiquid {
        id: CHARITY_ID,
        beneficiary: "beneficiary".to_string(),
        assets: GenericBalance {
            native: vec![Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(1000_u128),
            }],
            cw20: vec![],
        },
    };
    let err = execute(deps.as_mut(), mock_env(), info, withdraw_liquid_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to withdraw liquid amount
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::WithdrawLiquid {
        id: CHARITY_ID,
        beneficiary: "beneficiary".to_string(),
        assets: GenericBalance {
            native: vec![Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(10_u128),
            }],
            cw20: vec![],
        },
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_liquid_msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn test_vault_receipt() {
    let (mut deps, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try to run "vault_receipt"
    // Fails since no funds
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultReceipt { id: CHARITY_ID },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Success, but no messages since "config.pending_redemptions == None"
    let info = mock_info(
        "vault",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultReceipt { id: CHARITY_ID },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // First, update the "config.pending_redemptions"
    let msg = ExecuteMsg::UpdateStrategies {
        id: CHARITY_ID,
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
    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(2, endow.strategies.len());

    // Success, check if the "config.redemptions" is decreased
    let info = mock_info(
        "vault",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::VaultReceipt { id: CHARITY_ID },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(0, endow.pending_redemptions);
}

#[test]
fn test_close_endowment() {
    let (mut deps, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        id: CHARITY_ID,
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // confirm we have true for deposit and withdraw
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.withdraw_approved, true);
    assert_eq!(endow.deposit_approved, true);

    // Fails since non-registrar calls the entry
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Success
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: None,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Check the config & state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.withdraw_approved, true);
    assert_eq!(endow.deposit_approved, true);
    assert_eq!(endow.pending_redemptions, 0);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.closing_endowment, true);
    assert_eq!(state.closing_beneficiary, "");
}
