use cosmwasm_std::{
    attr, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, Fraction,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::{Asset, AssetInfo};

use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{
    ExecuteMsg, LoopFarmingExecuteMsg, LoopFarmingQueryMsg, LoopPairExecuteMsg, ReceiveMsg,
    UpdateConfigMsg,
};
use angel_core::responses::{accounts::EndowmentListResponse, registrar::ConfigResponse};
use angel_core::structs::{AccountType, EndowmentEntry};
use terraswap::querier::{
    query_balance, query_pair_info, query_pair_info_from_pair, query_token_balance,
};

use crate::state::{Config, APTAX, BALANCES, CONFIG, TOKEN_INFO};

/// Contract entry: **update_owner**
pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the owner/admin of the contract can update their address in the configs
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed args
    config.owner = new_owner;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Contract entry: **update_registrar**
///
/// Update the `registrar_contract` address in **CONFIG**
pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the registrar contract can update it's address in the config
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed args
    config.registrar_contract = new_registrar;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Contract entry: **update_owner**
///
/// Update the **CONFIG** of the contract
pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Update the config
    config.sibling_vault = match msg.sibling_vault {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.sibling_vault,
    };

    config.lp_staking_contract = match msg.lp_staking_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.lp_staking_contract,
    };

    config.lp_pair_contract = match msg.lp_pair_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.lp_pair_contract,
    };

    let pair_info = query_pair_info_from_pair(&deps.querier, config.lp_pair_contract.clone())?;
    config.lp_pair_asset_infos = pair_info.asset_infos;
    config.lp_token_contract = deps.api.addr_validate(&pair_info.liquidity_token)?;

    config.keeper = match msg.keeper {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.keeper,
    };
    config.tax_collector = match msg.tax_collector {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.tax_collector,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Contract entry: **deposit**
///   1. Swap the half of input token to lp contract pair token
///   2. Call the `(this contract::)add_liquidity` entry
pub fn deposit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg_sender: String,
    endowment_id: u32,
    deposit_asset_info: AssetInfo,
    deposit_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if the `caller` is "accounts_contract" & "endowment_id" is valid
    validate_action_caller_n_endow_id(deps.as_ref(), &config, msg_sender.clone(), endowment_id)?;

    // Check if the "deposit_asset_info" is valid
    if !config.lp_pair_asset_infos.contains(&deposit_asset_info) {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    // Check if the "deposit_amount" is zero
    if deposit_amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // Swap the half of input token to lp contract pair token
    let input_amount = deposit_amount.multiply_ratio(1_u128, 2_u128);
    let loop_pair_swap_msgs = prepare_loop_pair_swap_msg(
        &config.lp_pair_contract.to_string(),
        &deposit_asset_info,
        input_amount,
    )?;

    // Call the "(this contract::)add_liquidity" entry
    let contract_add_liquidity_msgs = prepare_contract_add_liquidity_msgs(
        deps,
        env,
        &config,
        Some(endowment_id),
        deposit_asset_info,
    )?;

    Ok(Response::default()
        .add_messages(loop_pair_swap_msgs)
        .add_messages(contract_add_liquidity_msgs)
        .add_attribute("action", "deposit")
        .add_attribute("sender", msg_sender)
        .add_attribute("endow_id", endowment_id.to_string())
        .add_attribute("deposit_amount", deposit_amount))
}

/// Contract entry: **restake_claim_reward**
///   1. Compute the amount of `lp_reward_token`(`LOOP`) generated from `harvest(claim)`
///   2. Convert the `lp_reward_token`(`LOOP`)s to the LP tokens
///   3. Re-stake the LP tokens
pub fn restake_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    // Check if the caller is this contract
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Compute the `lp_reward_token` amount
    let reward_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.lp_reward_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    let reward_amount = reward_token_bal_query
        .balance
        .checked_sub(reward_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;

    // Re-stake the `reward token`s for more yield
    //
    // If the `lp_reward_token` is one of pair tokens in `lp_pair_contract`,
    // it follows the `deposit` entry flow.
    //
    // Otherwise, it converts the `lp_reward_token` to lp contract pair token.
    // Then, it converts lp contract pair tokens to LP token & does the staking.
    let reward_asset_info = AssetInfo::Token {
        contract_addr: config.lp_reward_token.to_string(),
    };
    let mut loop_pair_swap_msgs = vec![];
    let mut contract_add_liquidity_msgs = vec![];
    if config.lp_pair_asset_infos.contains(&reward_asset_info) {
        // Swap the half of input token to the lp contract pair token
        loop_pair_swap_msgs.extend_from_slice(&prepare_loop_pair_swap_msg(
            &config.lp_pair_contract.to_string(),
            &AssetInfo::Token {
                contract_addr: config.lp_reward_token.to_string(),
            },
            reward_amount.multiply_ratio(1_u128, 2_u128),
        )?);

        // Call the "(this contract::)add_liquidity" entry
        contract_add_liquidity_msgs.extend_from_slice(&prepare_contract_add_liquidity_msgs(
            deps,
            env,
            &config,
            None,
            AssetInfo::Token {
                contract_addr: config.lp_reward_token.to_string(),
            },
        )?);
    } else {
        let pair_1_contract = query_pair_info(
            &deps.querier,
            config.lp_factory_contract.clone(),
            &[
                reward_asset_info.clone(),
                config.lp_pair_asset_infos[0].clone(),
            ],
        )?
        .contract_addr;
        let pair_2_contract = query_pair_info(
            &deps.querier,
            config.lp_factory_contract.clone(),
            &[
                reward_asset_info.clone(),
                config.lp_pair_asset_infos[1].clone(),
            ],
        )?
        .contract_addr;

        // Swap the half of input token to lp contract pair token(token 1)
        loop_pair_swap_msgs = prepare_loop_pair_swap_msg(
            &pair_1_contract,
            &AssetInfo::Token {
                contract_addr: config.lp_reward_token.to_string(),
            },
            reward_amount.multiply_ratio(1_u128, 2_u128),
        )?;

        // Swap the half of input token to lp contract pair token(token 2)
        loop_pair_swap_msgs.extend_from_slice(&prepare_loop_pair_swap_msg(
            &pair_2_contract,
            &AssetInfo::Token {
                contract_addr: config.lp_reward_token.to_string(),
            },
            reward_amount.multiply_ratio(1_u128, 2_u128),
        )?);

        // Call the "(this contract::)add_liquidity" entry
        let token1_bal_before = query_asset_balance(
            deps.as_ref(),
            env.contract.address.clone(),
            config.lp_pair_asset_infos[0].clone(),
        )?;
        let token2_bal_before = query_asset_balance(
            deps.as_ref(),
            env.contract.address.clone(),
            config.lp_pair_asset_infos[0].clone(),
        )?;
        contract_add_liquidity_msgs =
            prepare_add_liquidity_msgs(deps, env, &config, token1_bal_before, token2_bal_before)?;
    }

    Ok(Response::default()
        .add_messages(loop_pair_swap_msgs)
        .add_messages(contract_add_liquidity_msgs)
        .add_attributes(vec![attr("action", "restake_claimed_reward")]))
}

fn prepare_add_liquidity_msgs(
    deps: DepsMut,
    env: Env,
    config: &Config,
    token1_bal_before: Uint128,
    token2_bal_before: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    // Compute the amounts of lp contract pair tokens, available for `provide_liquidity` operation
    let token1_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_asset_infos[0].clone(),
    )?;
    let token2_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_asset_infos[0].clone(),
    )?;
    let token1_amount = token1_bal - token1_bal_before;
    let token2_amount = token2_bal - token2_bal_before;

    // Call the "loopswap::pair::provide_liquidity" entry
    let loop_pair_provide_liquidity_msgs = prepare_loop_pair_provide_liquidity_msgs(
        config.lp_pair_asset_infos[0].clone(),
        token1_amount,
        config.lp_pair_asset_infos[1].clone(),
        token2_amount,
        config.lp_pair_contract.to_string(),
    )?;
    msgs.extend_from_slice(&loop_pair_provide_liquidity_msgs);

    // Call the "(this contract::)stake" entry
    let contract_stake_msgs =
        prepare_contract_stake_msgs(deps.as_ref(), env, None, config.lp_pair_contract.clone())?;
    msgs.extend_from_slice(&contract_stake_msgs);

    Ok(msgs)
}

