use crate::contract::{execute, instantiate, migrate, query};
use crate::testing::mock_querier::mock_dependencies;
use angel_core::errors::core::ContractError;
use angel_core::msgs::swap_router::{
    ConfigResponse,
    Cw20HookMsg,
    ExecuteMsg,
    InstantiateMsg, // JunoSwapExecuteMsg, JunoSwapQueryMsg,
    MigrateMsg,
    QueryMsg,
    SimulateSwapOperationsResponse,
};
use angel_core::structs::{AccountType, Pair, SwapOperation};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, to_binary, Addr, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;

const ACCOUNTS_CONTRACT: &str = "accounts_contract_addr";
const REGISTRAR_CONTRACT: &str = "registrar_contract_addr";
const USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const USDT: &str = "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF";

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![],
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(ACCOUNTS_CONTRACT, config.accounts_contract.as_str());
}

#[test]
fn execute_swap_operations() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![
            Pair {
                assets: [
                    AssetInfo::Native(USDC.to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ],
                contract_address: Addr::unchecked("loopswap-contract"),
            },
            Pair {
                assets: [
                    AssetInfo::Cw20(Addr::unchecked("asset0001")),
                    AssetInfo::Native("ujuno".to_string()),
                ],
                contract_address: Addr::unchecked("contract-2"),
            },
            Pair {
                assets: [
                    AssetInfo::Native("ujuno".to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0002")),
                ],
                contract_address: Addr::unchecked("junoswap-contract"),
            },
        ],
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    let msg = ExecuteMsg::ExecuteSwapOperations {
        strategy_key: None,
        operations: vec![],
        minimum_receive: None,
        endowment_id: 1,
        acct_type: AccountType::Locked,
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::MustProvideOperations {});

    let msg = ExecuteMsg::ExecuteSwapOperations {
        strategy_key: None,
        operations: vec![
            SwapOperation::Loop {
                offer_asset_info: AssetInfo::Native(USDT.to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked(USDC)),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked(USDC)),
                ask_asset_info: AssetInfo::Native("ujuno".to_string()),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
            },
        ],
        minimum_receive: Some(Uint128::from(1000000u128)),
        endowment_id: 1,
        acct_type: AccountType::Locked,
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::Loop {
                        offer_asset_info: AssetInfo::Native(USDT.to_string()),
                        ask_asset_info: AssetInfo::Cw20(Addr::unchecked(USDC)),
                    },
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::JunoSwap {
                        offer_asset_info: AssetInfo::Cw20(Addr::unchecked(USDC)),
                        ask_asset_info: AssetInfo::Native("ujuno".to_string()),
                    },
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::JunoSwap {
                        offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                        ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
                    },
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::AssertMinimumReceive {
                    asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
                    prev_balance: Uint128::from(1000000_u128),
                    minimum_receive: Uint128::from(1000000u128),
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::SendSwapReceipt {
                    asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
                    prev_balance: Uint128::from(1000000_u128),
                    endowment_id: 1,
                    acct_type: AccountType::Locked,
                    vault_addr: None,
                })
                .unwrap(),
            })),
        ]
    );

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "vault-1".into(),
        amount: Uint128::from(1000000u128),
        msg: to_binary(&Cw20HookMsg::ExecuteSwapOperations {
            strategy_key: None,
            operations: vec![
                SwapOperation::JunoSwap {
                    offer_asset_info: AssetInfo::Native(USDC.to_string()),
                    ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                },
                SwapOperation::JunoSwap {
                    offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                    ask_asset_info: AssetInfo::Native("ujuno".to_string()),
                },
                SwapOperation::Loop {
                    offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                    ask_asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
                },
            ],
            minimum_receive: None,
            endowment_id: 1,
            acct_type: AccountType::Locked,
        })
        .unwrap(),
    });

    let env = mock_env();
    let info = mock_info("asset0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::JunoSwap {
                        offer_asset_info: AssetInfo::Native(USDC.to_string()),
                        ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                    },
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::JunoSwap {
                        offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                        ask_asset_info: AssetInfo::Native("ujuno".to_string()),
                    },
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::Loop {
                        offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                        ask_asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
                    },
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::SendSwapReceipt {
                    asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
                    prev_balance: Uint128::from(1000000_u128),
                    endowment_id: 1,
                    acct_type: AccountType::Locked,
                    vault_addr: Some(Addr::unchecked("vault-1")),
                })
                .unwrap(),
            })),
        ]
    );
}

