///
/// This file is a clone of some `msg` types defined for `cw20-stake` contract
/// Ref: https://github.com/DA0-DA0/dao-contracts/blob/main/contracts/cw20-stake/src/msg.rs
///
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Unstake { amount: Uint128 },
    Claim {},
    AddHook { addr: String },
    RemoveHook { addr: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Stake {},
    Fund {},
}