/// Contract entry: **redeem**
///   1. Unstake/unfarm the LP tokens from the `loopswap::farming` contract
///   2. Re-stake the `lp_reward_token(`LOOP`)
///   3. Return the `LP token`s to the `accounts` contract
pub fn redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: u32,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let registar_config: ConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &RegistrarQueryMsg::Config {},
    )?;
    let accounts_contract = deps
        .api
        .addr_validate(&registar_config.accounts_contract.unwrap())?;

    let burn_shares_amount = amount;

    let beneficiary: Addr;
    let id: Option<u32>;
    if info.sender == config.tax_collector {
        beneficiary = config.tax_collector.clone();
        id = None;
    } else {
        // Check if the `caller` is "accounts_contract" & "endowment_id" is valid
        validate_action_caller_n_endow_id(
            deps.as_ref(),
            &config,
            info.sender.to_string(),
            endowment_id,
        )?;
        beneficiary = accounts_contract.clone();
        id = Some(endowment_id);
    }

    // First, burn the vault tokens
    execute_burn(deps.branch(), env.clone(), info, id, burn_shares_amount).map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {}",
                burn_shares_amount,
                endowment_id.to_string()
            ),
        })
    })?;

    // Update the config
    let lp_amount = burn_shares_amount.multiply_ratio(config.total_lp_amount, config.total_shares);
    config.total_lp_amount -= lp_amount;
    config.total_shares -= burn_shares_amount;

    CONFIG.save(deps.storage, &config)?;

    // Call the "loopswap::farming::unstake_and_claim(unfarm)" entry
    let mut msgs = vec![];
    let lp_token_contract = config.lp_token_contract;
    let flp_token_contract: String = deps.querier.query_wasm_smart(
        config.lp_staking_contract.to_string(),
        &LoopFarmingQueryMsg::QueryFlpTokenFromPoolAddress {
            pool_address: lp_token_contract.to_string(),
        },
    )?;

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: flp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::UnstakeAndClaim {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens
    let lp_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        lp_token_contract,
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before: lp_bal_query.balance,
            beneficiary,
            id,
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the lp_reward_token(LOOP) tokens (Re-stake the lp_reward_tokens)
    let reward_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.lp_reward_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: reward_token_bal_query.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::default().add_messages(msgs).add_attributes(vec![
        attr("action", "withdraw"),
        attr("burn_shares", burn_shares_amount.to_string()),
        attr("lp_amount", lp_amount.to_string()),
    ]))
}

