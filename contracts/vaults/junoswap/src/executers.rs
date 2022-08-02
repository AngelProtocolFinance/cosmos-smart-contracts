use std::vec;

use crate::config::{PENDING, REMNANTS};
use crate::util::query_denom_balance;
use crate::wasmswap::{swap_msg, InfoResponse, Token2ForToken1PriceResponse, TokenSelect};
use crate::{config, cw20_stake, wasmswap};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{AccountWithdrawMsg, ExecuteMsg, UpdateConfigMsg};
use angel_core::responses::registrar::EndowmentListResponse;
use angel_core::structs::EndowmentEntry;
use cosmwasm_std::{
    attr, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, SubMsgResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::Denom;
use cw_controllers::ClaimsResponse;

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only the owner/admin of the contract can update their address in the configs
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed args
    config.owner = new_owner;
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: Addr,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only the registrar contract can update it's address in the config
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed args
    config.registrar_contract = new_registrar;
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.pool_addr = match msg.swap_pool_addr {
        Some(ref addr) => deps.api.addr_validate(&addr)?,
        None => config.pool_addr,
    };

    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(&config.pool_addr, &wasmswap::QueryMsg::Info {})?;

    config.pool_lp_token_addr = deps.api.addr_validate(&swap_pool_info.lp_token_address)?;
    config.input_denoms = vec![swap_pool_info.token1_denom, swap_pool_info.token2_denom];
    config.staking_addr = match msg.staking_addr {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.staking_addr,
    };

    // Add more addresses to `config.routes`
    for addr in msg.routes.add {
        if !config.routes.contains(&addr) {
            config.routes.push(addr);
        }
    }

    // Remove the addresses from `config.routes`
    for addr in msg.routes.remove {
        if config.routes.contains(&addr) {
            let id = config
                .routes
                .iter()
                .enumerate()
                .find(|(_, v)| **v == addr)
                .unwrap()
                .0;
            config.routes.swap_remove(id);
        }
    }

    config.output_token_denom = msg.output_token_denom.unwrap_or(config.output_token_denom);
    if !config.input_denoms.contains(&config.output_token_denom) {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Invalid output token denom: {:?}",
                config.output_token_denom.clone()
            ),
        }));
    }

    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    depositor: String,
    deposit_denom: Denom,
    deposit_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    if !config.input_denoms.contains(&deposit_denom) {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    if deposit_amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // check that the depositor is an Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::EndowmentList {
                name: None,
                owner: None,
                status: None,
                tier: None,
                un_sdg: None,
                endow_type: None,
            })?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments
        .iter()
        .position(|p| p.address.to_string() == depositor);
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", depositor.to_string())
        .add_attribute("deposit_amount", deposit_amount);

    // 1. Add the "swap" message
    res = res.add_messages(swap_msg(&config, &deposit_denom, deposit_amount)?);

    // 2. Add the "add_liquidity" message
    let in_denom = deposit_denom;
    let out_denom = if in_denom == config.input_denoms[0] {
        config.input_denoms[1].clone()
    } else {
        config.input_denoms[0].clone()
    };
    let in_denom_bal_before =
        query_denom_balance(&deps, &in_denom, env.contract.address.to_string());
    let out_denom_bal_before =
        query_denom_balance(&deps, &out_denom, env.contract.address.to_string());
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::AddLiquidity {
            depositor,
            in_denom,
            out_denom,
            in_denom_bal_before,
            out_denom_bal_before,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
}

/// Claim: Call the `claim` entry of "staking" contract
pub fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Addr,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    // check that the depositor is an Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::EndowmentList {
                name: None,
                owner: None,
                status: None,
                tier: None,
                un_sdg: None,
                endow_type: None,
            })?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments
        .iter()
        .position(|p| p.address.to_string() == info.sender.to_string());
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // First, check if there is any possible claim in "staking" contract
    let claims_resp: ClaimsResponse = deps.querier.query_wasm_smart(
        config.staking_addr.to_string(),
        &cw20_stake::QueryMsg::Claims {
            address: env.contract.address.to_string(),
        },
    )?;
    if claims_resp.claims.len() == 0 {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Nothing to claim".to_string(),
        }));
    }

    // Performs the "claim"
    let mut res = Response::default();
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.staking_addr.to_string(),
        msg: to_binary(&cw20_stake::ExecuteMsg::Claim {}).unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr,
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before: lp_token_bal.balance,
            beneficiary,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
}

