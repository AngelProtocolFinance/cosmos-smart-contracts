use super::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;

use angel_core::messages::settings_controller::{
    CreateEndowSettingsMsg, ExecuteMsg, InstantiateMsg, QueryMsg, UpdateConfigMsg,
    UpdateEndowmentFeesMsg, UpdateEndowmentSettingsMsg, UpdateMaturityWhitelist,
};
use angel_core::responses::settings_controller::{ConfigResponse, EndowmentSettingsResponse};
use angel_core::structs::{
    AccountType, Beneficiary, Categories, EndowmentFee, EndowmentType, SettingsController,
    SettingsPermissions, SplitDetails, StrategyComponent, SwapOperation,
};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, coins, from_binary, to_binary, Addr, Coin, Decimal, Env, OwnedDeps, StdError, Uint128,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetUnchecked};
use cw_utils::{Expiration, Threshold};

const AP_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const CHARITY_ID: u32 = 1;
const CHARITY_ADDR: &str = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v";
const REGISTRAR_CONTRACT: &str = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
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
        id: 1,
        donation_match_active: true,
        donation_match_contract: Some(Addr::unchecked("donation-match-contract")),
        whitelisted_beneficiaries: vec![PLEB.to_string()],
        whitelisted_contributors: vec![DEPOSITOR.to_string()],
        maturity_whitelist: vec![],
        settings_controller: SettingsController::default(),
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
        endow_settings.whitelisted_beneficiaries,
        msg.whitelisted_beneficiaries
    );
    assert_eq!(
        endow_settings.whitelisted_contributors,
        msg.whitelisted_contributors
    );
    assert_eq!(endow_settings.maturity_whitelist, msg.maturity_whitelist);
    assert_eq!(endow_settings.settings_controller, msg.settings_controller);
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
        id: 1,
        donation_match_active: false,
        donation_match_contract: None,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        maturity_whitelist: vec![],
        settings_controller: SettingsController::default(),
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

    // Only "accounts" contract can call this entry
    let info = mock_info("non-accounts-contract", &[]);
    let mut msg = UpdateEndowmentSettingsMsg {
        setting_updater: Addr::unchecked("anyone"),
        id: 1,
        donation_match_active: Some(true),
        whitelisted_beneficiaries: Some(vec![PLEB.to_string()]),
        whitelisted_contributors: Some(vec![DEPOSITOR.to_string()]),
        maturity_whitelist: Some(UpdateMaturityWhitelist {
            add: vec![],
            remove: vec![],
        }),
        settings_controller: Some(SettingsController::default()),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Endowment state should be NOT closing
    let info = mock_info("accounts-contract", &[]);
    msg.id = 2;
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::UpdatesAfterClosed {});

    // "setting_updater" SHOULD be either of "config.owner" or "endowment owner"
    let info = mock_info("accounts-contract", &[]);
    msg.id = 1;
    msg.setting_updater = Addr::unchecked("anyone");
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the Endowment settings
    let info = mock_info("accounts-contract", &[]);
    msg.id = 1;
    msg.setting_updater = Addr::unchecked(AP_TEAM);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentSettings(msg.clone()),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Check the result of updated endowment settings
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentSettings { id: 1 },
    )
    .unwrap();
    let endow_settings: EndowmentSettingsResponse = from_binary(&res).unwrap();
    assert_eq!(
        endow_settings.settings_controller,
        msg.settings_controller.clone().unwrap()
    );
}

#[test]
fn test_update_endowment_fees() {
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
        id: 1,
        donation_match_active: false,
        donation_match_contract: None,
        whitelisted_beneficiaries: vec![],
        whitelisted_contributors: vec![],
        maturity_whitelist: vec![],
        settings_controller: SettingsController::default(),
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

    // Endowment SHOULD be "Charity" type
    let info = mock_info("anyone", &[]);
    let mut msg = UpdateEndowmentFeesMsg {
        id: 2,
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
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentFees(msg.clone()),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Charity Endowments may not change endowment fees".to_string()
        })
    );

    // Succeed to update the endowment fees
    let info = mock_info("endowment-owner", &[]);
    msg.id = 1;
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowmentFees(msg.clone()),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Check the result
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentSettings { id: msg.id },
    )
    .unwrap();
    let endow_settings: EndowmentSettingsResponse = from_binary(&res).unwrap();
    assert_eq!(endow_settings.aum_fee, msg.aum_fee);
    assert_eq!(endow_settings.earnings_fee, msg.earnings_fee);
    assert_eq!(endow_settings.deposit_fee, msg.deposit_fee);
    assert_eq!(endow_settings.withdraw_fee, msg.withdraw_fee);
}
