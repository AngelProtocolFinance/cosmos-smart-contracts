use super::mock_querier::mock_dependencies;
use crate::contract::{execute, instantiate, migrate, query, reply};
use angel_core::errors::core::*;

use angel_core::msgs::accounts_settings_controller::{
    ConfigResponse, EndowmentPermissionsResponse, EndowmentSettingsResponse,
};
use angel_core::msgs::accounts_settings_controller::{
    CreateEndowSettingsMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, UpdateConfigMsg,
    UpdateEndowmentSettingsMsg, UpdateMaturityAllowlist,
};
use angel_core::structs::{EndowmentController, EndowmentFee, EndowmentType};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, Decimal, Event, Reply, StdError, SubMsgResponse, Uint128};

const AP_TEAM: &str = "juno1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const ENDOW_ID: u32 = 1;
const NOT_EXISTING_ENDOW_ID: u32 = 2;
const REGISTRAR_CONTRACT: &str = "juno18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "juno17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
const DEPOSITOR: &str = "depositor";

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the config
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config.owner, AP_TEAM.to_string());
    assert_eq!(config.registrar_contract, REGISTRAR_CONTRACT.to_string());

    // Check the endowment permissions
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentPermissions {
            id: ENDOW_ID,
            updater: Addr::unchecked("anyone"),
        },
    )
    .unwrap();
    let permission: EndowmentPermissionsResponse = from_binary(&res).unwrap();
    assert!(!permission.aum_fee);
    assert!(!permission.endowment_controller);
    assert!(!permission.maturity_allowlist);
}

#[test]
fn test_update_config() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Only owner can update the config
    let info = mock_info("anyone", &[]);
    let update_config_msg = UpdateConfigMsg {
        owner: None,
        registrar_contract: None,
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the config
    let info = mock_info(AP_TEAM, &[]);
    let update_config_msg = UpdateConfigMsg {
        owner: Some("new-owner".to_string()),
        registrar_contract: Some("new-registrar".to_string()),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Check the new config
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config.owner, "new-owner".to_string());
    assert_eq!(config.registrar_contract, "new-registrar".to_string());
}

#[test]
fn test_create_endowment_settings() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Only the "accounts_contract" can call this entry.
    let info = mock_info("non-accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: true,
        donation_match_contract: Some(Addr::unchecked("donation-match-contract")),
        beneficiaries_allowlist: vec![PLEB.to_string()],
        contributors_allowlist: vec![DEPOSITOR.to_string()],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: true,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg.clone()),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Cannot create EndowmentSettings for ID, which has been already created.
    let info = mock_info("accounts-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::AlreadyInUse {});

    // Check the created EndowmentSettings
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentSettings { id: 1 },
    )
    .unwrap();
    let endow_settings: EndowmentSettingsResponse = from_binary(&res).unwrap();
    assert_eq!(endow_settings.dao, None);
    assert_eq!(endow_settings.dao_token, None);
    assert_eq!(
        endow_settings.donation_match_active,
        msg.donation_match_active
    );
    assert_eq!(
        endow_settings.donation_match_contract,
        msg.donation_match_contract
    );
    assert_eq!(
        endow_settings.beneficiaries_allowlist,
        msg.beneficiaries_allowlist
    );
    assert_eq!(
        endow_settings.contributors_allowlist,
        msg.contributors_allowlist
    );
    assert_eq!(endow_settings.maturity_allowlist, msg.maturity_allowlist);
    assert_eq!(endow_settings.parent, msg.parent);
    assert_eq!(endow_settings.split_to_liquid, msg.split_to_liquid);
    assert_eq!(endow_settings.ignore_user_splits, msg.ignore_user_splits);
    assert_eq!(endow_settings.earnings_fee, msg.earnings_fee);
}

