use crate::operations::{assert_minium_receive, execute_swap_operation, send_swap_receipt};
use crate::state::{pair_key, Config, CONFIG, PAIRS};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::QueryMsg as AccountsQueryMsg;
use angel_core::messages::dexs::{
    InfoResponse, JunoSwapQueryMsg, LoopQueryMsg, SimulationResponse, Token1ForToken2PriceResponse,
    Token2ForToken1PriceResponse,
};
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::messages::router::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    SimulateSwapOperationsResponse,
};
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse,
};
use angel_core::structs::{AccountType, Pair, SwapOperation};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, Denom};
use cw_asset::{Asset, AssetInfo};
use std::collections::HashMap;

// version info for migration info
const CONTRACT_NAME: &str = "swap-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(
        deps.storage,
        &Config {
            registrar_contract: msg.registrar_contract,
            accounts_contract: msg.accounts_contract,
        },
    )?;

    for pair in msg.pairs.iter() {
        PAIRS.save(deps.storage, &pair_key(&pair.assets), pair)?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, msg),
        ExecuteMsg::UpdatePairs { add, remove } => execute_update_pairs(deps, info, add, remove),
        ExecuteMsg::ExecuteSwapOperations {
            endowment_id,
            acct_type,
            operations,
            minimum_receive,
        } => execute_swap_operations(
            deps,
            env,
            info.sender,
            endowment_id,
            acct_type,
            operations,
            minimum_receive,
        ),
        ExecuteMsg::ExecuteSwapOperation { operation, to } => {
            execute_swap_operation(deps, env, info, operation, to)
        }
        ExecuteMsg::AssertMinimumReceive {
            asset_info,
            prev_balance,
            minimum_receive,
            receiver,
        } => assert_minium_receive(
            deps.as_ref(),
            asset_info,
            prev_balance,
            minimum_receive,
            receiver,
        ),
        ExecuteMsg::SendSwapReceipt {
            asset_info,
            prev_balance,
            endowment_id,
            acct_type,
        } => send_swap_receipt(
            deps.as_ref(),
            env,
            info,
            asset_info,
            prev_balance,
            endowment_id,
            acct_type,
        ),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(&cw20_msg.sender)?;
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::ExecuteSwapOperations {
            endowment_id,
            acct_type,
            operations,
            minimum_receive,
        } => execute_swap_operations(
            deps,
            env,
            sender,
            endowment_id,
            acct_type,
            operations,
            minimum_receive,
        ),
    }
}

pub fn execute_update_pairs(
    deps: DepsMut,
    info: MessageInfo,
    add: Vec<Pair>,
    remove: Vec<[AssetInfo; 2]>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    if info.sender != registrar_config.owner {
        return Err(ContractError::Unauthorized {});
    }

    for pair in add.iter() {
        PAIRS.save(deps.storage, &pair_key(&pair.assets), pair)?;
    }
    for assets in remove.iter() {
        PAIRS.remove(deps.storage, &pair_key(assets));
    }
    Ok(Response::new())
}

pub fn execute_swap_operations(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    endowment_id: u32,
    acct_type: AccountType,
    operations: Vec<SwapOperation>,
    minimum_receive: Option<Uint128>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // Swaps are restricted to the Accounts contract (endowments) & approved Vault contracts
    if sender != config.accounts_contract {
        // check that the deposit token came from an approved Vault SC
        let vault_res: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: sender.to_string(),
                })?,
            }))?;
        if !vault_res.vault.approved {
            return Err(ContractError::Unauthorized {});
        }
    }

    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(ContractError::MustProvideOperations {});
    }

    // Assert the operations are properly set
    assert_operations(&operations)?;

    let target_asset_info = operations.last().unwrap().get_ask_asset_info();

    let mut operation_index = 0;
    let mut messages: Vec<CosmosMsg> = operations
        .into_iter()
        .map(|op| {
            operation_index += 1;
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: op,
                    to: if operation_index == operations_len {
                        Some(sender.clone())
                    } else {
                        None
                    },
                })?,
                funds: vec![],
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    // Execute minimum amount assertion
    if let Some(minimum_receive) = minimum_receive {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::AssertMinimumReceive {
                asset_info: target_asset_info.clone(),
                prev_balance: target_asset_info.query_balance(&deps.querier, &sender)?,
                minimum_receive,
                receiver: sender,
            })?,
        }));
    }

    // Send a Swap Receipt message back to sender as the final message
    let prev_balance: Uint128 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.accounts_contract.to_string(),
        msg: to_binary(&AccountsQueryMsg::TokenAmount {
            id: endowment_id,
            asset_info: target_asset_info.clone(),
            acct_type: acct_type.clone(),
        })?,
    }))?;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SendSwapReceipt {
            asset_info: target_asset_info,
            prev_balance,
            endowment_id,
            acct_type,
        })?,
    }));

    Ok(Response::new().add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::SimulateSwapOperations {
            offer_amount,
            operations,
        } => to_binary(&simulate_swap_operations(deps, offer_amount, operations)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage).unwrap();
    Ok(ConfigResponse {
        registrar_contract: config.registrar_contract,
        accounts_contract: config.accounts_contract,
    })
}