/// Contract entry: **harvest**
///   1. `Claim` the `lp_reward_token` from `loopswap::farming` contract
///   2. Convert the `lp_reward_token`(LOOP) to LP tokens
///   3. Re-stake the LP tokens to the `farming` contract
pub fn harvest(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if the caller is `keeper` address
    if info.sender != config.keeper {
        return Err(ContractError::Unauthorized {});
    }

    // Call the "loopswap::farming::claim_reward" entry
    let mut msgs = vec![];
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_staking_contract.to_string(),
        msg: to_binary(&LoopFarmingExecuteMsg::ClaimReward {}).unwrap(),
        funds: vec![],
    }));

    // Re-stake the lp_reward_token(LOOP)
    let reward_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.lp_reward_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: reward_token_bal_query.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::default()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "harvest")]))
}

/// Contract entry: **reinvest_to_locked** (liquid vault logic)
///   1. Burn the `vault_token`
///   2. Unstake the LP tokens from `farming` contract
///   3. Re-stake the lp_reward_token from `unstake` operation
///   4. Send the unstaked LP tokens to the sibling vault(locked)
pub fn reinvest_to_locked_execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    amount: Uint128, // vault tokens
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // Check that the vault acct_type is `liquid`
    if config.acct_type != AccountType::Liquid {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "This is locked vault".to_string(),
        }));
    }

    // 0. Check that the message sender is the Accounts contract
    validate_action_caller_n_endow_id(deps.as_ref(), &config, info.sender.to_string(), id)?;
    // 1. Check that this vault has a sibling set
    if config.sibling_vault.to_string() == env.contract.address.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Sibling vault not created".to_string(),
        }));
    }
    // 2. Check that sender ID has >= amount of vault tokens in it's balance
    let endowment_vt_balance = crate::queriers::query_balance(deps.as_ref(), id).balance;
    if amount > endowment_vt_balance {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Insufficient balance: Needed {}, existing: {}",
                amount, endowment_vt_balance
            ),
        }));
    }
    // 3. Burn vault tokens an calculate the LP Tokens equivalent
    // First, burn the vault tokens
    execute_burn(deps.branch(), env.clone(), info, Some(id), amount).map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {}",
                amount,
                id.to_string()
            ),
        })
    })?;

    // Update the config
    let lp_amount = amount.multiply_ratio(config.total_lp_amount, config.total_shares);
    config.total_lp_amount -= lp_amount;
    config.total_shares -= amount;

    CONFIG.save(deps.storage, &config)?;

    // Unfarm the LP token from "lp_staking_contract"
    let lp_token_contract = config.lp_token_contract.clone();
    let flp_token_contract: String = deps.querier.query_wasm_smart(
        config.lp_staking_contract.to_string(),
        &LoopFarmingQueryMsg::QueryFlpTokenFromPoolAddress {
            pool_address: lp_token_contract.to_string(),
        },
    )?;

    let unstake_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: flp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::UnstakeAndClaim {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    })];

    // 4. SEND LP tokens to the Locked Account (using ReinvestToLocked recieve msg)
    let reinvest_to_locked_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.sibling_vault.to_string(),
            amount: lp_amount,
            msg: to_binary(&ReceiveMsg::ReinvestToLocked {
                endowment_id: id,
                amount: lp_amount,
            })
            .unwrap(),
        })
        .unwrap(),
        funds: vec![],
    })];

    // 5. Handle the reward LOOP tokens(= re-stake the reward tokens)
    let reward_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.lp_reward_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    let restake_reward_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: reward_token_bal_query.balance,
        })
        .unwrap(),
        funds: vec![],
    })];

    Ok(Response::new()
        .add_messages(unstake_msgs)
        .add_messages(reinvest_to_locked_msgs)
        .add_messages(restake_reward_msgs)
        .add_attributes(vec![attr("action", "reinvest_to_locked_vault")]))
}

