use crate::state::CONFIG;
use angel_core::errors::core::ContractError;
use angel_core::msgs::registrar::{
    ConfigResponse as RegistrarConfigResponse, QueryMsg as RegistrarQuerier, StrategyDetailResponse,
};
// use angel_core::msgs::registrar::QueryMsg as RegistrarQuerier;
use angel_core::structs::{StrategyApprovalState, StrategyParams, VaultActionData};
use angel_core::utils::validate_deposit_fund;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, MessageInfo, QueryRequest, Response, StdError, SubMsg,
    Uint128, WasmMsg, WasmQuery,
};
use cw_asset::Asset;

pub fn execute_invest(
    deps: DepsMut,
    _sender: Addr,
    action: VaultActionData,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    // check the router should be the final handler for this action
    if action.destination_chain != registrar_config.axelar_chain_id {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Vault Router can only handle messages bound for {}",
                action.destination_chain
            ),
        }));
    }

    // Check if the passed token is in "accepted_tokens"
    let deposit_token = validate_deposit_fund(
        deps.as_ref(),
        config.registrar_contract.as_str(),
        fund.clone(),
    )?;

    // make sure it's a non-zero amount
    if deposit_token.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // validate the action amounts data (against total funds passed)
    if !action.validate_amounts(fund.amount) {
        return Err(ContractError::InvalidInputs {});
    }

    let strategy_res: StrategyDetailResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Strategy {
                strategy_key: action.strategy_id,
            })?,
        }))?;
    let strategy: StrategyParams = strategy_res.strategy;

    if strategy.approval_state != StrategyApprovalState::Approved {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Strategy is not approved".to_string(),
        }));
    }

    Ok(Response::default().add_attribute("action", "invest"))
}

pub fn execute_redeem(
    deps: DepsMut,
    _sender: Addr,
    action: VaultActionData,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    // check if the router should re-route onward to Axelar Gateway or to local Accounts
    if action.destination_chain == registrar_config.axelar_chain_id {
        let strategy_res: StrategyDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Strategy {
                    strategy_key: action.strategy_id,
                })?,
            }))?;
        let strategy: StrategyParams = strategy_res.strategy;

        if strategy.approval_state != StrategyApprovalState::Approved
            || strategy.approval_state != StrategyApprovalState::WithdrawOnly
        {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Withdraws are not allowed from this Strategy".to_string(),
            }));
        }

        let mut res = Response::default().add_attribute("action", "redeem");
        // Check the vault token(VT) balances and add redeem messages to the final response
        if action.lock_amt > Uint128::zero() {
            let available_locked: Uint128 = deps.querier.query_wasm_smart(
                strategy.locked_addr.clone().unwrap().to_string(),
                &angel_core::msgs::vault::QueryMsg::Balance {
                    endowment_id: action.account_ids[0],
                },
            )?;
            if action.lock_amt > available_locked {
                return Err(ContractError::BalanceTooSmall {});
            }
            res = res.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: strategy.locked_addr.unwrap().to_string(),
                msg: to_binary(&angel_core::msgs::vault::ExecuteMsg::Redeem {
                    endowment_id: action.account_ids[0],
                    amount: action.lock_amt,
                })
                .unwrap(),
                funds: vec![],
            })));
        }

        if action.liq_amt > Uint128::zero() {
            let available_liquid: Uint128 = deps.querier.query_wasm_smart(
                strategy.liquid_addr.clone().unwrap().to_string(),
                &angel_core::msgs::vault::QueryMsg::Balance {
                    endowment_id: action.account_ids[0],
                },
            )?;
            if action.liq_amt > available_liquid {
                return Err(ContractError::BalanceTooSmall {});
            }
            res = res.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: strategy.liquid_addr.unwrap().to_string(),
                msg: to_binary(&angel_core::msgs::vault::ExecuteMsg::Redeem {
                    endowment_id: action.account_ids[0],
                    amount: action.liq_amt,
                })
                .unwrap(),
                funds: vec![],
            })));
        }

        Ok(res)
    } else {
        Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Vault Router can only handle messages bound for {}",
                action.destination_chain
            ),
        }))
    }
}