/// Withdraw: Takes in an amount of vault tokens
/// to withdraw from the vault for USDC to send back to a beneficiary
pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // check that the depositor is an Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::EndowmentList {
                name: None,
                owner: None,
                status: None,
                tier: None,
                un_sdg: None,
                endow_type: None,
            })?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments
        .iter()
        .position(|p| p.address.to_string() == info.sender.to_string());
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // Query the "lp_token" balance beforehand in order to resolve the `deps` issue.
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    // First, burn the vault tokens
    let account_info = MessageInfo {
        sender: info.sender.clone(),
        funds: vec![],
    };
    config.total_shares -= msg.amount;
    config::store(deps.storage, &config)?;

    cw20_base::contract::execute_burn(deps, env.clone(), account_info, msg.amount).map_err(
        |_| {
            ContractError::Std(StdError::GenericErr {
                msg: format!(
                    "Cannot burn the {} vault tokens from {}",
                    msg.amount,
                    info.sender.to_string()
                ),
            })
        },
    )?;

    // Perform the "unstaking"
    let mut res = Response::default();
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.staking_addr.to_string(),
        msg: to_binary(&cw20_stake::ExecuteMsg::Unstake { amount: msg.amount }).unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before: lp_token_bal.balance,
            beneficiary: msg.beneficiary,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
}

pub fn harvest(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collector_address: String,
    collector_share: Decimal,
) -> Result<Response, ContractError> {
    // TODO
    Ok(Response::default())
}

pub fn process_junoswap_pool_reply(
    deps: DepsMut,
    env: Env,
    id: u64,
    result: SubMsgResult,
) -> Result<Response, ContractError> {
    // pull up the pending transaction details from storage
    let transaction = PENDING.load(deps.storage, &id.to_be_bytes())?;

    // remove this pending transaction
    PENDING.remove(deps.storage, &id.to_be_bytes());

    match result {
        SubMsgResult::Ok(subcall) => {
            // TODO
            Ok(Response::default())
        }
        SubMsgResult::Err(err) => Err(ContractError::Std(StdError::GenericErr { msg: err })),
    }
}

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    depositor: String,
    in_denom: Denom,
    out_denom: Denom,
    in_denom_bal_before: Uint128,
    out_denom_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    let in_denom_bal = query_denom_balance(&deps, &in_denom, env.contract.address.to_string());
    let out_denom_bal = query_denom_balance(&deps, &out_denom, env.contract.address.to_string());

    let token1_denom: Denom;
    let token2_denom: Denom;
    let token1_amount: Uint128;
    let token2_amount: Uint128;

    if in_denom == config.input_denoms[0] {
        token1_denom = in_denom;
        token2_denom = out_denom;
        token1_amount = in_denom_bal_before - in_denom_bal;
        token2_amount = out_denom_bal - out_denom_bal_before;
    } else {
        token1_denom = out_denom;
        token2_denom = in_denom;
        token1_amount = out_denom_bal - out_denom_bal_before;
        token2_amount = in_denom_bal_before - in_denom_bal;
    }

    let price_query: Token2ForToken1PriceResponse = deps.querier.query_wasm_smart(
        config.pool_addr.to_string(),
        &crate::wasmswap::QueryMsg::Token2ForToken1Price { token2_amount },
    )?;

    if price_query.token1_amount > token1_amount {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Invalid liquidity amount - Needed: {}, Current: {}",
                price_query.token1_amount, token1_amount
            ),
        }));
    }

    let token1_denom_string = match token1_denom {
        Denom::Native(ref denom) => denom.to_string(),
        Denom::Cw20(ref addr) => addr.to_string(),
    };

    REMNANTS.update(
        deps.storage,
        token1_denom_string,
        |amount| -> Result<Uint128, ContractError> {
            let amount =
                amount.unwrap_or(Uint128::zero()) + token1_amount - price_query.token1_amount;
            Ok(amount)
        },
    )?;
    let token1_amount = price_query.token1_amount;

    let mut funds = vec![];
    let mut msgs = vec![];

    match token1_denom {
        Denom::Native(denom) => funds.push(Coin {
            denom,
            amount: token1_amount,
        }),
        Denom::Cw20(contract_addr) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: config.pool_addr.to_string(),
                amount: token1_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    match token2_denom {
        Denom::Native(denom) => funds.push(Coin {
            denom,
            amount: token2_amount,
        }),
        Denom::Cw20(contract_addr) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: config.pool_addr.to_string(),
                amount: token2_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_addr.to_string(),
        msg: to_binary(&crate::wasmswap::ExecuteMsg::AddLiquidity {
            token1_amount,
            min_liquidity: Uint128::zero(),
            max_token2: token2_amount,
            expiration: None,
        })
        .unwrap(),
        funds,
    }));

    // Add the "stake" message at last
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::Stake {
            depositor,
            lp_token_bal_before: lp_token_bal.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "add_liquidity_to_swap_pool")]))
}

pub fn stake_lp_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    depositor: String,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;

    // Perform the "staking"
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let stake_amount = lp_token_bal.balance - lp_token_bal_before;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_lp_token_addr.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.staking_addr.to_string(),
            amount: stake_amount,
            msg: to_binary(&crate::cw20_stake::ReceiveMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    // Mint the `vault_token`
    config.total_shares += stake_amount;
    config::store(deps.storage, &config)?;

    cw20_base::contract::execute_mint(deps, env, info, depositor.to_string(), stake_amount)
        .map_err(|_| {
            ContractError::Std(StdError::GenericErr {
                msg: format!(
                    "Cannot mint the {} vault token for {}",
                    stake_amount, depositor
                ),
            })
        })?;

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![attr("action", "stake_lp_token")]))
}

pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    lp_token_bal_before: Uint128,
    beneficiary: Addr,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    // First, compute the current "lp_token" balance
    let lp_token_bal_now: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    // Compute the "lp_token" amount to be used for "remove_liquidity"
    let lp_token_amt = lp_token_bal_now
        .balance
        .checked_sub(lp_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::Overflow { source: e }))?;

    // Perform the "remove_liquidity"
    let mut res = Response::default();
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_addr.to_string(),
        msg: to_binary(&wasmswap::ExecuteMsg::RemoveLiquidity {
            amount: lp_token_amt,
            min_token1: Uint128::zero(),
            min_token2: Uint128::zero(),
            expiration: None,
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the returning token pairs
    let token1_denom_bal = query_denom_balance(
        &deps,
        &config.input_denoms[0],
        env.contract.address.to_string(),
    );
    let token2_denom_bal = query_denom_balance(
        &deps,
        &config.input_denoms[1],
        env.contract.address.to_string(),
    );
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapAndSendTo {
            token1_denom_bal_before: token1_denom_bal,
            token2_denom_bal_before: token2_denom_bal,
            beneficiary,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
}

pub fn swap_and_send(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    token1_denom_bal_before: Uint128,
    token2_denom_bal_before: Uint128,
    beneficiary: Addr,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

    // First, compute the token balances
    let token1_denom_bal_now = query_denom_balance(
        &deps,
        &config.input_denoms[0],
        env.contract.address.to_string(),
    );
    let token2_denom_bal_now = query_denom_balance(
        &deps,
        &config.input_denoms[1],
        env.contract.address.to_string(),
    );
    let token1_amt = token1_denom_bal_now
        .checked_sub(token1_denom_bal_before)
        .map_err(|e| ContractError::Std(StdError::Overflow { source: e }))?;
    let token2_amt = token2_denom_bal_now
        .checked_sub(token2_denom_bal_before)
        .map_err(|e| ContractError::Std(StdError::Overflow { source: e }))?;

    // Check if `output_token_denom` is `Token1` or `Token2`
    // Also, determine which token to send directly, which token to `SwapAndSendTo`
    let direct_send_token_denom = config.output_token_denom;
    let direct_send_token_amt: Uint128;

    let swap_input_token_denom: Denom;
    let swap_input_token_amt: Uint128;

    let swap_input_token: TokenSelect;

    if direct_send_token_denom == config.input_denoms[0] {
        direct_send_token_amt = token1_amt;

        swap_input_token_denom = config.input_denoms[1].clone();
        swap_input_token_amt = token2_amt;

        swap_input_token = TokenSelect::Token2;
    } else {
        direct_send_token_amt = token2_amt;

        swap_input_token_denom = config.input_denoms[0].clone();
        swap_input_token_amt = token1_amt;

        swap_input_token = TokenSelect::Token1;
    };

    // Perform the direct send of `output_token_denom`
    let mut res = Response::default();
    let mut msgs: Vec<CosmosMsg> = vec![];

    match direct_send_token_denom {
        Denom::Native(denom) => {
            msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: beneficiary.to_string(),
                amount: coins(direct_send_token_amt.u128(), denom),
            }));
        }
        Denom::Cw20(token_addr) => {
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_addr.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: beneficiary.to_string(),
                    amount: direct_send_token_amt,
                })
                .unwrap(),
                funds: vec![],
            }));
        }
    }

    // Perform the `SwapAndSendTo` of `juno-swap-pool` contract
    match swap_input_token_denom {
        Denom::Native(denom) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.pool_addr.to_string(),
            msg: to_binary(&wasmswap::ExecuteMsg::SwapAndSendTo {
                input_token: swap_input_token,
                input_amount: swap_input_token_amt,
                recipient: beneficiary.to_string(),
                min_token: Uint128::zero(),
                expiration: None,
            })
            .unwrap(),
            funds: coins(swap_input_token_amt.u128(), denom),
        })),
        Denom::Cw20(token_addr) => {
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_addr.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                    spender: config.pool_addr.to_string(),
                    amount: swap_input_token_amt,
                    expires: None,
                })
                .unwrap(),
                funds: vec![],
            }));
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.pool_addr.to_string(),
                msg: to_binary(&wasmswap::ExecuteMsg::SwapAndSendTo {
                    input_token: swap_input_token,
                    input_amount: swap_input_token_amt,
                    recipient: beneficiary.to_string(),
                    min_token: Uint128::zero(),
                    expiration: None,
                })
                .unwrap(),
                funds: vec![],
            }))
        }
    }

    res = res.add_messages(msgs);
    Ok(res)
}
