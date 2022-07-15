use super::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;
use angel_core::messages::accounts::*;
use angel_core::messages::dao_token::CurveType;
use angel_core::responses::accounts::*;
use angel_core::structs::{EndowmentType, Profile, SocialMedialUrls};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{attr, coins, from_binary, to_binary, Addr, Coin, Decimal, OwnedDeps, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfoBase;
use cw_utils::{Duration, Threshold};
use std::vec;

const AP_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const CHARITY_ADDR: &str = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v";
const REGISTRAR_CONTRACT: &str = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
const DEPOSITOR: &str = "depositor";

fn create_endowment() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut deps = mock_dependencies(&[]);
    let profile: Profile = Profile {
        name: "Test Endowment".to_string(),
        overview: "Endowment to power an amazing charity".to_string(),
        un_sdg: None,
        tier: None,
        logo: None,
        image: None,
        url: None,
        registration_number: None,
        country_of_origin: None,
        street_address: None,
        contact_email: None,
        social_media_urls: SocialMedialUrls {
            facebook: None,
            twitter: None,
            linkedin: None,
        },
        number_of_employees: None,
        average_annual_budget: None,
        annual_revenue: None,
        charity_navigator_rating: None,
        endow_type: EndowmentType::Charity,
    };

    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
        owner: CHARITY_ADDR.to_string(),
        name: "Endowment".to_string(),
        description: "New Endowment Creation".to_string(),
        split_max: Decimal::one(),
        split_min: Decimal::one(),
        split_default: Decimal::one(),
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        dao: true,
        dao_setup_option: DaoSetupOption::SetupBondCurveToken(CurveType::Constant {
            value: Uint128::zero(),
            scale: 2u32,
        }),
        donation_match: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
        donation_match_setup_option: 2,
        reserve_token: None,
        reserve_token_lp_contract: None,
        settings_controller: None,
        parent: None,
        withdraw_before_maturity: false,
        maturity_time: Some(1000_u64),
        profile: profile,
        cw4_members: vec![],
        kyc_donors_only: true,
        cw3_multisig_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_multisig_max_vote_period: Duration::Time(60),
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let env = mock_env();
    let _ = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    deps
}

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let profile: Profile = Profile {
        name: "Test Endowment".to_string(),
        overview: "Endowment to power an amazing charity".to_string(),
        un_sdg: None,
        tier: None,
        logo: None,
        image: None,
        url: None,
        registration_number: None,
        country_of_origin: None,
        street_address: None,
        contact_email: None,
        social_media_urls: SocialMedialUrls {
            facebook: None,
            twitter: None,
            linkedin: None,
        },
        number_of_employees: None,
        average_annual_budget: None,
        annual_revenue: None,
        charity_navigator_rating: None,
        endow_type: EndowmentType::Charity,
    };

    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
        owner: CHARITY_ADDR.to_string(),
        name: "Endowment".to_string(),
        description: "New Endowment Creation".to_string(),
        split_max: Decimal::one(),
        split_min: Decimal::one(),
        split_default: Decimal::one(),
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        dao: true,
        dao_setup_option: DaoSetupOption::SetupBondCurveToken(CurveType::Constant {
            value: Uint128::zero(),
            scale: 2u32,
        }),
        donation_match: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
        donation_match_setup_option: 2,
        reserve_token: None,
        reserve_token_lp_contract: None,
        settings_controller: None,
        parent: None,
        withdraw_before_maturity: false,
        maturity_time: Some(1000_u64),
        profile: profile,
        cw4_members: vec![],
        kyc_donors_only: true,
        cw3_multisig_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_multisig_max_vote_period: Duration::Time(60),
    };
    let info = mock_info("creator", &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(2, res.messages.len());
}