/// Contract entry: **reinvest_to_locked** (locked vault logic)
///   1. Receive the LP tokens from sibling vault(liquid)
///   2. Stake the LP tokens to the `farming` contract
pub fn reinvest_to_locked_recieve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    amount: Uint128, // asset tokens (ex. LPs)
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // 0. Check that the message sender is the Sibling vault contract
    // ensure `received token` is `lp_token`
    if info.sender != config.lp_token_contract {
        return Err(ContractError::Unauthorized {});
    }

    // ensure this vault is "locked"
    if config.acct_type != AccountType::Locked {
        return Err(ContractError::Unauthorized {});
    }

    // ensure `msg_sender` is sibling_vault(liquid)
    let msg_sender = cw20_msg.sender;
    let sibling_config_resp: angel_core::responses::vault::ConfigResponse =
        deps.querier.query_wasm_smart(
            config.sibling_vault.to_string(),
            &angel_core::messages::vault::QueryMsg::Config {},
        )?;
    if msg_sender != config.sibling_vault.to_string()
        || sibling_config_resp.acct_type != AccountType::Liquid
    {
        return Err(ContractError::Unauthorized {});
    }

    // ensure the `amount` matches `sent_amount`.
    let sent_amount = cw20_msg.amount;
    if amount != sent_amount {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Balance does not match: Received: {}, Expected: {}",
                sent_amount, amount
            ),
        }));
    }

    // 1. Treat as a Deposit for the given ID (mint vault tokens for deposited assets)
    // Prepare the messages for "loop::farming::stake" opeartion
    let lp_stake_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount,
            msg: to_binary(&LoopFarmingExecuteMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    })];

    // 2. Mint the vault tokens
    // Compute the `vault_token` amount
    config.total_lp_amount += amount;
    let vt_mint_amount = match config.total_shares.u128() {
        0 => Uint128::from(1000000_u128), // Here, the original mint amount should be 1 VT.
        _ => amount.multiply_ratio(config.total_shares, config.total_lp_amount),
    };
    config.total_shares += vt_mint_amount;
    CONFIG.save(deps.storage, &config)?;

    // Mint the `vault_token`
    execute_mint(deps, env, info, Some(id), vt_mint_amount).map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!("Cannot mint the {} vault token for {}", vt_mint_amount, id),
        })
    })?;

    Ok(Response::new()
        .add_messages(lp_stake_msgs)
        .add_attributes(vec![attr("action", "reinvest_to_locked_vault")]))
}

