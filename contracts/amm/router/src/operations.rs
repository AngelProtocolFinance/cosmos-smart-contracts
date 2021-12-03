use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};

use crate::error::ContractError;
use crate::querier::compute_tax;
use crate::state::{Config, CONFIG};

use cw20::Cw20ExecuteMsg;
use halo_amm::asset::{Asset, AssetInfo};
use halo_amm::factory::FactoryPairInfo;
use halo_amm::pair::ExecuteMsg as PairExecuteMsg;
use halo_amm::querier::{query_balance, query_factory_pair_info, query_token_balance};
use halo_amm::router::SwapOperation;
use terra_cosmwasm::{create_swap_msg, create_swap_send_msg, TerraMsgWrapper};

/// Execute swap operation
/// swap all offer asset to ask asset
pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation: SwapOperation,
    to: Option<Addr>,
) -> Result<Response<TerraMsgWrapper>, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let messages: Vec<CosmosMsg<TerraMsgWrapper>> = match operation {
        SwapOperation::NativeSwap {
            offer_denom,
            ask_denom,
        } => {
            let amount = query_balance(
                deps.as_ref(),
                &env.contract.address,
                offer_denom.to_string(),
            )?;
            if let Some(to) = to {
                // if the operation is last, and requires send
                // deduct tax from the offer_coin
                let amount =
                    amount.checked_sub(compute_tax(deps.as_ref(), amount, offer_denom.clone())?)?;
                vec![create_swap_send_msg(
                    to.to_string(),
                    Coin {
                        denom: offer_denom,
                        amount,
                    },
                    ask_denom,
                )]
            } else {
                vec![create_swap_msg(
                    Coin {
                        denom: offer_denom,
                        amount,
                    },
                    ask_denom,
                )]
            }
        }
        SwapOperation::HaloSwap {
            offer_asset_info,
            ask_asset_info,
        } => {
            let config: Config = CONFIG.load(deps.as_ref().storage)?;
            let pair_info: FactoryPairInfo = query_factory_pair_info(
                deps.as_ref(),
                &config.halo_factory,
                &[offer_asset_info.clone(), ask_asset_info],
            )?;

            let amount = match offer_asset_info.clone() {
                AssetInfo::NativeToken { denom } => {
                    query_balance(deps.as_ref(), &env.contract.address, denom)?
                }
                AssetInfo::Token { contract_addr } => {
                    query_token_balance(deps.as_ref(), &contract_addr, &env.contract.address)?
                }
            };
            let offer_asset: Asset = Asset {
                info: offer_asset_info,
                amount,
            };

            vec![asset_into_swap_msg(
                deps.as_ref(),
                Addr::unchecked(pair_info.contract_addr),
                offer_asset,
                None,
                to,
            )?]
        }
    };

    Ok(Response::new().add_messages(messages))
}

pub fn asset_into_swap_msg(
    deps: Deps,
    pair_contract: Addr,
    offer_asset: Asset,
    max_spread: Option<Decimal>,
    to: Option<Addr>,
) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
    match offer_asset.info.clone() {
        AssetInfo::NativeToken { denom } => {
            // deduct tax first
            let amount = offer_asset.amount.checked_sub(compute_tax(
                deps,
                offer_asset.amount,
                denom.clone(),
            )?)?;

            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_contract.to_string(),
                funds: vec![Coin { denom, amount }],
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset: Asset {
                        amount,
                        ..offer_asset
                    },
                    belief_price: None,
                    max_spread,
                    to,
                })?,
            }))
        }
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&PairExecuteMsg::Swap {
                    offer_asset,
                    belief_price: None,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}
