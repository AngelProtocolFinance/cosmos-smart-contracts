use crate::structs::{AccountType, Pair, SwapOperation};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub registrar_contract: Addr,
    pub accounts_contract: Addr,
    pub pairs: Vec<Pair>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// Add/Remove Pairs
    UpdatePairs {
        add: Vec<Pair>,
        remove: Vec<[AssetInfo; 2]>,
    },
    /// Execute multiple BuyOperation
    /// NOTE: There are 2 contracts which are able to call this entry: `accounts` and `vault`.
    ///       `endowmnent_id` & `acct_type` fields are only used when `accounts` contract call.
    ///       When calling from `vault` contract, `endowment_id` & `acct_type` are meaningless and
    ///       filled with random value(Mostly, `endowment_id`: 1, `acct_type`: AccountType::Locked).
    ExecuteSwapOperations {
        endowment_id: u32,
        acct_type: AccountType,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        strategy_key: Option<String>,
    },
    /// Internal use
    /// Swap all offer tokens to ask token
    ExecuteSwapOperation {
        operation: SwapOperation,
    },
    /// Internal use
    /// Check the swap amount is exceed minimum_receive
    AssertMinimumReceive {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
    },
    /// Send a Swap Receipt message back to the original contract
    /// Used by Accounts to properly credit the Endowment with
    /// newly swapped asset in either involved Balance
    SendSwapReceipt {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        endowment_id: u32,
        acct_type: AccountType,
        vault_addr: Option<Addr>,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    ExecuteSwapOperations {
        endowment_id: u32,
        acct_type: AccountType,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        strategy_key: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(SimulateSwapOperationsResponse)]
    SimulateSwapOperations {
        offer_amount: Uint128,
        operations: Vec<SwapOperation>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub registrar_contract: Addr,
    pub accounts_contract: Addr,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct SimulateSwapOperationsResponse {
    pub amount: Uint128,
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}
