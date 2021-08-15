use crate::config;
use angel_portals::error::ContractError;
use angel_portals::utils::deduct_tax;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractResult, CosmosMsg, Deps, DepsMut, Env, QueryRequest, Response,
    StdResult, SubMsgExecutionResponse, Uint128, WasmMsg, WasmQuery,
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
    pub collector_contract: String,
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
    RegisterDepositToken {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
}

pub fn register_deposit_token(
    deps: DepsMut,
    _env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    match msg {
        ContractResult::Ok(subcall) => {
            let mut token_address = String::from("");
            for event in subcall.events {
                if event.ty == "instantiate_contract".to_string() {
                    for attrb in event.attributes {
                        if attrb.key == "contract_address" {
                            token_address = attrb.value;
                        }
                    }
                }
            }
            // Register the new Endowment on success Reply
            let mut config = config::read(deps.storage)?;
            if config.deposit_token != config.owner {
                return Err(ContractError::Unauthorized {});
            }
            config.deposit_token = deps.api.addr_validate(&token_address)?;
            config::store(deps.storage, &config)?;
            Ok(Response::new()
                .add_attribute("register_deposit_token", config.deposit_token.to_string()))
        }
        ContractResult::Err(_) => Err(ContractError::PortalNotCreated {}),
    }
}

pub fn deposit_stable_msg(
    deps: Deps,
    market: &Addr,
    denom: &str,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: market.to_string(),
        msg: to_binary(&HandleMsg::DepositStable {})?,
        funds: vec![deduct_tax(
            deps,
            Coin {
                denom: denom.to_string(),
                amount,
            },
        )?],
    })])
}

pub fn redeem_stable_msg(
    _deps: Deps,
    market: &Addr,
    token: &Addr,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: market.to_string(),
            amount,
            msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
        })?,
        funds: vec![],
    })])
}
