use crate::contract::{execute, instantiate, query};
use crate::testing::mock_querier::mock_dependencies;
use angel_core::errors::core::{ContractError, PaymentError};
use angel_core::messages::router::{
    ConfigResponse,
    Cw20HookMsg,
    ExecuteMsg,
    InstantiateMsg, // JunoSwapExecuteMsg, JunoSwapQueryMsg,
    QueryMsg,
    SimulateSwapOperationsResponse,
};
use angel_core::structs::{AccountType, Pair, SwapOperation};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, to_binary, Addr, CosmosMsg, StdError, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked("apaccountscontract"),
        registrar_contract: Addr::unchecked("apregistrarcontract"),
        pairs: vec![],
    };

    let env = mock_env();
    let info = mock_info("apaccountscontract", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("apaccountscontract", config.accounts_contract.as_str());
}

#[test]
fn execute_swap_operations() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked("apaccountscontract"),
        registrar_contract: Addr::unchecked("apregistrarcontract"),
        pairs: vec![
            Pair {
                assets: [
                    AssetInfo::Native("usdc".to_string()),
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
    let info = mock_info("apaccountscontract", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    let msg = ExecuteMsg::ExecuteSwapOperations {
        operations: vec![],
        minimum_receive: None,
        endowment_id: 1,
        acct_type: AccountType::Locked,
    };

    let env = mock_env();
    let info = mock_info("apaccountscontract", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::MustProvideOperations {});

    let msg = ExecuteMsg::ExecuteSwapOperations {
        operations: vec![
            SwapOperation::Loop {
                offer_asset_info: AssetInfo::Native("usdt".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("usdc")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("usdc")),
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
    let info = mock_info("apaccountscontract", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::Loop {
                        offer_asset_info: AssetInfo::Native("usdt".to_string()),
                        ask_asset_info: AssetInfo::Cw20(Addr::unchecked("usdc")),
                    },
                    to: None,
                })
                .unwrap(),
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.into(),
                funds: vec![],
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: SwapOperation::JunoSwap {
                        offer_asset_info: AssetInfo::Cw20(Addr::unchecked("usdc")),
                        ask_asset_info: AssetInfo::Native("ujuno".to_string()),
                    },
                    to: None,
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
                    to: Some(Addr::unchecked("apaccountscontract")),
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
                    receiver: Addr::unchecked("apaccountscontract"),
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
                })
                .unwrap(),
            })),
        ]
    );

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "vault-1".into(),
        amount: Uint128::from(1000000u128),
        msg: to_binary(&Cw20HookMsg::ExecuteSwapOperations {
            operations: vec![
                SwapOperation::JunoSwap {
                    offer_asset_info: AssetInfo::Native("usdc".to_string()),
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
                        offer_asset_info: AssetInfo::Native("usdc".to_string()),
                        ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                    },
                    to: None,
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
                    to: None,
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
                    to: Some(Addr::unchecked("vault-1")),
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
                })
                .unwrap(),
            })),
        ]
    );
}

#[test]
fn query_buy_with_routes() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        accounts_contract: Addr::unchecked("apaccountscontract"),
        registrar_contract: Addr::unchecked("apregistrarcontract"),
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
                    AssetInfo::Native("usdc".to_string()),
                    AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ],
                contract_address: Addr::unchecked("contract-2"),
            },
        ],
    };

    let env = mock_env();
    let info = mock_info("apaccountscontract", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = QueryMsg::SimulateSwapOperations {
        offer_amount: Uint128::from(1000000_u128),
        operations: vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0000")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0000")),
                ask_asset_info: AssetInfo::Native("usdc".to_string()),
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
    let info = mock_info("apaccountscontract", &[]);
    // success
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Native("ujuno".to_string()),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000000u128),
        receiver: Addr::unchecked("addr0000"),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // assertion failed; native token
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Native("ujuno".to_string()),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000001u128),
        receiver: Addr::unchecked("addr0000"),
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
    let info = mock_info("apaccountscontract", &[]);
    // success
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Cw20(Addr::unchecked("token0000")),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000000u128),
        receiver: Addr::unchecked("addr0000"),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // assertion failed; native token
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Cw20(Addr::unchecked("token0000")),
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000001u128),
        receiver: Addr::unchecked("addr0000"),
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(StdError::GenericErr {
            msg: "assertion failed; minimum receive amount: 1000001, swap amount: 1000000".into()
        })
    )
}