pub fn execute_redeem_all(
    deps: DepsMut,
    _sender: Addr,
    action: VaultActionData,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    // check if the router should re-route onward to Axelar Gateway or to local Accounts
    if action.destination_chain == registrar_config.axelar_chain_id {
        let strategy_res: StrategyDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Strategy {
                    strategy_key: action.strategy_id,
                })?,
            }))?;
        let strategy: StrategyParams = strategy_res.strategy;

        if strategy.approval_state != StrategyApprovalState::Approved
            || strategy.approval_state != StrategyApprovalState::WithdrawOnly
        {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Withdraws are not allowed from this Strategy".to_string(),
            }));
        }

        let mut res = Response::default().add_attribute("action", "redeem_all");
        // Check the vault token(VT) balances and add redeem messages to the final response
        if action.lock_amt > Uint128::zero() {
            let available_locked: Uint128 = deps.querier.query_wasm_smart(
                strategy.locked_addr.clone().unwrap().to_string(),
                &angel_core::msgs::vault::QueryMsg::Balance {
                    endowment_id: action.account_ids[0],
                },
            )?;
            if !available_locked.is_zero() {
                res = res.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: strategy.locked_addr.unwrap().to_string(),
                    msg: to_binary(&angel_core::msgs::vault::ExecuteMsg::Redeem {
                        endowment_id: action.account_ids[0],
                        amount: action.lock_amt,
                    })
                    .unwrap(),
                    funds: vec![],
                })));
            }
        }

        if action.liq_amt > Uint128::zero() {
            let available_liquid: Uint128 = deps.querier.query_wasm_smart(
                strategy.liquid_addr.clone().unwrap().to_string(),
                &angel_core::msgs::vault::QueryMsg::Balance {
                    endowment_id: action.account_ids[0],
                },
            )?;
            if !available_liquid.is_zero() {
                res = res.add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: strategy.liquid_addr.unwrap().to_string(),
                    msg: to_binary(&angel_core::msgs::vault::ExecuteMsg::RedeemAll {
                        endowment_id: action.account_ids[0],
                    })
                    .unwrap(),
                    funds: vec![],
                })));
            }
        }

        Ok(res)
    } else {
        Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Vault Router can only handle messages bound for {}",
                action.destination_chain
            ),
        }))
    }
}

pub fn execute_harvest(
    deps: DepsMut,
    _sender: Addr,
    action: VaultActionData,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;

    // check if the router should re-route onward to Axelar Gateway or to local Accounts
    if action.destination_chain == registrar_config.axelar_chain_id {
        let strategy_res: StrategyDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Strategy {
                    strategy_key: action.strategy_id,
                })?,
            }))?;
        let strategy: StrategyParams = strategy_res.strategy;

        Ok(Response::default()
            .add_attribute("action", "harvest")
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: strategy.liquid_addr.unwrap().to_string(),
                msg: to_binary(&angel_core::msgs::vault::ExecuteMsg::Harvest {
                    account_ids: action.account_ids.clone(),
                })
                .unwrap(),
                funds: vec![],
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: strategy.locked_addr.unwrap().to_string(),
                msg: to_binary(&angel_core::msgs::vault::ExecuteMsg::Harvest {
                    account_ids: action.account_ids,
                })
                .unwrap(),
                funds: vec![],
            }))))
    } else {
        Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Vault Router can only handle messages bound for {}",
                action.destination_chain
            ),
        }))
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    registrar_contract: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // only owner can execute changes to config
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    config.owner = match owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => config.owner,
    };

    config.registrar_contract = match registrar_contract {
        Some(contract) => deps.api.addr_validate(&contract)?,
        None => config.registrar_contract,
    };
    // save modified config
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}
