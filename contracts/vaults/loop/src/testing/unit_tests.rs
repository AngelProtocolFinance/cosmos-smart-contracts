use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Coin, OverflowError, OwnedDeps, StdError, Uint128,
};

use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{ExecuteMsg, InstantiateMsg, QueryMsg, UpdateConfigMsg};
use angel_core::responses::vault::{ConfigResponse, StateResponse};
use angel_core::structs::{AccountType, SwapOperation};
use cw20::{BalanceResponse, TokenInfoResponse};
use cw_asset::AssetInfoBase;

use crate::contract::{execute, instantiate, migrate, query};
use crate::executers::PENDING_OWNER_DEADLINE;
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};

fn create_mock_vault(
    acct_type: AccountType,
    sibling_vault: Option<String>,
    coins: Vec<Coin>,
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let mut deps = mock_dependencies(&coins);
    let instantiate_msg = InstantiateMsg {
        acct_type,
        sibling_vault,
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),
        tax_collector: "tax-collector".to_string(),
        swap_router: "swap-router".to_string(),

        lp_factory_contract: "loop-factory".to_string(),
        lp_staking_contract: "loop-farming".to_string(),
        pair_contract: "loop-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        native_token: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
        reward_to_native_route: vec![],
        native_to_lp0_route: vec![],
        native_to_lp1_route: vec![],

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,

        minimum_initial_deposit: Uint128::from(100_u128),
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
        tax_collector: "tax-collector".to_string(),
        swap_router: "swap-router".to_string(),

        lp_factory_contract: "loop-factory".to_string(),
        lp_staking_contract: "loop-farming".to_string(),
        pair_contract: "loop-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        native_token: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
        reward_to_native_route: vec![],
        native_to_lp0_route: vec![],
        native_to_lp1_route: vec![],

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,

        minimum_initial_deposit: Uint128::from(100_u128),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the initial state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::TotalBalance {}).unwrap();
    let total_balance: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(total_balance.balance, Uint128::zero());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::ApTaxBalance {}).unwrap();
    let ap_tax: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(ap_tax.balance, Uint128::zero());
}

#[test]
fn test_update_owner() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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

    // Check the `pending_owner` & `pending_owner_deadline` settings
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config.owner, "creator".to_string());
    assert_eq!(config.pending_owner, "new-owner".to_string());
    assert_eq!(
        config.pending_owner_deadline,
        mock_env().block.height + PENDING_OWNER_DEADLINE
    );

    let info = mock_info("new-owner", &[]);
    let _ = execute(
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
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config.owner, "new-owner".to_string());
    assert_eq!(config.pending_owner, "".to_string());
    assert_eq!(config.pending_owner_deadline, 0);
}

#[test]
fn test_update_registrar() {
    // Instantiate the "vault" contract
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // Try to update the "config"
    let mut update_config_msg = UpdateConfigMsg {
        keeper: Some("new-keeper".to_string()),
        sibling_vault: None,
        tax_collector: Some("new-tax-collector".to_string()),

        native_token: Some(cw_asset::AssetInfoBase::Native("ujuno".to_string())),
        reward_to_native_route: None,
        native_to_lp0_route: None,
        native_to_lp1_route: None,

        minimum_initial_deposit: Some(Uint128::from(200_u128)),
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

    // `native_token` can be `cw20 token
    update_config_msg.native_token = Some(cw_asset::AssetInfoBase::Cw20(Addr::unchecked("token")));
    let info = mock_info("creator", &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg.clone()),
    )
    .unwrap();

    // Succeed to update the "config"
    update_config_msg.native_token = Some(cw_asset::AssetInfoBase::Native("ujuno".to_string()));
    update_config_msg.native_to_lp0_route = Some(vec![SwapOperation::Loop {
        offer_asset_info: AssetInfoBase::Native("ujuno".to_string()),
        ask_asset_info: AssetInfoBase::Cw20(Addr::unchecked("token1")),
    }]);
    update_config_msg.native_to_lp1_route = Some(vec![SwapOperation::Loop {
        offer_asset_info: AssetInfoBase::Native("ujuno".to_string()),
        ask_asset_info: AssetInfoBase::Cw20(Addr::unchecked("token2")),
    }]);
    update_config_msg.reward_to_native_route = Some(vec![SwapOperation::Loop {
        offer_asset_info: AssetInfoBase::Cw20(Addr::unchecked("lp-reward")),
        ask_asset_info: AssetInfoBase::Native("ujuno".to_string()),
    }]);
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
    assert_eq!(config_resp.keeper, "new-keeper".to_string());
    assert_eq!(config_resp.tax_collector, "new-tax-collector".to_string());
    assert_eq!(config_resp.minimum_initial_deposit, "200".to_string());
}

