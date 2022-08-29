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
    Redeem {
        endowment_id: u32,
        amount: Uint128, // vault tokens to be burned
    },
    /// reinvest vault assets from self (if AccountType::Liquid)
    /// over to it's AccountType::Locked (sibling) vault
    ReinvestToLocked {
        endowment_id: u32,
        amount: Uint128,
    },
    Withdraw(AccountWithdrawMsg),
    Harvest {
        collector_address: String,
        collector_share: Decimal,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub moneymarket: Option<String>,
    pub input_denom: Option<String>,
    pub yield_token: Option<String>,
    pub tax_per_block: Option<Decimal>,
    pub treasury_withdraw_threshold: Option<Uint128>,
    pub harvest_to_liquid: Option<Decimal>,
    pub sibling_vault: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountWithdrawMsg {
    pub endowment_id: u32,
    pub beneficiary: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    VaultConfig {},
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
        endowment_id: u32,
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