#[test]
fn test_update_endowment_settings() {
    let mut deps = create_endowment();

    // update the endowment owner and beneficiary
    let msg = UpdateEndowmentSettingsMsg {
        owner: Some(CHARITY_ADDR.to_string()),
        whitelisted_beneficiaries: None,
        whitelisted_contributors: None,
        name: None,
        description: None,
        withdraw_before_maturity: None,
        maturity_time: None,
        strategies: None,
        locked_endowment_configs: None,
        rebalance: None,
        kyc_donors_only: true,
        maturity_whitelist: None,
    };
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth "));
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
        owner: Some(CHARITY_ADDR.to_string()),
        whitelisted_beneficiaries: None,
        whitelisted_contributors: None,
        name: None,
        description: None,
        withdraw_before_maturity: None,
        maturity_time: None,
        strategies: None,
        locked_endowment_configs: None,
        rebalance: None,
        kyc_donors_only: true,
        maturity_whitelist: None,
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
    let mut deps = create_endowment();

    // Fail to update the endowment status since caller is not `registrar_contract`
    let update_status_msg = UpdateEndowmentStatusMsg {
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
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let endow: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(endow.deposit_approved, false);
    assert_eq!(endow.withdraw_approved, true);
}

#[test]
fn test_change_registrar_contract() {
    let mut deps = create_endowment();

    // change the registrar to some pleb
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
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
    let info = mock_info(PLEB, &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_admin() {
    let mut deps = create_endowment();

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
    let mut deps = create_endowment();

    // sum of the invested strategy components percentages is not equal 100%
    let msg = ExecuteMsg::UpdateStrategies {
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

    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidStrategyAllocation {});

    let msg = ExecuteMsg::UpdateStrategies {
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

    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StrategyComponentsNotUnique {});

    let msg = ExecuteMsg::UpdateStrategies {
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
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());

    let msg = ExecuteMsg::UpdateStrategies {
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
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::RedemptionInProgress {});
}

#[test]
fn test_update_endowment_profile() {
    let mut deps = create_endowment();
    let msg = UpdateProfileMsg {
        name: None,
        overview: Some("Test Endowment is for just testing".to_string()),
        un_sdg: Some(1_u64),
        tier: Some(2_u64),
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
    let info = mock_info(CHARITY_ADDR, &[]);
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

    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProfile {}).unwrap();
    let value: ProfileResponse = from_binary(&res).unwrap();
    assert_eq!(
        value.overview,
        "Test Endowment is for just testing".to_string()
    );
    assert_eq!(value.un_sdg, None);
    assert_eq!(value.tier, None);

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

    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProfile {}).unwrap();
    let value: ProfileResponse = from_binary(&res).unwrap();
    assert_eq!(value.un_sdg.unwrap(), 1);
    assert_eq!(value.tier.unwrap(), 2);
}

#[test]
fn test_donate() {
    let mut deps = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    assert_eq!(res.attributes.len(), 3);

    // Check the "STATE" for "transactions" field
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(state.donations_received.u128(), donation_amt);

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetTxRecords {
            sender: None,
            recipient: None,
            asset_info: AssetInfoBase::Native("uluna".to_string()),
        },
    )
    .unwrap();
    let txs_response: TxRecordsResponse = from_binary(&query_res).unwrap();
    assert_eq!(txs_response.txs.len(), 1);
}

#[test]
fn test_deposit_cw20() {
    let mut deps = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(donation_amt),
        msg: to_binary(&deposit_msg).unwrap(),
    });
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes.len(), 3);

    // Check the "STATE" for "transactions" field
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(state.donations_received.u128(), donation_amt);

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetTxRecords {
            sender: None,
            recipient: None,
            asset_info: AssetInfoBase::Cw20(Addr::unchecked("test-cw20")),
        },
    )
    .unwrap();
    let txs_response: TxRecordsResponse = from_binary(&query_res).unwrap();
    assert_eq!(txs_response.txs.len(), 1);
}

#[test]
fn test_withdraw() {
    let mut deps = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    // Try the "Withdraw"
    let info = mock_info(CHARITY_ADDR, &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        sources: vec![],
        beneficiary: "beneficiary".to_string(),
        asset_info: cw_asset::AssetInfoBase::Native("uluna".to_string()),
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_withdraw_liquid() {
    let mut deps = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    // Try the "WithdrawLiquid"
    // Fails since the amount is too big
    let info = mock_info(CHARITY_ADDR, &[]);
    let withdraw_liquid_msg = ExecuteMsg::WithdrawLiquid {
        liquid_amount: Uint128::from(200_u128),
        beneficiary: "beneficiary".to_string(),
        asset_info: AssetInfoBase::Native("uluna".to_string()),
    };
    let err = execute(deps.as_mut(), mock_env(), info, withdraw_liquid_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to withdraw liquid amount
    let info = mock_info(CHARITY_ADDR, &[]);
    let withdraw_liquid_msg = ExecuteMsg::WithdrawLiquid {
        liquid_amount: Uint128::from(100_u128),
        beneficiary: "beneficiary".to_string(),
        asset_info: AssetInfoBase::Native("uluna".to_string()),
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_liquid_msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn test_vault_receipt() {
    let mut deps = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try to run "vault_receipt"
    // Fails since no funds
    let info = mock_info("anyone", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::VaultReceipt {}).unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Success, but no messages since "config.pending_redemptions == None"
    let info = mock_info(
        "vault",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::VaultReceipt {}).unwrap();
    assert_eq!(0, res.messages.len());

    // First, update the "config.pending_redemptions"
    let msg = ExecuteMsg::UpdateStrategies {
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
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("1", config.pending_redemptions);

    // Success, check if the "config.redemptions" is decreased
    let info = mock_info(
        "vault",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::VaultReceipt {}).unwrap();
    assert_eq!(2, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("", config.pending_redemptions);
}

#[test]
fn test_close_endowment() {
    let mut deps = create_endowment();

    // Update the Endowment status
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try to close the endowment

    // Fails since non-registrar calls the entry
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CloseEndowment { beneficiary: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Success
    let info = mock_info(REGISTRAR_CONTRACT, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CloseEndowment { beneficiary: None },
    )
    .unwrap();
    assert_eq!(1, res.messages.len());

    // Check the config & state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config.pending_redemptions, "1");

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.closing_endowment, true);
    assert_eq!(state.closing_beneficiary, "");
}