#[test]
fn test_deposit_native_token() {
    // Instantiate the vault contract
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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

    // Should send only one token, NOT several tokens
    let info = mock_info(
        "accounts-contract",
        &[Coin::new(100, "ujuno"), Coin::new(100, "ucosm")],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { endowment_id: 10 },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Invalid: Multiple coins sent. Only accepts a single token as input.".to_string(),
        })
    );

    // Deposit amount should NOT be zero.
    let info = mock_info("accounts-contract", &coins(0, "ujuno"));
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { endowment_id: 1 },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::EmptyBalance {});

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
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        acct_type: AccountType::Locked,
        sibling_vault: Some("sibling-vault".to_string()),
        registrar_contract: "angelprotocolteamdano".to_string(),
        keeper: "keeper".to_string(),
        tax_collector: "tax-collector".to_string(),
        swap_router: "swap-router".to_string(),

        lp_factory_contract: "loop-factory".to_string(),
        lp_staking_contract: "loop-farming".to_string(),
        pair_contract: "loop-pair".to_string(),
        lp_reward_token: "lp-reward-token".to_string(),

        native_token: cw_asset::AssetInfoBase::Cw20(Addr::unchecked("halo-token")),
        reward_to_native_route: vec![],
        native_to_lp0_route: vec![],
        native_to_lp1_route: vec![],

        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,

        minimum_initial_deposit: Uint128::from(100_u128),
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
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_redeem() {
    let endowment_id = 1;
    let fake_endowment_id = 12;
    let _deposit_amount = Uint128::from(100_u128);
    let redeem_amount = Uint128::from(30_u128);

    // Instantiate the vault contract
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // First, fail to "redeem" since VT amount for Endowment is insufficient
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
    assert_eq!(err, ContractError::Std(StdError::GenericErr { msg: "Cannot burn the 30 vault tokens from Endowment 12 :: Overflow: Cannot Sub with 0 and 30".to_string() }));

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
                "Cannot burn the {} vault tokens from Endowment {} :: Overflow: Cannot Sub with 0 and 30",
                redeem_amount, endowment_id
            )
        })
    );
}

