use std::vec;

use super::mock_querier::mock_dependencies;
use crate::contract::{execute, instantiate, migrate, query};
use angel_core::errors::core::*;
use angel_core::messages::accounts::*;
use angel_core::responses::accounts::*;
use angel_core::structs::{EndowmentFee, EndowmentType, Profile, SocialMedialUrls};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, coins, from_binary, Addr, Decimal, Fraction};

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
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
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
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
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // update the endowment owner and beneficiary
    let msg = UpdateEndowmentSettingsMsg {
        owner: Some(charity_addr.clone()),
        whitelisted_beneficiaries: None,
        whitelisted_contributors: None,
        name: Some("Better Name".to_string()),
        description: Some("A better,description to satisfy donor curiosities".to_string()),
        withdraw_before_maturity: None,
        maturity_time: None,
        maturity_height: None,
        strategies: None,
        locked_endowment_configs: None,
        rebalance: None,
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
        owner: Some(charity_addr.clone()),
        whitelisted_beneficiaries: None,
        whitelisted_contributors: None,
        name: None,
        description: None,
        withdraw_before_maturity: None,
        maturity_time: None,
        maturity_height: None,
        strategies: None,
        locked_endowment_configs: None,
        rebalance: None,
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
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // change the owner to some pleb
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
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
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
fn migrate_contract() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let _pleb = "plebAccount".to_string();

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile.clone(),
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // try to migrate the contract
    let msg = MigrateMsg {
        last_earnings_harvest: 10_u64,
    };
    let res = migrate(deps.as_mut(), env.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len())
}

#[test]
fn test_update_strategy() {
    let mut deps = mock_dependencies(&[]);

    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // sum of the invested strategy components percentages is not equal 100%
    let msg = ExecuteMsg::UpdateStrategies {
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(20),
                liquid_percentage: Decimal::percent(20),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(60),
                liquid_percentage: Decimal::percent(60),
            },
        ],
    };

    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidStrategyAllocation {});

    let msg = ExecuteMsg::UpdateStrategies {
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(20),
                liquid_percentage: Decimal::percent(20),
            },
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
        ],
    };

    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::StrategyComponentsNotUnique {});

    let msg = ExecuteMsg::UpdateStrategies {
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(60),
                liquid_percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());

    let msg = ExecuteMsg::UpdateStrategies {
        strategies: vec![
            Strategy {
                vault: "cash_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(40),
                liquid_percentage: Decimal::percent(40),
            },
            Strategy {
                vault: "tech_strategy_component_addr".to_string(),
                locked_percentage: Decimal::percent(60),
                liquid_percentage: Decimal::percent(60),
            },
        ],
    };
    let info = mock_info(pleb.as_ref(), &coins(100000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_endowment_profile() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let pleb = "plebAccount".to_string();

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
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
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
    let depositor = Addr::unchecked("depositor");
    let deposit_fee_perc = Decimal::percent(10);

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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("payout-address"),
            fee_percentage: deposit_fee_perc,
            active: true,
        }),
        withdraw_fee: None,
        aum_fee: None,
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
    let info = mock_info(depositor.as_str(), &coins(donation_amt, "uusd"));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    assert_eq!(res.attributes.len(), 3);

    // Check the "STATE" for "transactions" field
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&query_res).unwrap();
    // Since the `deposit_fee` is configured, the real `donation_amt` is less than original one.
    let deposit_fee = donation_amt * deposit_fee_perc.numerator() / deposit_fee_perc.denominator();
    assert_eq!(state.donations_received.u128(), donation_amt - deposit_fee);

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetTxRecords {
            sender: None,
            recipient: None,
            denom: None,
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
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
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
    let info = mock_info(depositor.as_str(), &coins(donation_amt, "uusd"));
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
    };
    let res = execute(deps.as_mut(), mock_env(), info, withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_query_endowment_fees() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("payout-wallet"),
            fee_percentage: Decimal::percent(3),
            active: false,
        }),
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // Query the "EndowmentFee"s
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::GetEndowmentFees {}).unwrap();
    let endow_fee_response: EndowmentFeesResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        endow_fee_response.earnings_fee,
        Some(EndowmentFee {
            payout_address: Addr::unchecked("payout-wallet"),
            fee_percentage: Decimal::percent(3),
            active: false,
        })
    );
    assert_eq!(endow_fee_response.deposit_fee, None);
    assert_eq!(endow_fee_response.withdraw_fee, None);
    assert_eq!(endow_fee_response.aum_fee, None);
}

#[test]
fn test_update_endowment_fees() {
    let mut deps = mock_dependencies(&[]);
    // meet the cast of characters
    let ap_team = "angelprotocolteamdano".to_string();
    let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
    let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
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
        cw4_members: vec![],
        dao: false,
        donation_match: false,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        locked_endowment_configs: vec![],
        name: "Test Endowment".to_string(),
        description: "Endowment to power an amazing charity".to_string(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        curve_type: None,
        split_max: Decimal::one(),
        split_min: Decimal::zero(),
        split_default: Decimal::percent(30),
        beneficiary: charity_addr.clone(),
        profile: profile,
        earnings_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("payout-wallet"),
            fee_percentage: Decimal::percent(3),
            active: false,
        }),
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

    // Update the "EndowmentFee"s
    let update_endowment_fees_msg = UpdateEndowmentFeesMsg {
        earnings_fee: None,
        deposit_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("another-payout-address"),
            fee_percentage: Decimal::percent(2),
            active: true,
        }),
        withdraw_fee: None,
        aum_fee: None,
    };

    let info = mock_info(&ap_team, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentFees(update_endowment_fees_msg),
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "update_endowment_fees"),
            attr("sender", ap_team.to_string()),
        ]
    );

    // Query the "EndowmentFee"s
    let query_res = query(deps.as_ref(), mock_env(), QueryMsg::GetEndowmentFees {}).unwrap();
    let endow_fee_response: EndowmentFeesResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        endow_fee_response.deposit_fee,
        Some(EndowmentFee {
            payout_address: Addr::unchecked("another-payout-address"),
            fee_percentage: Decimal::percent(2),
            active: true,
        })
    );
    assert_eq!(endow_fee_response.earnings_fee, None);
    assert_eq!(endow_fee_response.withdraw_fee, None);
    assert_eq!(endow_fee_response.aum_fee, None);
}