#[test]
fn test_update_endowment_settings() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![PLEB.to_string()],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    let mut msg = UpdateEndowmentSettingsMsg {
        id: 2,
        donation_match_active: Some(true),
        beneficiaries_allowlist: Some(vec![AP_TEAM.to_string(), PLEB.to_string()]),
        contributors_allowlist: Some(vec![DEPOSITOR.to_string()]),
        maturity_allowlist: Some(UpdateMaturityAllowlist {
            add: vec![],
            remove: vec![],
        }),
        ignore_user_splits: Some(true),
        split_to_liquid: None,
        earnings_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("beneficiary1"),
            fee_percentage: Decimal::percent(10),
            active: true,
        }),
        deposit_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("beneficiary2"),
            fee_percentage: Decimal::percent(5),
            active: true,
        }),
        withdraw_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("beneficiary3"),
            fee_percentage: Decimal::percent(5),
            active: true,
        }),
        aum_fee: Some(EndowmentFee {
            payout_address: Addr::unchecked("beneficiary4"),
            fee_percentage: Decimal::percent(15),
            active: true,
        }),
    };

    // Endowment state should be NOT closing
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("accounts-contract", &[]),
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::UpdatesAfterClosed {});

    // sender SHOULD be either of "endowment owner" or the "endowment gov" contracts (or a delegate address)
    msg.id = ENDOW_ID;
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(PLEB, &[]),
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the Endowment settings
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("endowment-owner", &[]),
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap();

    // Check the result of updated endowment settings
    let endow_settings: EndowmentSettingsResponse = from_binary(
        &query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::EndowmentSettings { id: ENDOW_ID },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        endow_settings.ignore_user_splits,
        msg.ignore_user_splits.clone().unwrap()
    );
    assert_eq!(
        endow_settings.beneficiaries_allowlist,
        vec![AP_TEAM.to_string(), PLEB.to_string()]
    );
    assert_eq!(endow_settings.aum_fee, msg.aum_fee);
    assert_eq!(endow_settings.earnings_fee, msg.earnings_fee);
    assert_eq!(endow_settings.deposit_fee, msg.deposit_fee);
    assert_eq!(endow_settings.withdraw_fee, msg.withdraw_fee);
}

