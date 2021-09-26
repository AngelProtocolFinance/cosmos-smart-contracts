use cosmwasm_std::{Addr, Decimal, Uint128};
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
    UpdateOwner { new_owner: String },
    UpdateRegistrar { new_registrar: Addr },
    UpdateConfig(UpdateConfigMsg),
    Deposit(AccountTransferMsg),
    Redeem {},
    Withdraw(AccountWithdrawMsg),
    Harvest {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub moneymarket: String,
    pub input_denom: String,
    pub yield_token: String,
    pub tax_per_block: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountTransferMsg {
    pub locked: Uint128,
    pub liquid: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountWithdrawMsg {
    pub beneficiary: Addr,
    pub locked: Uint128,
    pub liquid: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    ExchangeRate {
        input_denom: String,
    },
    Deposit {
        amount: Uint128,
    }, // some qty of "input_denom"
    Redeem {
        amount: Uint128,
    }, // some qty of "yield_token"
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    Balance {
        address: String,
    },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
    // Only with "enumerable" extension
    // Returns all accounts that have balances. Supports pagination.
    // Return type: AllAccountsResponse.
    // AllAccounts {
    //     start_after: Option<String>,
    //     limit: Option<u32>,
    // },
}
