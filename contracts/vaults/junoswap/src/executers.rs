use crate::config::{Config, PendingInfo, BALANCES, PENDING, REMNANTS, TOKEN_INFO};
use crate::util::query_denom_balance;
use crate::wasmswap::{swap_msg, InfoResponse, Token2ForToken1PriceResponse};
use crate::{config, wasmswap};
use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{AccountWithdrawMsg, ExecuteMsg, UpdateConfigMsg};
use angel_core::responses::registrar::EndowmentListResponse;
use angel_core::structs::{BalanceInfo, EndowmentEntry};
use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    Order, QueryRequest, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128,
    WasmMsg, WasmQuery,
};
use cw20::{Balance, Denom};

// wallet that we use for regular, automated harvests of vault
const CRON_WALLET: &str = "terra1janh9rs6pme3tdwhyag2lmsr2xv6wzhcrjz0xx";

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

    config.harvest_to_liquid = msg.harvest_to_liquid.unwrap_or(config.harvest_to_liquid);

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

/// Redeem Stable: Take in an amount of locked/liquid deposit tokens
/// to redeem from the vault for stablecoins to send back to the the Accounts SC
pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    account_addr: Addr,
) -> Result<Response, ContractError> {
    // TODO
    Ok(Response::default())
}

/// Withdraw Stable: Takes in an amount of locked/liquid deposit tokens
/// to withdraw from the vault for UST to send back to a beneficiary
pub fn withdraw_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    // TODO
    Ok(Response::default())
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
    _info: MessageInfo,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;

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

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![attr("action", "stake_lp_token")]))
}