/// Contract entry: **add_liquidity**
///   1. Add/Provide the `liquidity` to the `loopswap::pair` contract
///   2. Call the `(this contract::)stake` entry
pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: Option<u32>,
    in_asset_info: AssetInfo,
    out_asset_info: AssetInfo,
    in_asset_bal_before: Uint128,
    out_asset_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Add the "loopswap::pair::provide_liquidity" message
    let config = CONFIG.load(deps.storage)?;

    let in_asset_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        in_asset_info.clone(),
    )?;
    let out_asset_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        out_asset_info.clone(),
    )?;

    let token1_asset_info: AssetInfo;
    let token2_asset_info: AssetInfo;
    let token1_amount: Uint128;
    let token2_amount: Uint128;

    let asset_infos = config.lp_pair_asset_infos;

    if in_asset_info == asset_infos[0] {
        token1_asset_info = in_asset_info;
        token2_asset_info = out_asset_info;
        token1_amount = in_asset_bal_before - in_asset_bal;
        token2_amount = out_asset_bal - out_asset_bal_before;
    } else {
        token1_asset_info = out_asset_info;
        token2_asset_info = in_asset_info;
        token1_amount = out_asset_bal - out_asset_bal_before;
        token2_amount = in_asset_bal_before - in_asset_bal;
    }

    let loop_pair_provide_liquidity_msgs = prepare_loop_pair_provide_liquidity_msgs(
        token1_asset_info,
        token1_amount,
        token2_asset_info,
        token2_amount,
        config.lp_pair_contract.to_string(),
    )?;

    // Add the "(this contract::)stake" message
    let contract_stake_msgs =
        prepare_contract_stake_msgs(deps.as_ref(), env, endowment_id, config.lp_pair_contract)?;

    Ok(Response::new()
        .add_messages(loop_pair_provide_liquidity_msgs)
        .add_messages(contract_stake_msgs)
        .add_attributes(vec![attr("action", "add_liquidity_to_loopswap_pair")]))
}

/// Prepare the `loopswap::pair::provide_liquidity` msgs
fn prepare_loop_pair_provide_liquidity_msgs(
    token1_asset_info: AssetInfo,
    token1_amount: Uint128,
    token2_asset_info: AssetInfo,
    token2_amount: Uint128,
    pair_contract: String,
) -> StdResult<Vec<CosmosMsg>> {
    let mut funds = vec![];
    let mut msgs = vec![];
    match token1_asset_info {
        AssetInfo::NativeToken { ref denom } => funds.push(Coin {
            denom: denom.to_string(),
            amount: token1_amount,
        }),
        AssetInfo::Token { ref contract_addr } => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: pair_contract.to_string(),
                amount: token1_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    match token2_asset_info {
        AssetInfo::NativeToken { ref denom } => funds.push(Coin {
            denom: denom.to_string(),
            amount: token2_amount,
        }),
        AssetInfo::Token { ref contract_addr } => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: pair_contract.to_string(),
                amount: token2_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_contract.to_string(),
        msg: to_binary(&LoopPairExecuteMsg::ProvideLiquidity {
            assets: [
                Asset {
                    info: token1_asset_info,
                    amount: token1_amount,
                },
                Asset {
                    info: token2_asset_info,
                    amount: token2_amount,
                },
            ],
        })
        .unwrap(),
        funds,
    }));

    Ok(msgs)
}

