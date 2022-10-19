use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Coin, Decimal, OwnedDeps, StdError, Uint128,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UpdateConfigMsg};
use crate::responses::{ConfigResponse, StateResponse};
use angel_core::errors::vault::ContractError;
use angel_core::structs::AccountType;
use cw20::TokenInfoResponse;

use crate::contract::{execute, instantiate, query};
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};

fn create_mock_vault(
    acct_type: AccountType,
    coins: Vec<Coin>,
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut deps = mock_dependencies(&coins);
    let instantiate_msg = InstantiateMsg {
        ibc_host: "ibc-relayer".to_string(),
        ibc_controller: "ibc-sender".to_string(),
        ap_tax_rate: Decimal::from_ratio(10_u128, 100_u128),
        interest_distribution: Decimal::from_ratio(25_u128, 100_u128),

        acct_type,
        sibling_vault: Some("sibling-vault".to_string()),
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),
        tax_collector: "tax-collector".to_string(),
        swap_router: "astroport-router".to_string(),

        lp_factory_contract: "astroport-factory".to_string(),
        lp_staking_contract: "astroport-generator".to_string(),
        pair_contract: "astroport-usdc-usdt-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        native_token: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
        reward_to_native_route: vec![],
        native_to_lp0_route: vec![],
        native_to_lp1_route: vec![],

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
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
        ibc_host: "ibc-relayer".to_string(),
        ibc_controller: "ibc-sender".to_string(),
        ap_tax_rate: Decimal::from_ratio(10_u128, 100_u128),
        interest_distribution: Decimal::from_ratio(25_u128, 100_u128),

        acct_type: AccountType::Locked,
        sibling_vault: Some("sibling-vault".to_string()),
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),
        tax_collector: "tax-collector".to_string(),
        swap_router: "astroport-router".to_string(),

        lp_factory_contract: "astroport-factory".to_string(),
        lp_staking_contract: "astroport-generator".to_string(),
        pair_contract: "astroport-usdc-usdt-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        native_token: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
        reward_to_native_route: vec![],
        native_to_lp0_route: vec![],
        native_to_lp1_route: vec![],

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_update_owner() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

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
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_resp: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_resp.owner, "new-owner".to_string());
}

#[test]
fn test_update_config() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

    // Try to update the "config"
    let update_config_msg = UpdateConfigMsg {
        ibc_host: Some("new-ibc-relayer".to_string()),
        ibc_controller: Some("new-ibc-sender".to_string()),

        lp_staking_contract: Some("new-astroport-generator".to_string()),
        lp_pair_contract: Some("new-astroport-pair".to_string()),
        keeper: Some("new-keeper".to_string()),
        sibling_vault: None,
        tax_collector: Some("new-tax-collector".to_string()),

        native_token: Some(cw_asset::AssetInfoBase::Native("ujuno".to_string())),
        reward_to_native_route: None,
        native_to_lp0_route: None,
        native_to_lp1_route: None,
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
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config.ibc_host, "new-ibc-relayer".to_string());
    assert_eq!(config.ibc_controller, "new-ibc-sender".to_string());
    assert_eq!(config.lp_pair_contract, "new-astroport-pair".to_string());
    assert_eq!(
        config.lp_staking_contract,
        "new-astroport-generator".to_string()
    );
    assert_eq!(config.keeper, "new-keeper".to_string());
    assert_eq!(config.tax_collector, "new-tax-collector".to_string());
}

#[test]
fn test_deposit_native_token() {
    // Instantiate the vault contract
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

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
    let info = mock_info("ibc-relayer", &coins(100, "ujuno"));
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
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        ibc_host: "ibc-relayer".to_string(),
        ibc_controller: "ibc-sender".to_string(),
        ap_tax_rate: Decimal::from_ratio(10_u128, 100_u128),
        interest_distribution: Decimal::from_ratio(25_u128, 100_u128),

        acct_type: AccountType::Locked,
        sibling_vault: Some("sibling-vault".to_string()),
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),
        tax_collector: "tax-collector".to_string(),
        swap_router: "astroport-router".to_string(),

        lp_factory_contract: "astroport-factory".to_string(),
        lp_staking_contract: "astroport-generator".to_string(),
        pair_contract: "astroport-usdc-usdt-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        native_token: cw_asset::AssetInfoBase::Cw20(Addr::unchecked("halo-token")),
        reward_to_native_route: vec![],
        native_to_lp0_route: vec![],
        native_to_lp1_route: vec![],

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let _ = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

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
        sender: "ibc-relayer".to_string(),
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
        sender: "ibc-relayer".to_string(),
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
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

    // First, fail to "redeem" since VT amount for Endowment is insufficient
    let info = mock_info("ibc-relayer", &[]);
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
    assert_eq!(err, ContractError::Std(StdError::GenericErr { msg: "Cannot burn the 30 vault tokens from Endowment 12 :: Overflow: Cannot Sub with 0 and 30".to_string() }));

    // Also, fail to "redeem" since the `vault` does not have any deposit
    let info = mock_info("ibc-relayer", &[]);
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
                "Cannot burn the {} vault tokens from Endowment {} :: Overflow: Cannot Sub with 0 and 30",
                redeem_amount, endowment_id
            )
        })
    );
}

