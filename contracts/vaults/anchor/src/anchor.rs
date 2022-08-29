use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal256, Deps, QueryRequest, StdResult, Uint128, Uint256,
    WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {
        block_height: Option<u64>,
    },
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub aterra_contract: String,
    pub interest_model: String,
    pub distribution_model: String,
    pub overseer_contract: String,
    pub distributor_contract: String,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
}

pub fn config(deps: Deps, market: &Addr) -> StdResult<ConfigResponse> {
    let market_config =
        deps.querier
            .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: market.to_string(),
                msg: to_binary(&QueryMsg::Config {})?,
            }))?;

    Ok(market_config)
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
    /// reinvest vault assets (ex. LPs) from self (if AccountType::Liquid)
    /// over to it's AccountType::Locked (sibling) vault
    ReinvestToLocked { id: u32, amount: Uint128 },
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