fn simulate_swap_operations(
    deps: Deps,
    offer_amount: Uint128,
    operations: Vec<SwapOperation>,
) -> StdResult<SimulateSwapOperationsResponse> {
    if operations.is_empty() {
        return Err(StdError::generic_err("must provide operations"));
    }

    assert_operations(&operations)?;
    assert_operations_order(&operations)?;

    let mut offer_amount = offer_amount;
    for operation in operations.into_iter() {
        match operation {
            SwapOperation::JunoSwap {
                offer_asset_info,
                ask_asset_info,
            } => {
                let pair: Pair = PAIRS.load(
                    deps.storage,
                    &pair_key(&[offer_asset_info.clone(), ask_asset_info]),
                )?;
                let pair_info: InfoResponse =
                    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: pair.contract_address.to_string(),
                        msg: to_binary(&JunoSwapQueryMsg::Info {})?,
                    }))?;

                let offer_addr = offer_asset_info.to_string();
                let token1_denom = match pair_info.token1_denom {
                    Denom::Native(denom) => denom,
                    Denom::Cw20(addr) => addr.to_string(),
                };
                let token2_denom = match pair_info.token2_denom {
                    Denom::Native(denom) => denom,
                    Denom::Cw20(addr) => addr.to_string(),
                };
                if token1_denom == offer_addr {
                    let res: Token1ForToken2PriceResponse =
                        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: pair.contract_address.to_string(),
                            msg: to_binary(&JunoSwapQueryMsg::Token1ForToken2Price {
                                token1_amount: offer_amount,
                            })?,
                        }))?;
                    offer_amount = res.token2_amount;
                } else if token2_denom == offer_addr {
                    let res: Token2ForToken1PriceResponse =
                        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: pair.contract_address.to_string(),
                            msg: to_binary(&JunoSwapQueryMsg::Token2ForToken1Price {
                                token2_amount: offer_amount,
                            })?,
                        }))?;
                    offer_amount = res.token1_amount;
                }
            }
            SwapOperation::Loop {
                offer_asset_info,
                ask_asset_info,
            } => {
                let pair: Pair = PAIRS.load(
                    deps.storage,
                    &pair_key(&[offer_asset_info.clone(), ask_asset_info]),
                )?;
                let res: SimulationResponse =
                    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: pair.contract_address.to_string(),
                        msg: to_binary(&LoopQueryMsg::Simulation {
                            offer_asset: Asset {
                                info: offer_asset_info,
                                amount: offer_amount,
                            },
                        })?,
                    }))?;
                offer_amount = res.return_amount;
            }
        }
    }

    Ok(SimulateSwapOperationsResponse {
        amount: offer_amount,
    })
}

fn assert_operations_order(operations: &[SwapOperation]) -> StdResult<()> {
    let mut prev_ask = String::new();

    for operation in operations.iter() {
        let offer_asset = operation.get_offer_asset_info();
        let ask_asset = operation.get_ask_asset_info();

        if !prev_ask.is_empty() && prev_ask != offer_asset.to_string() {
            return Err(StdError::generic_err(
                "invalid operations order; offer does not equal to prev ask",
            ));
        }

        prev_ask = ask_asset.to_string()
    }

    Ok(())
}

fn assert_operations(operations: &[SwapOperation]) -> StdResult<()> {
    let mut ask_asset_map: HashMap<String, bool> = HashMap::new();

    for operation in operations.iter() {
        let offer_asset = operation.get_offer_asset_info();
        let ask_asset = operation.get_ask_asset_info();

        ask_asset_map.remove(&offer_asset.to_string());
        ask_asset_map.insert(ask_asset.to_string(), true);
    }

    if ask_asset_map.keys().len() != 1 {
        return Err(StdError::generic_err(
            "invalid operations; multiple output token",
        ));
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[test]
fn test_invalid_operations() {
    // empty error
    assert_eq!(true, assert_operations(&vec![]).is_err());

    // ujuno output
    assert_eq!(
        true,
        assert_operations(&vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("usdc".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ask_asset_info: AssetInfo::Native("ujuno".to_string()),
            }
        ])
        .is_ok()
    );

    // asset0002 output
    assert_eq!(
        true,
        assert_operations(&vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("usdc".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ask_asset_info: AssetInfo::Native("ujuno".to_string()),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
            },
        ])
        .is_ok()
    );

    // multiple output token types error
    assert_eq!(
        true,
        assert_operations(&vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("usdc".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ask_asset_info: AssetInfo::Native("uaud".to_string()),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
            },
        ])
        .is_err()
    );
}

#[test]
fn test_invalid_operations_order() {
    assert_eq!(
        true,
        assert_operations_order(&vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ask_asset_info: AssetInfo::Native("ujuno".to_string()),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("ujuno".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0002")),
            },
        ])
        .is_ok()
    );

    assert_eq!(
        true,
        assert_operations_order(&vec![
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native("usdc".to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
            },
            SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Cw20(Addr::unchecked("asset0001")),
                ask_asset_info: AssetInfo::Native("ujuno".to_string()),
            }
        ])
        .is_err()
    );
}