#[test]
fn test_update_delegate() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    // "setting" name should be correct
    let info = mock_info("endowment-owner", &[]);
    let _err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateDelegate {
            endowment_id: ENDOW_ID,
            setting: "any-fee".to_string(),
            action: "set".to_string(),
            delegate_address: "new-delegate-address".to_string(),
            delegate_expiry: None,
        },
    )
    .unwrap_err();

    // "action" should be either of "set" or "revoke"
    let info = mock_info("endowment-owner", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateDelegate {
            endowment_id: ENDOW_ID,
            setting: "aum_fee".to_string(),
            action: "blahblah".to_string(),
            delegate_address: "new-delegate-address".to_string(),
            delegate_expiry: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Succeed to update the settings
    let info = mock_info("endowment-owner", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateDelegate {
            endowment_id: ENDOW_ID,
            setting: "aum_fee".to_string(),
            action: "set".to_string(),
            delegate_address: "new-delegate-address".to_string(),
            delegate_expiry: None,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_setup_dao() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    // Only the Endowment owner can call this entry
    let info = mock_info(AP_TEAM, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetupDao {
            endowment_id: ENDOW_ID,
            setup: angel_core::structs::DaoSetup {
                quorum: Decimal::percent(10),
                threshold: Decimal::percent(50),
                voting_period: 300,
                timelock_period: 200,
                expiration_period: 500,
                proposal_deposit: Uint128::from(1000000_u128),
                snapshot_period: 100,
                token: angel_core::structs::DaoToken::NewCw20 {
                    initial_supply: Uint128::from(1000000_u128),
                    name: "New cw20".to_string(),
                    symbol: "NC2".to_string(),
                },
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed
    let info = mock_info("endowment-owner", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetupDao {
            endowment_id: ENDOW_ID,
            setup: angel_core::structs::DaoSetup {
                quorum: Decimal::percent(10),
                threshold: Decimal::percent(50),
                voting_period: 300,
                timelock_period: 200,
                expiration_period: 500,
                proposal_deposit: Uint128::from(1000000_u128),
                snapshot_period: 100,
                token: angel_core::structs::DaoToken::NewCw20 {
                    initial_supply: Uint128::from(1000000_u128),
                    name: "New cw20".to_string(),
                    symbol: "NC2".to_string(),
                },
            },
        },
    )
    .unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn test_setup_donation_match() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    // Only the Endowment owner can call this entry
    let info = mock_info(AP_TEAM, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetupDonationMatch {
            endowment_id: ENDOW_ID,
            setup: angel_core::structs::DonationMatch::Cw20TokenReserve {
                reserve_addr: "reserve-token".to_string(),
                lp_addr: "lp-token".to_string(),
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed
    let info = mock_info("endowment-owner", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetupDonationMatch {
            endowment_id: ENDOW_ID,
            setup: angel_core::structs::DonationMatch::Cw20TokenReserve {
                reserve_addr: "reserve-token".to_string(),
                lp_addr: "lp-token".to_string(),
            },
        },
    )
    .unwrap();
    assert_eq!(1, res.messages.len());

    let info = mock_info("endowment-owner", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetupDonationMatch {
            endowment_id: ENDOW_ID,
            setup: angel_core::structs::DonationMatch::HaloTokenReserve {},
        },
    )
    .unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn test_setup_dao_reply() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    // Only reply id "1" is acceptable.
    let reply_msg = Reply {
        id: 0,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };
    let err = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // SubMsgResult should be Ok
    let reply_msg = Reply {
        id: ENDOW_ID as u64,
        result: cosmwasm_std::SubMsgResult::Err("error".to_string()),
    };
    let err = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "error".to_string()
        })
    );

    // SubMsgResult should have results in "events"
    let reply_msg = Reply {
        id: ENDOW_ID as u64,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };
    let err = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(err, ContractError::AccountNotCreated {});

    // Succeed to save Dao info in EndowmentSettings
    let reply_msg = Reply {
        id: ENDOW_ID as u64,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![Event::new("wasm")
                .add_attribute("endow_id", "1")
                .add_attribute("dao_addr", "dao_addr")
                .add_attribute("dao_token_addr", "dao_token_addr")],
            data: None,
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Check the result
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentSettings { id: 1 },
    )
    .unwrap();
    let endow_settings: EndowmentSettingsResponse = from_binary(&res).unwrap();
    assert_eq!(endow_settings.dao, Some(Addr::unchecked("dao_addr")));
    assert_eq!(
        endow_settings.dao_token,
        Some(Addr::unchecked("dao_token_addr"))
    );
}

#[test]
fn test_donation_match_reply() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    // Only reply id "2" is acceptable.
    let reply_msg = Reply {
        id: 0,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };
    let err = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // SubMsgResult should be Ok
    let reply_msg = Reply {
        id: NOT_EXISTING_ENDOW_ID as u64,
        result: cosmwasm_std::SubMsgResult::Err("error".to_string()),
    };
    let err = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "error".to_string()
        })
    );

    // SubMsgResult should have results in "events"
    let reply_msg = Reply {
        id: NOT_EXISTING_ENDOW_ID as u64,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };
    let err = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(err, ContractError::AccountNotCreated {});

    // Succeed to save Dao info in EndowmentSettings
    let reply_msg = Reply {
        id: NOT_EXISTING_ENDOW_ID as u64,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![Event::new("wasm")
                .add_attribute("endow_id", "1")
                .add_attribute("donation_match_addr", "donation_match_addr")],
            data: None,
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // Check the result
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentSettings { id: 1 },
    )
    .unwrap();
    let endow_settings: EndowmentSettingsResponse = from_binary(&res).unwrap();
    assert_eq!(
        endow_settings.donation_match_contract,
        Some(Addr::unchecked("donation_match_addr"))
    );
}

#[test]
fn test_migrate() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        owner_sc: AP_TEAM.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(AP_TEAM, &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to create EndowmentSettings
    let info = mock_info("accounts-contract", &[]);
    let msg = CreateEndowSettingsMsg {
        id: ENDOW_ID,
        donation_match_active: false,
        donation_match_contract: None,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        maturity_allowlist: vec![],
        endowment_controller: EndowmentController::default(&EndowmentType::Normal),
        parent: None,
        split_to_liquid: None,
        ignore_user_splits: false,
        earnings_fee: None,
        deposit_fee: None,
        withdraw_fee: None,
        aum_fee: None,
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CreateEndowmentSettings(msg),
    )
    .unwrap();

    let _err = migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap_err();
}