/// Prepare the `(this contract::)stake` msgs for `endowment_id`
fn prepare_contract_stake_msgs(
    deps: Deps,
    env: Env,
    endowment_id: Option<u32>,
    pair_contract: Addr,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    let lp_token_contract =
        query_pair_info_from_pair(&deps.querier, pair_contract)?.liquidity_token;
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        lp_token_contract,
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::Stake {
            endowment_id,
            lp_token_bal_before: lp_token_bal.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(msgs)
}

/// Contract entry: **Stake**
///   1. Stake/Farm the `LP` tokens received from `provide_liquidity`
///   2. Mint the `vault token`s
pub fn stake_lp_token(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: Option<u32>,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let mut config: Config = CONFIG.load(deps.storage)?;
    let mut harvest_to_liquid_msgs = vec![];

    // Prepare the "loop::farming::stake" msg
    let lp_token_contract = config.lp_token_contract.clone();
    let lp_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        lp_token_contract.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    let lp_amount = lp_bal_query
        .balance
        .checked_sub(lp_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;
    if lp_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let lp_stake_amount: Uint128;

    match endowment_id {
        // Case of `deposit` from `endowment`
        Some(endowment_id) => {
            // Compute the `vault_token` amount
            config.total_lp_amount += lp_amount;
            let vt_mint_amount = match config.total_shares.u128() {
                0 => Uint128::from(1000000_u128), // Here, the original mint amount should be 1 VT.
                _ => lp_amount.multiply_ratio(config.total_shares, config.total_lp_amount),
            };
            config.total_shares += vt_mint_amount;

            // Mint the `vault_token`
            execute_mint(deps.branch(), env, info, Some(endowment_id), vt_mint_amount).map_err(
                |_| {
                    ContractError::Std(StdError::GenericErr {
                        msg: format!(
                            "Cannot mint the {} vault token for {}",
                            vt_mint_amount, endowment_id
                        ),
                    })
                },
            )?;

            // Stake all the LP tokens
            lp_stake_amount = lp_amount;
        }
        // Case of `harvest` from `keeper` wallet
        None => {
            // Compute the `vault_token` amount
            config.total_lp_amount += lp_amount;
            let vt_mint_amount = match config.total_shares.u128() {
                0 => Uint128::from(1000000_u128), // Here, the original mint amount should be 1 VT.
                _ => lp_amount.multiply_ratio(config.total_shares, config.total_lp_amount),
            };
            config.total_shares += vt_mint_amount;

            // Compute the `ap_treasury tax portion` & store in APTAX
            let registrar_config: ConfigResponse = deps.querier.query_wasm_smart(
                config.registrar_contract.to_string(),
                &RegistrarQueryMsg::Config {},
            )?;
            let tax_rate = registrar_config.tax_rate;
            let tax_mint_amount =
                vt_mint_amount.multiply_ratio(tax_rate.numerator(), tax_rate.denominator());

            APTAX.update(deps.storage, |mut balance: Uint128| -> StdResult<Uint128> {
                balance = balance.checked_add(tax_mint_amount)?;
                Ok(balance)
            })?;

            match config.acct_type {
                AccountType::Liquid => {
                    // Mint the `vault token`
                    // update supply and enforce cap
                    let mut token_info = TOKEN_INFO.load(deps.storage)?;
                    token_info.total_supply += vt_mint_amount;
                    if let Some(limit) = token_info.get_cap() {
                        if token_info.total_supply > limit {
                            return Err(ContractError::CannotExceedCap {});
                        }
                    }
                    TOKEN_INFO.save(deps.storage, &token_info)?;

                    // Stake all the LP token
                    lp_stake_amount = lp_amount;
                }
                AccountType::Locked => {
                    // Send the portion of LP tokens to the sibling(liquid) vault
                    let lp_tax_amount =
                        lp_amount.multiply_ratio(tax_rate.numerator(), tax_rate.denominator());
                    let lp_less_tax = lp_amount - lp_tax_amount;

                    let send_liquid_ratio = registrar_config.rebalance.interest_distribution;
                    let send_liquid_lp_amount = lp_less_tax.multiply_ratio(
                        send_liquid_ratio.numerator(),
                        send_liquid_ratio.denominator(),
                    );
                    let vt_shares_to_burn = (vt_mint_amount - tax_mint_amount).multiply_ratio(
                        send_liquid_ratio.numerator(),
                        send_liquid_ratio.denominator(),
                    );
                    config.total_shares -= vt_shares_to_burn;
                    config.total_lp_amount -= send_liquid_lp_amount;

                    // Mint the `vault token` with left LP token
                    // update supply and enforce cap
                    let mut token_info = TOKEN_INFO.load(deps.storage)?;
                    token_info.total_supply += vt_mint_amount - vt_shares_to_burn;
                    if let Some(limit) = token_info.get_cap() {
                        if token_info.total_supply > limit {
                            return Err(ContractError::CannotExceedCap {});
                        }
                    }
                    TOKEN_INFO.save(deps.storage, &token_info)?;

                    // Stake only leftover LP tokens
                    lp_stake_amount = lp_amount - send_liquid_lp_amount;

                    harvest_to_liquid_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: lp_token_contract.to_string(),
                        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                            contract: config.sibling_vault.to_string(),
                            amount: send_liquid_lp_amount,
                            msg: to_binary(&ReceiveMsg::HarvestToLiquid {}).unwrap(),
                        })
                        .unwrap(),
                        funds: vec![],
                    }))
                }
            }
        }
    }
    CONFIG.save(deps.storage, &config)?;

    let farming_stake_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_stake_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(farming_stake_msg)
        .add_messages(harvest_to_liquid_msgs)
        .add_attributes(vec![attr("action", "stake_lp_token")]))
}

