use crate::structs::{AccountType, Pair, SwapOperation};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: Addr,
    pub accounts_contract: Addr,
    pub pairs: Vec<Pair>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// Add/Remove Pairs
    UpdatePairs {
        add: Vec<Pair>,
        remove: Vec<[AssetInfo; 2]>,
    },
    /// Execute multiple BuyOperation
    ExecuteSwapOperations {
        endowment_id: u32,
        acct_type: AccountType,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
    },
    /// Internal use
    /// Swap all offer tokens to ask token
    ExecuteSwapOperation {
        operation: SwapOperation,
        to: Option<Addr>,
    },
    /// Internal use
    /// Check the swap amount is exceed minimum_receive
    AssertMinimumReceive {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
        receiver: Addr,
    },
    /// Send a Swap Receipt message back to the original contract
    /// Used by Accounts to properly credit the Endowment with
    /// newly swapped asset in either involved Balance
    SendSwapReceipt {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        endowment_id: u32,
        acct_type: AccountType,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    ExecuteSwapOperations {
        endowment_id: u32,
        acct_type: AccountType,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    SimulateSwapOperations {
        offer_amount: Uint128,
        operations: Vec<SwapOperation>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub registrar_contract: Addr,
    pub accounts_contract: Addr,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateSwapOperationsResponse {
    pub amount: Uint128,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}