#[test]
fn test_harvest() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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
fn test_harvest_to_liquid() {
    // Cannot call the `harvest_to_liqduid` entry of `Locked` vault
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    let harvest_to_liquid_msg = cw20::Cw20ReceiveMsg {
        sender: "liquid_sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::HarvestToLiquid {}).unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(harvest_to_liquid_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // LP token(cw20) should be sent for this entry
    let mut deps = create_mock_vault(
        AccountType::Liquid,
        Some("locked_sibling_vault".to_string()),
        vec![],
    );

    let harvest_to_liquid_msg = cw20::Cw20ReceiveMsg {
        sender: "locked_sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::HarvestToLiquid {}).unwrap(),
    };

    let info = mock_info("non-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(harvest_to_liquid_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Sibling_vault should send the LP tokens
    let harvest_to_liquid_msg = cw20::Cw20ReceiveMsg {
        sender: "non_liquid_sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::HarvestToLiquid {}).unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(harvest_to_liquid_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to `harvest_to_liquid`
    let harvest_to_liquid_msg = cw20::Cw20ReceiveMsg {
        sender: "locked_sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::HarvestToLiquid {}).unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(harvest_to_liquid_msg),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    // Check the LP amount increase in liquid vault
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.total_lp_amount, Uint128::from(100_u128).to_string());
    assert_eq!(state.total_shares, Uint128::zero().to_string());
}

#[test]
fn test_stake_lp_token_from_deposit() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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

    // Computation of `lp_amount` can detect overflow/underflow error.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::from(200_u128),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::Overflow {
            source: OverflowError {
                operation: cosmwasm_std::OverflowOperation::Sub,
                operand1: "100".to_string(),
                operand2: "200".to_string()
            },
        })
    );

    // Original `lp_amount` for stake should NOT be less than `minimum_initial_deposit`.
    // Here, the mocked `minimum_initial_deposit` is 100.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: Some(1_u32),
            lp_token_bal_before: Uint128::from(50_u128),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!("Received {}, should be bigger than {}.", 100 - 50, 100),
        })
    );

    // Same as above when `stake` the LP tokens from `harvest`
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: None,
            lp_token_bal_before: Uint128::from(50_u128),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!("Received {}, should be bigger than {}.", 100 - 50, 100),
        })
    );

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
    let expected_total_share: u128 = 1000000 + 100 * 1000000 / 100;
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
    let minted_amount: u128 = 100 * 2000000 / 200;
    let expected_total_share: u128 = 2000000 + minted_amount;
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

    // Same as above tests in liquid vault
    let mut deps = create_mock_vault(
        AccountType::Liquid,
        Some("locked_sibling_vault".to_string()),
        vec![],
    );
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
}

#[test]
fn test_stake_lp_token_from_harvest() {
    // When `harvest`ing in locked vault, it does 2 actions - `harvest_to_liquid` + `restake`
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: None,
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1 + 1);

    // When `harvest`ing in liquid vault, it just does `restake`
    let mut deps = create_mock_vault(
        AccountType::Liquid,
        Some("locked_sibling_vault".to_string()),
        vec![],
    );
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Stake {
            endowment_id: None,
            lp_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_redeem_lp_token() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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
    let info = mock_info("accounts-contract", &[]);
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
    let info = mock_info("accounts-contract", &[]);
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
    // Cannot call the `reinvest_to_locked` entry of `Locked` vault contract.
    let mut deps = create_mock_vault(AccountType::Locked, None, vec![]);
    let info = mock_info("accounts-contract", &[]);
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
            msg: "This is locked vault".to_string(),
        })
    );

    // `sibling_vault` contract should be pre-set.
    let mut deps = create_mock_vault(AccountType::Liquid, None, vec![]);
    let info = mock_info("accounts-contract", &[]);
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
            msg: "Sibling vault not created".to_string(),
        })
    );

    // Mock the correct contract settings
    let mut deps = create_mock_vault(
        AccountType::Liquid,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

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
    let info = mock_info("accounts-contract", &[]);
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
    let info = mock_info("accounts-contract", &[]);
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

#[test]
fn test_reinvest_to_locked_receive() {
    // Cannot call the `reinvest_to_locked`(receive) in liquid vault contract
    let mut deps = create_mock_vault(
        AccountType::Liquid,
        Some("sibling_vault".to_string()),
        vec![],
    );

    let reinvest_to_locked_msg = cw20::Cw20ReceiveMsg {
        sender: "sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(100_u128),
        })
        .unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(reinvest_to_locked_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // LP token(cw20) should be sent for `reinvest_to_locked` call.
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    let reinvest_to_locked_msg = cw20::Cw20ReceiveMsg {
        sender: "liquid_sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(100_u128),
        })
        .unwrap(),
    };

    let info = mock_info("non-loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(reinvest_to_locked_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Sibling vault(which sent this message) should be liquid vault
    let reinvest_to_locked_msg = cw20::Cw20ReceiveMsg {
        sender: "sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(100_u128),
        })
        .unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(reinvest_to_locked_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Amount of LP tokens sent should match with message parameter.
    let reinvest_to_locked_msg = cw20::Cw20ReceiveMsg {
        sender: "liquid_sibling_vault".to_string(),
        amount: Uint128::from(50_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(100_u128),
        })
        .unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(reinvest_to_locked_msg),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Balance does not match: Received: {}, Expected: {}",
                50, 100
            ),
        })
    );

    // Amount of LP tokens sent should be sufficient for minting VT(vault token)s.
    let reinvest_to_locked_msg = cw20::Cw20ReceiveMsg {
        sender: "liquid_sibling_vault".to_string(),
        amount: Uint128::from(50_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(50_u128),
        })
        .unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(reinvest_to_locked_msg),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!("Received {}, should be bigger than {}.", 50, 100),
        })
    );

    // Succeed to `reinvest_to_locked`
    let reinvest_to_locked_msg = cw20::Cw20ReceiveMsg {
        sender: "liquid_sibling_vault".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&angel_core::messages::vault::ReceiveMsg::ReinvestToLocked {
            endowment_id: 1_u32,
            amount: Uint128::from(100_u128),
        })
        .unwrap(),
    };

    let info = mock_info("loop-lp-token", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(reinvest_to_locked_msg),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    // Check the LP token increase & VT mint amount
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.total_lp_amount, Uint128::from(100_u128).to_string());
    assert_eq!(state.total_shares, Uint128::from(1000000_u128).to_string());

    // Check the VT balance of endowment
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            endowment_id: 1_u32,
        },
    )
    .unwrap();
    let endow_vt_balance: Uint128 = from_binary(&res).unwrap();
    assert_eq!(endow_vt_balance, Uint128::from(1000000_u128));
}