pub fn harvest_to_liquid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    let lp_token_contract = config.lp_token_contract.to_string();

    // 0. Check that the message sender is the Sibling vault contract
    // ensure `received token` is `lp_token`
    if info.sender.to_string() != lp_token_contract {
        return Err(ContractError::Unauthorized {});
    }

    // ensure this vault is "liquid"
    if config.acct_type != AccountType::Liquid {
        return Err(ContractError::Unauthorized {});
    }

    // ensure `msg_sender` is sibling_vault(locked)
    let msg_sender = cw20_msg.sender;
    let sibling_config_resp: angel_core::responses::vault::ConfigResponse =
        deps.querier.query_wasm_smart(
            config.sibling_vault.to_string(),
            &angel_core::messages::vault::QueryMsg::Config {},
        )?;
    if msg_sender != config.sibling_vault.to_string()
        || sibling_config_resp.acct_type != AccountType::Locked
    {
        return Err(ContractError::Unauthorized {});
    }

    // 1. Increase the lp_token amount
    let lp_stake_amount = cw20_msg.amount;
    config.total_lp_amount += lp_stake_amount;
    CONFIG.save(deps.storage, &config)?;

    // 2. Stake the received lp tokens
    let farming_stake_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_stake_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    Ok(Response::default()
        .add_message(farming_stake_msg)
        .add_attributes(vec![attr("action", "harvest_to_liquid")]))
}

pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lp_token_bal_before: Uint128,
    beneficiary: Addr,
    id: Option<u32>,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;
    let lp_token_contract = config.lp_token_contract.to_string();
    let asset_infos = config.lp_pair_asset_infos;
    let pair_contract = config.lp_pair_contract;

    // First, compute "unfarm"ed LP token balance for "remove_liquidity"
    let lp_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        lp_token_contract.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let lp_token_amt = lp_token_bal_query
        .balance
        .checked_sub(lp_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::Overflow { source: e }))?;

    // Prepare the "remove_liquidity" messages
    let mut withdraw_liquidity_msgs = vec![];
    withdraw_liquidity_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
            spender: pair_contract.to_string(),
            amount: lp_token_amt,
            expires: None,
        })
        .unwrap(),
        funds: vec![],
    }));
    withdraw_liquidity_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: pair_contract.to_string(),
            amount: lp_token_amt,
            msg: to_binary(&LoopPairExecuteMsg::WithdrawLiquidity {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the returning token pairs
    let mut send_asset_msgs = vec![];
    let asset_0_bal_before = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        asset_infos[0].clone(),
    )?;
    let asset_1_bal_before = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        asset_infos[1].clone(),
    )?;

    send_asset_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SendAsset {
            beneficiary: beneficiary.clone(),
            id,
            asset_info: asset_infos[0].clone(),
            asset_bal_before: asset_0_bal_before,
        })
        .unwrap(),
        funds: vec![],
    }));
    send_asset_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SendAsset {
            beneficiary,
            id,
            asset_info: asset_infos[1].clone(),
            asset_bal_before: asset_1_bal_before,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::default()
        .add_messages(withdraw_liquidity_msgs)
        .add_messages(send_asset_msgs)
        .add_attributes(vec![attr("action", "remove_liquidity")]))
}

pub fn send_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Addr,
    id: Option<u32>,
    asset_info: AssetInfo,
    asset_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config: Config = CONFIG.load(deps.storage)?;

    // Send the asset to the `beneficiary`
    let asset_bal = query_asset_balance(deps.as_ref(), env.contract.address, asset_info.clone())?;
    let send_amount = asset_bal
        .checked_sub(asset_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;

    let mut msgs = vec![];
    match asset_info {
        AssetInfo::NativeToken { denom } => match id {
            Some(id) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: beneficiary.to_string(),
                msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::VaultReceipt {
                    id,
                    acct_type: config.acct_type,
                })
                .unwrap(),
                funds: coins(send_amount.u128(), denom),
            })),
            None => msgs.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: beneficiary.to_string(),
                amount: coins(send_amount.u128(), denom),
            })),
        },
        AssetInfo::Token { contract_addr } => match id {
            Some(id) => {
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.clone(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                        spender: beneficiary.to_string(),
                        amount: send_amount,
                        expires: None,
                    })
                    .unwrap(),
                    funds: vec![],
                }));
                msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr,
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                        contract: beneficiary.to_string(),
                        amount: send_amount,
                        msg: to_binary(&angel_core::messages::accounts::ReceiveMsg::VaultReceipt {
                            id,
                            acct_type: config.acct_type,
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                }));
            }
            None => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr,
                msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                    recipient: beneficiary.to_string(),
                    amount: send_amount,
                })
                .unwrap(),
                funds: vec![],
            })),
        },
    };

    Ok(Response::default()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "send_asset")]))
}

