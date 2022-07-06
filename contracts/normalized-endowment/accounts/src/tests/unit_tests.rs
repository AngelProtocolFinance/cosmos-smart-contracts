use super::mock_querier::mock_dependencies;
use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;
use angel_core::messages::accounts::*;
use angel_core::messages::cw3_multisig::Threshold;
use angel_core::messages::dao_token::CurveType;
use angel_core::responses::accounts::*;
use angel_core::structs::{EndowmentType, Profile, SocialMedialUrls};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, coins, from_binary, from_binary, to_binary, Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfoBase;
use cw_utils::Duration;
use std::vec;

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    assert_eq!(2, res.messages.len()); // no news is good news! :)
}

#[test]
fn test_get_config() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn test_update_endowment_settings() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();

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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        profile: profile,
        cw4_members: vec![],
        kyc_donors_only: false,
        cw3_multisig_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_multisig_max_vote_period: Duration::Time(60),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(1, res.messages.len());

    // update the endowment owner and beneficiary
    let msg = UpdateEndowmentSettingsMsg {
        owner: charity_addr.clone(),
        kyc_donors_only: true,
    };
    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth "));
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
        kyc_donors_only: true,
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
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();

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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(2, res.messages.len());

    // change the owner to some pleb
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
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
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_admin() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();

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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(2, res.messages.len());

    // change the admin to some pleb
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateOwner {
            new_owner: pleb.clone(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(pleb.clone(), value.owner);

    // Original owner should not be able to update the configs now
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: charity_addr.clone(),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth "));
    let env = mock_env();
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_strategy() {
    let mut deps = mock_dependencies(&[]);

    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();

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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(2, res.messages.len());

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

    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

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

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
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
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::RedemptionInProgress {});
}

#[test]
fn test_update_endowment_profile() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();

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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

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
    let info = mock_info(pleb.as_ref(), &[]);
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
    let info = mock_info(charity_addr.as_str(), &[]);
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
            attr("sender", charity_addr.clone())
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
    let info = mock_info(ap_team.as_str(), &[]);
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
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let depositor = Addr::unchecked("depositor");

    // Initialize the Endowment
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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // Update the Endowment status
    let info = mock_info(registrar_contract.as_str(), &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(depositor.as_str(), &coins(donation_amt, "uluna"));
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
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let depositor = Addr::unchecked("depositor");

    // Initialize the Endowment
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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
        beneficiary: charity_addr.clone(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        profile: profile,
        cw4_members: vec![],
        kyc_donors_only: true,
        cw3_multisig_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_multisig_max_vote_period: Duration::Time(60),
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // Update the Endowment status
    let info = mock_info(registrar_contract.as_str(), &[]);
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
        sender: depositor.to_string(),
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
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let registrar_contract = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt".to_string();
    let depositor = Addr::unchecked("depositor");

    // Initialize the Endowment
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
        owner_sc: ap_team.clone(),
        registrar_contract: registrar_contract.clone(),
        owner: charity_addr.clone(),
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
        halo_ust_lp_pair_contract: None,
        user_reserve_token: None,
        user_reserve_ust_lp_pair_contract: None,
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
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // Update the Endowment status
    let info = mock_info(registrar_contract.as_str(), &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        deposit_approved: true,
        withdraw_approved: true,
    });
    let _res = execute(deps.as_mut(), mock_env(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(depositor.as_str(), &coins(donation_amt, "uluna"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    // Try the "Withdraw"
    let info = mock_info(charity_addr.as_str(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        sources: vec![],
        beneficiary: "beneficiary".to_string(),
        asset_info: cw_asset::AssetInfoBase::Native("uluna".to_string()),
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}
