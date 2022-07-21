use cw_asset::{Asset, AssetInfo, AssetInfoBase};
/// This file is just the clone of `wasmswap` messages.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};

use cw20::{Denom, Expiration};

use crate::config::Config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token1_denom: Denom,
    pub token2_denom: Denom,
    pub lp_token_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenSelect {
    Token1,
    Token2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddLiquidity {
        token1_amount: Uint128,
        min_liquidity: Uint128,
        max_token2: Uint128,
        expiration: Option<Expiration>,
    },
    RemoveLiquidity {
        amount: Uint128,
        min_token1: Uint128,
        min_token2: Uint128,
        expiration: Option<Expiration>,
    },
    Swap {
        input_token: TokenSelect,
        input_amount: Uint128,
        min_output: Uint128,
        expiration: Option<Expiration>,
    },
    /// Chained swap converting A -> B and B -> C by leveraging two swap contracts
    PassThroughSwap {
        output_amm_address: String,
        input_token: TokenSelect,
        input_token_amount: Uint128,
        output_min_token: Uint128,
        expiration: Option<Expiration>,
    },
    SwapAndSendTo {
        input_token: TokenSelect,
        input_amount: Uint128,
        recipient: String,
        min_token: Uint128,
        expiration: Option<Expiration>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance {
        address: String,
    },
    Info {},
    Token1ForToken2Price {
        token1_amount: Uint128,
    },
    Token2ForToken1Price {
        token2_amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub token1_reserve: Uint128,
    pub token1_denom: Denom,
    pub token2_reserve: Uint128,
    pub token2_denom: Denom,
    pub lp_token_supply: Uint128,
    pub lp_token_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token1ForToken2PriceResponse {
    pub token2_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token2ForToken1PriceResponse {
    pub token1_amount: Uint128,
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum HandleMsg {
//     DepositStable {},
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum Cw20HookMsg {
//     /// Return stable coins to a user
//     /// according to exchange rate
//     RedeemStable {},
// }

pub fn swap_msg(config: &Config, asset: Asset) -> StdResult<Vec<CosmosMsg>> {
    match asset.info {
        AssetInfoBase::Native(denom) => {
            let input_token_string = denom.to_string();
            let input_token = if input_token_string == config.input_denoms[0] {
                TokenSelect::Token1
            } else {
                TokenSelect::Token2
            };
            Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.target.to_string(),
                msg: to_binary(&ExecuteMsg::Swap {
                    input_token,
                    input_amount: asset.amount,
                    min_output: Uint128::zero(), // Here, we set the zero temporarily. Need to be fixed afterwards.
                    expiration: None,
                })?,
                funds: vec![Coin {
                    denom: input_token_string,
                    amount: asset.amount,
                }],
            })])
        }
        AssetInfoBase::Cw20(addr) => {
            let input_token_string = addr.to_string();
            let input_token = if input_token_string == config.input_denoms[0] {
                TokenSelect::Token1
            } else {
                TokenSelect::Token2
            };
            Ok(vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                        spender: config.target.to_string(),
                        amount: asset.amount,
                        expires: None,
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.target.to_string(),
                    msg: to_binary(&ExecuteMsg::Swap {
                        input_token,
                        input_amount: asset.amount,
                        min_output: Uint128::zero(), // Here, we set the zero temporarily. Need to be fixed afterwards.
                        expiration: None,
                    })?,
                    funds: vec![],
                }),
            ])
        }
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
    }
}

// pub fn redeem_stable_msg(market: &Addr, token: &Addr, amount: Uint128) -> StdResult<CosmosMsg> {
//     Ok(CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: token.into(),
//         msg: to_binary(&Cw20ExecuteMsg::Send {
//             contract: market.into(),
//             amount,
//             msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
//         })?,
//         funds: vec![],
//     }))
// }