fn prepare_contract_add_liquidity_msgs(
    deps: DepsMut,
    env: Env,
    config: &Config,
    endowment_id: Option<u32>,
    deposit_asset_info: AssetInfo,
) -> Result<Vec<CosmosMsg>, StdError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    let lp_pair_asset_infos = config.lp_pair_asset_infos.clone();
    let (in_asset_info, out_asset_info) = if deposit_asset_info == lp_pair_asset_infos[0] {
        (
            lp_pair_asset_infos[0].clone(),
            lp_pair_asset_infos[1].clone(),
        )
    } else {
        (
            lp_pair_asset_infos[1].clone(),
            lp_pair_asset_infos[0].clone(),
        )
    };

    let in_asset_bal_before = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        in_asset_info.clone(),
    )?;
    let out_asset_bal_before = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        out_asset_info.clone(),
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::AddLiquidity {
            endowment_id,
            in_asset_info,
            out_asset_info,
            in_asset_bal_before,
            out_asset_bal_before,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(msgs)
}

fn prepare_loop_pair_swap_msg(
    pair_contract: &str,
    input_asset_info: &AssetInfo,
    input_amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs: Vec<CosmosMsg> = vec![];
    if let AssetInfo::Token { contract_addr } = input_asset_info {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: pair_contract.to_string(),
                amount: input_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        }));
    }

    match input_asset_info {
        AssetInfo::NativeToken { denom } => {
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair_contract.to_string(),
                msg: to_binary(&LoopPairExecuteMsg::Swap {
                    offer_asset: Asset {
                        info: input_asset_info.clone(),
                        amount: input_amount,
                    },
                    belief_price: None,
                    max_spread: None,
                })?,
                funds: vec![Coin {
                    denom: denom.to_string(),
                    amount: input_amount,
                }],
            }));
        }
        AssetInfo::Token { contract_addr } => {
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                    contract: pair_contract.to_string(),
                    amount: input_amount,
                    msg: to_binary(&LoopPairExecuteMsg::Swap {
                        offer_asset: Asset {
                            info: input_asset_info.clone(),
                            amount: input_amount,
                        },
                        belief_price: None,
                        max_spread: None,
                    })
                    .unwrap(),
                })
                .unwrap(),
                funds: vec![],
            }));
        }
    };

    Ok(msgs)
}

/// Check if the `caller` is the `accounts_contract` address &
/// `endowment_id` is valid Endowment ID in `accounts_contract`
fn validate_action_caller_n_endow_id(
    deps: Deps,
    config: &Config,
    caller: String,
    endowment_id: u32,
) -> Result<(), ContractError> {
    // Check if sender address is the "accounts_contract"
    let registar_config: ConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &RegistrarQueryMsg::Config {},
    )?;
    if let Some(accounts_contract) = registar_config.accounts_contract {
        if caller != accounts_contract {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    // Check that the "deposit-endowment-id" is an Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&angel_core::messages::accounts::QueryMsg::EndowmentList {
                name: None,
                owner: None,
                status: None,
                tier: None,
                endow_type: None,
            })?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments.iter().position(|endow| endow.id == endowment_id);
    // reject if the "endowment-id" was not found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// Custom `mint` function for `vault token`
fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    endowment_id: Option<u32>,
    amount: Uint128,
) -> Result<(), ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let mut config = TOKEN_INFO.load(deps.storage)?;
    if config.mint.is_none() || config.mint.as_ref().unwrap().minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    match endowment_id {
        Some(id) => BALANCES.update(
            deps.storage,
            id,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?,
        None => APTAX.update(deps.storage, |balance: Uint128| -> StdResult<_> {
            Ok(balance + amount)
        })?,
    };

    Ok(())
}

/// Custom `burn` function for `vault token`
fn execute_burn(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    endowment_id: Option<u32>,
    amount: Uint128,
) -> Result<(), ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // lower balance
    match endowment_id {
        Some(id) => BALANCES.update(
            deps.storage,
            id,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?,
        None => APTAX.update(deps.storage, |balance: Uint128| -> StdResult<_> {
            Ok(balance.checked_sub(amount)?)
        })?,
    };
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    Ok(())
}

/// Query the `asset` balance of `account_addr`
fn query_asset_balance(
    deps: Deps,
    account_addr: Addr,
    asset_info: AssetInfo,
) -> StdResult<Uint128> {
    match asset_info {
        AssetInfo::NativeToken { denom } => query_balance(&deps.querier, account_addr, denom),
        AssetInfo::Token { contract_addr } => query_token_balance(
            &deps.querier,
            deps.api.addr_validate(&contract_addr)?,
            account_addr,
        ),
    }
}
