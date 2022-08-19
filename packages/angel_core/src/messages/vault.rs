use cosmwasm_std::{Addr, Binary, Decimal, Uint128};
use cw20::{Cw20ReceiveMsg, Denom};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub swap_pool_addr: String,
    pub staking_addr: String,
    pub output_token_denom: Denom,

    pub registrar_contract: String,
    pub keeper: String,

    pub name: String,
    pub symbol: String,
    pub decimals: u8,

    pub harvest_to_liquid: Decimal,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        new_owner: String,
    },
    UpdateRegistrar {
        new_registrar: Addr,
    },
    UpdateConfig(UpdateConfigMsg),
    Deposit {
        endowment_id: u32,
    },
    Claim {
        endowment_id: u32,
        beneficiary: Addr,
    },
    Redeem {
        endowment_id: u32,
    },
    Withdraw(AccountWithdrawMsg),
    Harvest {},
    HarvestSwap {
        token1_denom_bal_before: Uint128,
        token2_denom_bal_before: Uint128,
    },
    DistributeHarvest {
        output_token_bal_before: Uint128,
    },
    AddLiquidity {
        endowment_id: u32,
        in_denom: Denom,
        out_denom: Denom,
        in_denom_bal_before: Uint128,
        out_denom_bal_before: Uint128,
    },
    RemoveLiquidity {
        lp_token_bal_before: Uint128,
        action: RemoveLiquidAction,
    },
    Stake {
        endowment_id: u32,
        lp_token_bal_before: Uint128,
    },
    SwapAndSendTo {
        token1_denom_bal_before: Uint128,
        token2_denom_bal_before: Uint128,
        beneficiary: Addr,
    },
    Receive(Cw20ReceiveMsg),
    // // Cw20_base entries
    // Transfer {
    //     recipient: String,
    //     amount: Uint128,
    // },
    // Send {
    //     contract: String,
    //     amount: Uint128,
    //     msg: Binary,
    // },
    // IncreaseAllowance {
    //     spender: String,
    //     amount: Uint128,
    //     expires: Option<Expiration>,
    // },
    // DecreaseAllowance {
    //     spender: String,
    //     amount: Uint128,
    //     expires: Option<Expiration>,
    // },
    // TransferFrom {
    //     owner: String,
    //     recipient: String,
    //     amount: Uint128,
    // },
    // BurnFrom {
    //     owner: String,
    //     amount: Uint128,
    // },
    // SendFrom {
    //     owner: String,
    //     contract: String,
    //     amount: Uint128,
    //     msg: Binary,
    // },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub swap_pool_addr: Option<String>,
    pub staking_addr: Option<String>,
    pub routes: RoutesUpdateMsg,
    pub output_token_denom: Option<Denom>,
    pub keeper: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountWithdrawMsg {
    pub endowment_id: u32,
    pub beneficiary: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoutesUpdateMsg {
    pub add: Vec<Addr>,
    pub remove: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Deposit { endowment_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the configuration of the contract
    /// Return type: ConfigResponse.
    Config {},
    /// Returns the current balance of the given "Endowment ID", 0 if unset.
    /// Return type: BalanceResponse.
    Balance { id: u32 },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
    /// Returns the total balance/total_share of the contract
    /// Return type: BalanceResponse.
    TotalBalance {},
}

///
/// The following messages are just a clone of `msg` types defined in `wasmswap-contracts`.
/// Ref: https://github.com/Wasmswap/wasmswap-contracts/blob/main/src/msg.rs
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenSelect {
    Token1,
    Token2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmSwapExecuteMsg {
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
pub enum WasmSwapQueryMsg {
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
pub enum RemoveLiquidAction {
    Harvest,
    Claim { beneficiary: Addr },
    Withdraw { beneficiary: Addr },
}
