use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Coin, Decimal, OwnedDeps, StdError, Uint128,
};

use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{
    AccountWithdrawMsg, ExecuteMsg, InstantiateMsg, QueryMsg, UpdateConfigMsg,
};
use angel_core::responses::vault::ConfigResponse;
use angel_core::structs::AccountType;

use crate::contract::{execute, instantiate, query};
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};

fn create_mock_vault(coins: Vec<Coin>) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut deps = mock_dependencies(&coins);
    let instantiate_msg = InstantiateMsg {
        acct_type: AccountType::Locked,
        sibling_vault: Some("sibling-vault".to_string()),
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),

        lp_staking_contract: "loop-farming".to_string(),
        pair_contract: "loop-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,

        harvest_to_liquid: Decimal::from_ratio(10_u128, 100_u128),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let _ = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    deps
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        acct_type: AccountType::Locked,
        sibling_vault: Some("sibling-vault".to_string()),
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),

        lp_staking_contract: "loop-farming".to_string(),
        pair_contract: "loop-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,

        harvest_to_liquid: Decimal::from_ratio(10_u128, 100_u128),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_update_owner() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(vec![]);

    // Try to update the "owner"
    // Fail to update "owner" since non-owner calls the entry
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateOwner {
            new_owner: "new-owner".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update "owner" since real owner calls the entry
    let info = mock_info("creator", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateOwner {
            new_owner: "new-owner".to_string(),
        },
    )
    .unwrap();

    // Check if the "owner" has been changed
    let res = query(
        deps.as_ref(),
        mock_env(),
        angel_core::messages::vault::QueryMsg::Config {},
    )
    .unwrap();
    let config_resp: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_resp.owner, "new-owner".to_string());
}

#[test]
fn test_update_registrar() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(vec![]);

    // Try to update the "registrar" contract address
    // Fail to update the "registrar" since non-"registrar" address calls the entry
    let info = mock_info("any-address", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateRegistrar {
            new_registrar: Addr::unchecked("new-registrar"),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the "registrar" contract address
    let info = mock_info("angelprotocolteamdano", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateRegistrar {
            new_registrar: Addr::unchecked("new-registrar"),
        },
    )
    .unwrap();

    // Check if the "registrar" contract address has been changed
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_resp: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_resp.registrar_contract, "new-registrar".to_string());
}

#[test]
fn test_update_config() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(vec![]);

    // Try to update the "config"
    let update_config_msg = UpdateConfigMsg {
        lp_staking_contract: Some("new-loop-farming".to_string()),
        pair_contract: Some("new-loop-pair".to_string()),
        keeper: Some("new-keeper".to_string()),
        sibling_vault: None,
    };

    // Only "config.owner" can update the config, otherwise fails
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the "config"
    let info = mock_info("creator", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg.clone()),
    )
    .unwrap();

    // Check the "config" update
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_resp: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_resp.pair_contract, "new-loop-pair".to_string());
    assert_eq!(
        config_resp.lp_staking_contract,
        "new-loop-farming".to_string()
    );
    assert_eq!(config_resp.keeper, "new-keeper".to_string());
}

#[test]
fn test_deposit_native_token() {
    // Instantiate the vault contract
    let mut deps = create_mock_vault(vec![]);

    // Try to deposit the `native` token

    // First, fail to "deposit" token since non-Endowment calls the entry
    let info = mock_info("endowment-100", &coins(100, "ujuno"));
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { endowment_id: 10 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to "deposit" JUNO tokens
    let info = mock_info("accounts-contract", &coins(100, "ujuno"));
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { endowment_id: 1 },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_deposit_cw20_token() {
    // Instantiate the vault contract
    let mut deps = create_mock_vault(vec![]);

    // Try to deposit the "HALO"(cw20) token

    // First, fail to "deposit" token since non-Endowment calls the entry
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: "endowment-100".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::Deposit { endowment_id: 10 })
            .unwrap(),
    };
    let info = mock_info("halo-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Second, fail to "deposit" since the "token" deposited is not one of "input_denoms"
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: "accounts-contract".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::Deposit { endowment_id: 1 })
            .unwrap(),
    };
    let info = mock_info("cw20-token-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Succeed to "deposit" HALO tokens
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: "accounts-contract".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::Deposit { endowment_id: 1 })
            .unwrap(),
    };
    let info = mock_info("halo-token", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 3);
}

#[test]
fn test_redeem() {
    let endowment_id = 1;
    let fake_endowment_id = 12;
    let _deposit_amount = Uint128::from(100_u128);
    let redeem_amount = Uint128::from(30_u128);

    // Instantiate the vault contract
    let mut deps = create_mock_vault(vec![]);

    // First, fail to "redeem" since the `endowment` is not valid
    let info = mock_info("accounts-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Redeem {
            endowment_id: fake_endowment_id,
            amount: redeem_amount,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Also, fail to "redeem" since the `vault` does not have any deposit
    let info = mock_info("accounts-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Redeem {
            endowment_id,
            amount: redeem_amount,
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {}",
                redeem_amount, endowment_id
            )
        })
    );
}

#[test]
fn test_harvest() {
    let mut deps = create_mock_vault(vec![]);

    // Only "config.keeper" address can call the "harvest" entry
    let info = mock_info("non-keeper", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Harvest {}).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // "claim" entry outputs 2 messages
    let info = mock_info("keeper", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Harvest {}).unwrap();
    assert_eq!(res.messages.len(), 2);
}
