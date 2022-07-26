use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::{Cw20ReceiveMsg, Denom};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
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
    Deposit {},
    Redeem {
        account_addr: Addr,
    },
    Withdraw(AccountWithdrawMsg),
    Harvest {
        collector_address: String,
        collector_share: Decimal,
    },
    AddLiquidity {
        in_denom: Denom,
        out_denom: Denom,
        in_denom_bal_before: Uint128,
        out_denom_bal_before: Uint128,
    },
    Stake {
        lp_token_bal_before: Uint128,
    },
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub swap_pool_addr: Option<String>,
    pub staking_addr: Option<String>,
    pub routes: RoutesUpdateMsg,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountWithdrawMsg {
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
    Deposit {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    Balance {
        address: String,
    },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
}
