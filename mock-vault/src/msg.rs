use crate::state::EndowmentEntry;
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, Decimal256, Deps, QueryRequest, StdResult, Uint128,
    Uint256, WasmMsg, WasmQuery,
};
use cw20::{Cw20CoinVerified, Cw20ExecuteMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub moneymarket: String,
    pub yield_token: String,
    pub input_denom: String,
    pub registrar_contract: String,
    pub tax_per_block: Decimal,
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
    Deposit {},
    Redeem {
        account_addr: Addr,
    },
    Withdraw(AccountWithdrawMsg),
    Harvest {
        last_earnings_harvest: u64,
        last_harvest_fx: Option<Decimal256>,
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountWithdrawMsg {
    pub beneficiary: Addr,
    pub amount: Uint128,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

pub fn epoch_state(deps: Deps, market: &Addr) -> StdResult<EpochStateResponse> {
    let epoch_state = deps
        .querier
        .query::<EpochStateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: market.to_string(),
            msg: to_binary(&QueryMsg::EpochState {
                block_height: None,
                distributed_interest: None,
            })?,
        }))?;

    Ok(epoch_state)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    DepositStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
}

pub fn deposit_stable_msg(market: &Addr, denom: &str, amount: Uint128) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: market.to_string(),
        msg: to_binary(&HandleMsg::DepositStable {})?,
        funds: vec![Coin {
            denom: denom.to_string(),
            amount,
        }],
    }))
}

pub fn redeem_stable_msg(market: &Addr, token: &Addr, amount: Uint128) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.into(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: market.into(),
            amount,
            msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
        })?,
        funds: vec![],
    }))
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
    State {
        block_height: Option<u64>,
    },
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
    Endowment {
        endowment_addr: String,
    },
    // Gets list of all registered Endowments
    EndowmentList {
        status: Option<String>,
        name: Option<Option<String>>,
        owner: Option<Option<String>>,
        tier: Option<Option<String>>,
        un_sdg: Option<Option<u64>>,
        endow_type: Option<Option<String>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceResponse {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

impl BalanceResponse {
    pub fn default() -> Self {
        BalanceResponse {
            native: vec![],
            cw20: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub input_denom: String,
    pub yield_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    pub exchange_rate: Decimal256,
    pub yield_token_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultConfigResponse {
    pub owner: String,
    pub registrar_contract: String,
    pub moneymarket: String,
    pub input_denom: String,
    pub yield_token: String,
    pub tax_per_block: Decimal,
    pub harvest_to_liquid: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailResponse {
    pub endowment: EndowmentEntry,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct EndowmentListResponse {
    pub endowments: Vec<EndowmentEntry>,
}
