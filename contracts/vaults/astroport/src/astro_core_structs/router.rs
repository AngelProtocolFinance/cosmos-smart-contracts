use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::astro_core_structs::asset::AssetInfo;

/// This enum describes a swap operation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapOperation {
    /// Native swap
    NativeSwap {
        /// The name (denomination) of the native asset to swap from
        offer_denom: String,
        /// The name (denomination) of the native asset to swap to
        ask_denom: String,
    },
    /// ASTRO swap
    AstroSwap {
        /// Information about the asset being swapped
        offer_asset_info: AssetInfo,
        /// Information about the asset we swap to
        ask_asset_info: AssetInfo,
    },
}

impl SwapOperation {
    pub fn reverse_operation(&self) -> Self {
        match self {
            SwapOperation::NativeSwap {
                offer_denom,
                ask_denom,
            } => SwapOperation::NativeSwap {
                offer_denom: ask_denom.to_string(),
                ask_denom: offer_denom.to_string(),
            },
            SwapOperation::AstroSwap {
                offer_asset_info,
                ask_asset_info,
            } => SwapOperation::AstroSwap {
                offer_asset_info: ask_asset_info.clone(),
                ask_asset_info: offer_asset_info.clone(),
            },
        }
    }
}

/// This structure describes the execute messages available in the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ExecuteSwapOperations processes multiple swaps while mentioning the minimum amount of tokens to receive for the last swap operation
    ExecuteSwapOperations {
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<String>,
        max_spread: Option<Decimal>,
    },
}