#[test]
fn test_harvest() {
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

    // Only "config.keeper" address can call the "harvest" entry
    let info = mock_info("non-keeper", &[]);
    let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Harvest {}).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // "claim" entry outputs 2 messages
    let info = mock_info("keeper", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Harvest {}).unwrap();
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_stake_lp_token_entry() {
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

    // Fail to stake since zero LP amount to stake
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::from(100_u128),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // Only the contract itself can call "StakeLpToken" entry
    let info = mock_info("anyone", &[]);

    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to "stake" the LP tokens
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);

    // Check if only 1 VT(vault token) is minted for Endowment 1.
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.total_lp_amount, "100");
    assert_eq!(state.total_shares, "1000000");

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance { endowment_id: 1 },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(1000000_u128));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
    let token_info_resp: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(token_info_resp.total_supply, Uint128::from(1000000_u128));

    // Succeed to 'stake" lp token again
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);

    // Check the VT(vault token) balance
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.total_lp_amount, (100 + 100).to_string());
    let expected_total_share: u128 = 1000000 + 100 * 1000000 / 200;
    assert_eq!(state.total_shares, expected_total_share.to_string());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance { endowment_id: 1 },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(expected_total_share));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
    let token_info_resp: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        token_info_resp.total_supply,
        Uint128::from(expected_total_share)
    );

    // Mint the VT(vault token) to other endowment
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(2_u32),
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);

    // Check the VT(vault token) balance
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.total_lp_amount, (200 + 100).to_string());
    let minted_amount: u128 = 100 * 1500000 / 300;
    let expected_total_share: u128 = 1500000 + minted_amount;
    assert_eq!(state.total_shares, expected_total_share.to_string());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance { endowment_id: 2 },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(minted_amount));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
    let token_info_resp: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(
        token_info_resp.total_supply,
        Uint128::from(expected_total_share)
    );
}

#[test]
fn test_redeem_lp_token() {
    let mut deps = create_mock_vault(AccountType::Locked, vec![]);

    // Mint 1 VT for the Endowment 1.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);

    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    // Fail to redeem since non-accounts contract calls the entry
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Redeem {
            endowment_id: 1_u32,
            amount: Uint128::from(1000_u128),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to redeem since the VT amount for Endowment is insufficient
    let info = mock_info("ibc-relayer", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Redeem {
            endowment_id: 2_u32,
            amount: Uint128::from(1000_u128),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Std(StdError::GenericErr { msg: "Cannot burn the 1000 vault tokens from Endowment 2 :: Overflow: Cannot Sub with 0 and 1000".to_string() }));

    // Succeed to redeem for Endowment 1.
    let info = mock_info("ibc-relayer", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Redeem {
            endowment_id: 1_u32,
            amount: Uint128::from(100000_u128),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 3);

    // Check the VT(vault token) balance
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.total_lp_amount,
        (100 - 100000 * 100 / 1000000).to_string()
    );
    assert_eq!(state.total_shares, (1000000 - 100000).to_string());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance { endowment_id: 1 },
    )
    .unwrap();
    let balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(balance, Uint128::from(900000_u128));

    let res = query(deps.as_ref(), mock_env(), QueryMsg::TokenInfo {}).unwrap();
    let token_info_resp: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(token_info_resp.total_supply, Uint128::from(900000_u128));

    // Check the case of "redeem" from config.tax_collector
    let info = mock_info("tax-collector", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Redeem {
            endowment_id: 100_u32,
            amount: Uint128::from(100_u128),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::generic_err(
            "Cannot burn the 100 vault tokens from tax-collector :: Overflow: Cannot Sub with 0 and 100"
        ))
    );
}

#[test]
fn test_reinvest_to_locked() {
    let mut deps = create_mock_vault(AccountType::Liquid, vec![]);

    // Mint 1 VT for the Endowment 1.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    // Only "accounts-contract" can call the "reinvest_to_locked" entry
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(100_u128),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to "reinvest_to_locked" since the burn amount is bigger than 1 VT
    let info = mock_info("ibc-relayer", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(10000000_u128),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Insufficient balance: Needed {}, existing: {}",
                10000000, 1000000
            )
        })
    );

    // Succeed to "reinvest_to_locked"
    let info = mock_info("ibc-relayer", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(1000000_u128),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 3);

    // Check the VT(vault token) balance
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.total_lp_amount,
        (100 - 1000000 * 100 / 1000000).to_string()
    );
    assert_eq!(state.total_shares, (1000000 - 1000000).to_string());
}
