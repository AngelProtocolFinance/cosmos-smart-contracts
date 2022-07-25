use crate::contract::{execute, instantiate, query};
use crate::msg::InitMsg;
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};

use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{ExecuteMsg, QueryMsg, RoutesUpdateMsg, UpdateConfigMsg};
use angel_core::responses::vault::ConfigResponse;

use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{coins, from_binary, to_binary, Addr, Coin, Decimal, OwnedDeps, Uint128};

fn create_vault(coins: Vec<Coin>) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut deps = mock_dependencies(&coins);
    let instantiate_msg = InitMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
        swap_pool_addr: "junoswap-pool".to_string(),
        staking_addr: "lp-staking-contract".to_string(),
        registrar_contract: "angelprotocolteamdano".to_string(),
        harvest_to_liquid: Decimal::percent(75),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let _ = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    deps
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InitMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
        swap_pool_addr: "junoswap-pool".to_string(),
        staking_addr: "lp-staking-contract".to_string(),
        registrar_contract: "angelprotocolteamdano".to_string(),
        harvest_to_liquid: Decimal::percent(75),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_update_owner() {
    // Instantiate the "vault" contract
    let mut deps = create_vault(vec![]);

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
    let mut deps = create_vault(vec![]);

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
    let mut deps = create_vault(vec![]);

    // Try to update the "config"
    let update_config_msg = UpdateConfigMsg {
        swap_pool_addr: Some("new-swap-pool-addr".to_string()),
        staking_addr: Some("new-staking-addr".to_string()),
        harvest_to_liquid: Some(Decimal::one()),
        routes: RoutesUpdateMsg {
            add: vec![],
            remove: vec![],
        },
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
    assert_eq!(config_resp.harvest_to_liquid, Decimal::one());
    assert_eq!(config_resp.staking_addr, "new-staking-addr".to_string());
    assert_eq!(config_resp.pool_addr, "new-swap-pool-addr".to_string());
}

#[test]
fn test_deposit_native_token() {
    // Instantiate the vault contract
    let mut deps = create_vault(vec![]);

    // Try to deposit the `native` token

    // First, fail to "deposit" token since non-Endowment calls the entry
    let info = mock_info("endowment-100", &coins(100, "ujuno"));
    let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Deposit {}).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to "deposit" JUNO tokens
    let info = mock_info("endowment-1", &coins(100, "ujuno"));
    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Deposit {}).unwrap();
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_deposit_cw20_token() {
    // Instantiate the vault contract
    let mut deps = create_vault(vec![]);

    // Try to deposit the "HALO"(cw20) token

    // First, fail to "deposit" token since non-Endowment calls the entry
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: "endowment-100".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::Deposit {}).unwrap(),
    };
    let info = mock_info("halo-token-contract", &[]);
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
        sender: "endowment-100".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::Deposit {}).unwrap(),
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
        sender: "endowment-1".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::Deposit {}).unwrap(),
    };
    let info = mock_info("halo-token-contract", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 3);
}
