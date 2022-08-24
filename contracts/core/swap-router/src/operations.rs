use crate::state::{pair_key, CONFIG, PAIRS};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::ExecuteMsg as AccountsExecuteMsg;
use angel_core::messages::accounts::QueryMsg as AccountsQueryMsg;
use angel_core::messages::dexs::{
    InfoResponse, JunoSwapExecuteMsg, JunoSwapQueryMsg, LoopExecuteMsg, TokenSelect,
};
use angel_core::structs::{AccountType, Pair, SwapOperation};
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdError, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Cw20ExecuteMsg, Denom};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

pub fn send_swap_receipt(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    asset_info: AssetInfo,
    prev_balance: Uint128,
    endowment_id: u32,
    acct_type: AccountType,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let config = CONFIG.load(deps.storage)?;
    let receiver_balance: Uint128 = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.accounts_contract.to_string(),
        msg: to_binary(&AccountsQueryMsg::TokenAmount {
            id: endowment_id,
            asset_info: asset_info.clone(),
            acct_type: acct_type.clone(),
        })?,
    }))?;
    let swap_amount = receiver_balance.checked_sub(prev_balance)?;
    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.accounts_contract.to_string(),
        msg: to_binary(&AccountsExecuteMsg::SwapReceipt {
            id: endowment_id,
            acct_type,
            final_asset: Asset {
                info: asset_info,
                amount: swap_amount,
            },
        })?,
        funds: vec![],
    });

    Ok(Response::new().add_message(message))
}

pub fn assert_minium_receive(
    deps: Deps,
    asset_info: AssetInfo,
    prev_balance: Uint128,
    minimum_receive: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let receiver_balance = asset_info.query_balance(&deps.querier, &receiver)?;
    let swap_amount = receiver_balance.checked_sub(prev_balance)?;

    if swap_amount < minimum_receive {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "assertion failed; minimum receive amount: {}, swap amount: {}",
            minimum_receive, swap_amount
        ))));
    }

    Ok(Response::default())
}

/// Execute swap operation
/// swap all offer asset to ask asset
pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation: SwapOperation,
    _to: Option<Addr>,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let offer_asset: Asset;
    let pair: Pair;
    let binary_msg = match operation {
        SwapOperation::JunoSwap {
            offer_asset_info,
            ask_asset_info,
        } => {
            let amount = match offer_asset_info.clone() {
                AssetInfo::Native(denom) => AssetInfo::Native(denom)
                    .query_balance(&deps.querier, env.contract.address.to_string())?,
                AssetInfo::Cw20(contract_addr) => AssetInfoBase::Cw20(contract_addr)
                    .query_balance(&deps.querier, env.contract.address.to_string())?,
                _ => Uint128::zero(),
            };

            if amount.is_zero() {
                return Err(ContractError::InvalidInputs {});
            }

            pair = PAIRS.load(
                deps.storage,
                &pair_key(&[offer_asset_info.clone(), ask_asset_info]),
            )?;
            let pair_info: InfoResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: pair.contract_address.to_string(),
                    msg: to_binary(&JunoSwapQueryMsg::Info {})?,
                }))?;

            offer_asset = Asset {
                info: offer_asset_info.clone(),
                amount,
            };

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
                to_binary(&JunoSwapExecuteMsg::Swap {
                    input_token: TokenSelect::Token1,
                    input_amount: amount,
                    min_output: Uint128::zero(),
                    expiration: None,
                })?
            } else if token2_denom == offer_addr {
                to_binary(&JunoSwapExecuteMsg::Swap {
                    input_token: TokenSelect::Token2,
                    input_amount: amount,
                    min_output: Uint128::zero(),
                    expiration: None,
                })?
            } else {
                return Err(ContractError::InvalidInputs {});
            }
        }
        SwapOperation::Loop {
            offer_asset_info,
            ask_asset_info,
        } => {
            let amount = match offer_asset_info.clone() {
                AssetInfo::Native(denom) => AssetInfo::Native(denom)
                    .query_balance(&deps.querier, env.contract.address.to_string())?,
                AssetInfo::Cw20(contract_addr) => AssetInfoBase::Cw20(contract_addr)
                    .query_balance(&deps.querier, env.contract.address.to_string())?,
                _ => Uint128::zero(),
            };

            if amount.is_zero() {
                return Err(ContractError::InvalidInputs {});
            }

            pair = PAIRS.load(
                deps.storage,
                &pair_key(&[offer_asset_info.clone(), ask_asset_info]),
            )?;

            offer_asset = Asset {
                info: offer_asset_info.clone(),
                amount,
            };

            to_binary(&LoopExecuteMsg::Swap {
                offer_asset: offer_asset.clone(),
                belief_price: None,
                max_spread: None,
                to: None,
            })?
        }
    };
    let messages: Vec<CosmosMsg> = match offer_asset.info {
        AssetInfo::Native(ref denom) => vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair.contract_address.to_string(),
            funds: vec![Coin {
                denom: denom.to_string(),
                amount: offer_asset.amount,
            }],
            msg: binary_msg,
        })],
        AssetInfo::Cw20(ref contract_addr) => vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair.contract_address.to_string(),
                amount: offer_asset.amount,
                msg: binary_msg,
            })?,
        })],
        _ => vec![],
    };

    Ok(Response::new().add_messages(messages))
}