#[test]
fn test_execute_swap_operation() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![
            Pair {
                assets: [
                    AssetInfo::Native(USDC.to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ],
                contract_address: Addr::unchecked("loopswap-contract"),
            },
            Pair {
                assets: [
                    AssetInfo::Cw20(Addr::unchecked("asset0001")),
                    AssetInfo::Native("ujuno".to_string()),
                ],
                contract_address: Addr::unchecked("contract-2"),
            },
            Pair {
                assets: [
                    AssetInfo::Native("ujuno".to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0002")),
                ],
                contract_address: Addr::unchecked("junoswap-contract"),
            },
            Pair {
                assets: [
                    AssetInfo::Native(USDC.to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ],
                contract_address: Addr::unchecked("junoswap-contract"),
            },
        ],
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // This operation should be called by contract itself
    let info = mock_info("non-mock-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::ExecuteSwapOperation {
            operation: SwapOperation::Loop {
                offer_asset_info: cw_asset::AssetInfoBase::Native(USDC.to_string()),
                ask_asset_info: cw_asset::AssetInfoBase::Cw20(Addr::unchecked("asset0001")),
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to "swap_operation"
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::ExecuteSwapOperation {
            operation: SwapOperation::Loop {
                offer_asset_info: cw_asset::AssetInfoBase::Native(USDC.to_string()),
                ask_asset_info: cw_asset::AssetInfoBase::Cw20(Addr::unchecked("asset0001")),
            },
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::ExecuteSwapOperation {
            operation: SwapOperation::JunoSwap {
                offer_asset_info: cw_asset::AssetInfoBase::Native(USDC.to_string()),
                ask_asset_info: cw_asset::AssetInfoBase::Cw20(Addr::unchecked("asset0000")),
            },
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn query_buy_with_routes() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![
            Pair {
                assets: [
                    AssetInfo::Native("ujuno".to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ],
                contract_address: Addr::unchecked("contract-1"),
            },
            Pair {
                assets: [
                    AssetInfo::Native(USDC.to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ],
                contract_address: Addr::unchecked("contract-2"),
            },
        ],
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Routes should NOT be empty.
    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::SimulateSwapOperations {
            offer_amount: Uint128::from(1000000_u128),
            operations: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, StdError::generic_err("must provide operations"));

    let msg = QueryMsg::SimulateSwapOperations {
        offer_amount: Uint128::from(1000000_u128),
        operations: vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0000")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ask_asset_info: AssetInfo::Native(USDC.to_string()),
            },
        ],
    };

    let res: SimulateSwapOperationsResponse =
        from_binary(&query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();
    assert_eq!(
        res,
        SimulateSwapOperationsResponse {
            amount: Uint128::from(1000000_u128), // ujuno => usdc, usdc => asset0000
        }
    );
}

#[test]
fn assert_minimum_receive_native_token() {
    let mut deps = mock_dependencies(&[]);

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    // success
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Native("ujuno".to_string()),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000000u128),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // assertion failed; native token
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Native("ujuno".to_string()),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000001u128),
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(StdError::GenericErr {
            msg: "assertion failed; minimum receive amount: 1000001, swap amount: 1000000".into()
        })
    );
}

#[test]
fn assert_minimum_receive_token() {
    let mut deps = mock_dependencies(&[]);

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    // success
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Cw20(Addr::unchecked("token0000")),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000000u128),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // assertion failed; native token
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Cw20(Addr::unchecked("token0000")),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000001u128),
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(StdError::GenericErr {
            msg: "assertion failed; minimum receive amount: 1000001, swap amount: 1000000".into()
        })
    )
}

#[test]
fn test_update_pairs() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![],
    };
    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Only registrar_config.owner can update the pairs
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdatePairs {
            add: vec![],
            remove: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_migrate() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![],
    };
    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Try to migrate
    let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_send_swap_receipt() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked(ACCOUNTS_CONTRACT),
        registrar_contract: Addr::unchecked(REGISTRAR_CONTRACT),
        pairs: vec![
            Pair {
                assets: [
                    AssetInfo::Native(USDC.to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ],
                contract_address: Addr::unchecked("loopswap-contract"),
            },
            Pair {
                assets: [
                    AssetInfo::Cw20(Addr::unchecked("asset0001")),
                    AssetInfo::Native("ujuno".to_string()),
                ],
                contract_address: Addr::unchecked("contract-2"),
            },
            Pair {
                assets: [
                    AssetInfo::Native("ujuno".to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0002")),
                ],
                contract_address: Addr::unchecked("junoswap-contract"),
            },
            Pair {
                assets: [
                    AssetInfo::Native(USDC.to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ],
                contract_address: Addr::unchecked("junoswap-contract"),
            },
        ],
    };

    let env = mock_env();
    let info = mock_info(ACCOUNTS_CONTRACT, &[]);
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // This operation should be called by contract itself
    let info = mock_info("non-mock-contract", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SendSwapReceipt {
            asset_info: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
            prev_balance: Uint128::zero(),
            endowment_id: 1_u32,
            acct_type: AccountType::Locked,
            vault_addr: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // The "swap_amount" should NOT be zero.
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SendSwapReceipt {
            asset_info: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
            prev_balance: Uint128::from(1000000_u128),
            endowment_id: 1_u32,
            acct_type: AccountType::Locked,
            vault_addr: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::ZeroAmount {});

    // Succeed to "send_swap_receipt"
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SendSwapReceipt {
            asset_info: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
            prev_balance: Uint128::zero(),
            endowment_id: 1_u32,
            acct_type: AccountType::Locked,
            vault_addr: None,
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SendSwapReceipt {
            asset_info: cw_asset::AssetInfoBase::Native("ujuno".to_string()),
            prev_balance: Uint128::zero(),
            endowment_id: 1_u32,
            acct_type: AccountType::Locked,
            vault_addr: Some(Addr::unchecked("vault")),
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}