#[test]
fn test_restake_claim_reward() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // Only the contract itself can call the entry
    let info = mock_info("non-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // `reward_token` computation can handle the overflow/underflow error.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: Uint128::from(200_u128),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::Overflow {
            source: OverflowError {
                operation: cosmwasm_std::OverflowOperation::Sub,
                operand1: "100".to_string(),
                operand2: "200".to_string(),
            }
        })
    );

    // If the computed `reward_token` amount is zero, then it outputs error.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: Uint128::from(100_u128),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // Successful run returns several messages
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1 + 1 + 1);
}

#[test]
fn test_add_liquidity() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // Only the contract itself can call the entry
    let info = mock_info("non-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::AddLiquidity {
            endowment_id: None,
            lp_pair_token0_bal_before: Uint128::zero(),
            lp_pair_token1_bal_before: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Successful run returns several messages
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::AddLiquidity {
            endowment_id: None,
            lp_pair_token0_bal_before: Uint128::zero(),
            lp_pair_token1_bal_before: Uint128::zero(),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 2 + 1);
}

#[test]
fn test_remove_liquidity() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // Only the contract itself can call the entry
    let info = mock_info("non-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before: Uint128::zero(),
            beneficiary: Addr::unchecked("beneficiary"),
            id: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Successful run returns several messages
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before: Uint128::zero(),
            beneficiary: Addr::unchecked("beneficiary"),
            id: None,
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1 + 1 + 1);
}

#[test]
fn test_send_asset() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // Only the contract itself can call the entry
    let info = mock_info("non-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SendAsset {
            beneficiary: Addr::unchecked("beneficiary"),
            id: None,
            native_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Successful run returns several messages
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SendAsset {
            beneficiary: Addr::unchecked("beneficiary"),
            id: None,
            native_token_bal_before: Uint128::zero(),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_swap_back() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    // Only the contract itself can call the entry
    let info = mock_info("non-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapBack {
            lp_pair_token0_bal_before: Uint128::zero(),
            lp_pair_token1_bal_before: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Successful run returns several messages
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapBack {
            lp_pair_token0_bal_before: Uint128::zero(),
            lp_pair_token1_bal_before: Uint128::zero(),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_migrate() {
    let mut deps = create_mock_vault(
        AccountType::Locked,
        Some("liquid_sibling_vault".to_string()),
        vec![],
    );

    let err = migrate(
        deps.as_mut(),
        mock_env(),
        angel_core::messages::vault::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        })
    );
}
