use angel_core::utils::{query_balance, query_token_balance};
use cosmwasm_std::{
    attr, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfoBase as CwAssetInfoBase;
use terraswap::asset::{Asset, AssetInfo};

use angel_core::errors::vault::ContractError;
use angel_core::msgs::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::msgs::swap_router::ExecuteMsg as SwapRouterExecuteMsg;
use angel_core::msgs::vault::{
    ExecuteMsg, LoopFarmingExecuteMsg, LoopFarmingQueryMsg, LoopPairExecuteMsg, ReceiveMsg,
    UpdateConfigMsg,
};
use angel_core::msgs::{accounts::EndowmentDetailsResponse, registrar::ConfigResponse};
use angel_core::structs::{AccountType, SwapOperation};
use terraswap::querier::query_pair_info_from_pair;

use crate::state::{Config, State, APTAX, BALANCES, CONFIG, STATE, TOKEN_INFO};

// Initial VT(vault token) mint amount
const INIT_VT_MINT_AMOUNT: u128 = 1000000; // 1 VT

// Number of blocks until pending owner update is valid
pub const PENDING_OWNER_DEADLINE: u64 = 42069;

/// Contract entry: **UpdateOwner**
pub fn update_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // 2-step process of updating `config.owner`
    //
    // 1. Current `config.owner` suggests `new_owner` address
    //    - At this moment, the `pending_owner` is set with `new_owner` address.
    //    - Also, the `pending_owner_deadline` is set as current block height + constant DEADLINE height
    //
    // 2. The `pending_owner`(new_owner) completes the process & becomes the `config.owner`,
    //    OR the settings are unset for future process.
    //    - `pending_owner` address calls this entry(update_owner)
    //    - If the `pending_owner_deadline` is NOT reached at the moment of execution,
    //         the `pending_owner` becomes `config.owner`.
    //      If not, the `config.owner` remains unchanged.
    //    - All settings(`pending_owner` & `pending_owner_deadline`) are unset for future.

    match (config.pending_owner, config.pending_owner_deadline) {
        (None, None) => {
            if info.sender != config.owner {
                return Err(ContractError::Unauthorized {});
            }
            let new_owner = deps.api.addr_validate(&new_owner)?;
            config.pending_owner = Some(new_owner);
            config.pending_owner_deadline = Some(env.block.height + PENDING_OWNER_DEADLINE);
        }
        (Some(pending_owner), Some(deadline)) => {
            if info.sender != pending_owner {
                return Err(ContractError::Unauthorized {});
            }
            if env.block.height <= deadline {
                config.owner = pending_owner;
            }
            config.pending_owner = None;
            config.pending_owner_deadline = None;
        }
        _ => {
            return Err(ContractError::Std(StdError::generic_err(
                "Invalid owner update settings",
            )))
        }
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Contract entry: **UpdateRegistrar**
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

/// Contract entry: **UpdateConfig**
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

    config.keeper = match msg.keeper {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.keeper,
    };
    config.tax_collector = match msg.tax_collector {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.tax_collector,
    };

    config.native_token = match msg.native_token {
        None => config.native_token,
        Some(CwAssetInfoBase::Native(denom)) => AssetInfo::NativeToken { denom },
        Some(CwAssetInfoBase::Cw20(contract_addr)) => AssetInfo::Token {
            contract_addr: contract_addr.to_string(),
        },
        _ => unreachable!(),
    };
    config.reward_to_native_route = match msg.reward_to_native_route {
        Some(ops) => ops,
        None => config.reward_to_native_route,
    };
    config.native_to_lp0_route = match msg.native_to_lp0_route {
        Some(ops) => ops,
        None => config.native_to_lp0_route,
    };
    config.native_to_lp1_route = match msg.native_to_lp1_route {
        Some(ops) => ops,
        None => config.native_to_lp1_route,
    };

    config.minimum_initial_deposit = match msg.minimum_initial_deposit {
        Some(v) => v,
        None => config.minimum_initial_deposit,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Contract entry: **Deposit**
///   1. Swap the `native_token` to lp contract pair tokens
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

    // Check if the `deposit_asset_info` is valid
    if deposit_asset_info != config.native_token {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    if config.native_token != config.lp_pair_token0
        && config.native_token != config.lp_pair_token1
        && config.native_to_lp0_route.is_empty()
        && config.native_to_lp1_route.is_empty()
    {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot find a way to swap native token to pair tokens".to_string(),
        }));
    }

    // Check if the `deposit_amount` is zero
    if deposit_amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // Here, we take care of 2 cases.
    //  - The `native_token` is either of `lp_pair_token0` or `lp_pair_token1`
    //  - The `native_token` is not any of lp pair tokens
    let mut swap_router_swap_msgs = vec![];
    let mut loop_pair_swap_msgs = vec![];
    let mut contract_add_liquidity_msgs = vec![];
    if deposit_asset_info == config.lp_pair_token0 || deposit_asset_info == config.lp_pair_token1 {
        // Swap the half of `native_token`(`deposit`) to lp contract pair token
        loop_pair_swap_msgs.extend_from_slice(&prepare_loop_pair_swap_msg(
            config.lp_pair_contract.as_ref(),
            &deposit_asset_info,
            deposit_amount.multiply_ratio(1_u128, 2_u128),
        )?);

        // Call the "(this contract::)add_liquidity" entry
        contract_add_liquidity_msgs.extend_from_slice(&prepare_contract_add_liquidity_msgs(
            deps,
            env,
            &config,
            Some(endowment_id),
            Some(deposit_asset_info),
            Some(deposit_amount),
        )?);
    } else {
        // Swap the half of `native_token`(`deposit`) to the `lp_pair_token0`, and another half to `lp_pair_token1`.
        let swap_amount = deposit_amount.multiply_ratio(1_u128, 2_u128);
        swap_router_swap_msgs.extend_from_slice(&prepare_swap_router_swap_msgs(
            config.swap_router.to_string(),
            config.native_token.clone(),
            swap_amount,
            config.native_to_lp0_route.clone(),
        )?);
        swap_router_swap_msgs.extend_from_slice(&prepare_swap_router_swap_msgs(
            config.swap_router.to_string(),
            config.native_token.clone(),
            swap_amount,
            config.native_to_lp1_route.clone(),
        )?);

        // Call the "(this contract::)add_liquidity" entry
        contract_add_liquidity_msgs.extend_from_slice(&prepare_contract_add_liquidity_msgs(
            deps,
            env,
            &config,
            Some(endowment_id),
            None,
            None,
        )?);
    }

    Ok(Response::default()
        .add_messages(swap_router_swap_msgs)
        .add_messages(loop_pair_swap_msgs)
        .add_messages(contract_add_liquidity_msgs)
        .add_attribute("action", "deposit")
        .add_attribute("sender", msg_sender)
        .add_attribute("endow_id", endowment_id.to_string())
        .add_attribute("deposit_amount", deposit_amount))
}

/// Contract entry: **RestakeClaimReward**
///   1. Compute the amount of `lp_reward_token` generated from `harvest(claim)`
///   2. Convert the `lp_reward_token`(`LOOP`)s to the LP tokens
///   3. Re-stake the LP tokens
pub fn restake_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    // Check if the caller is this contract itself.
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Compute the `lp_reward_token` amount
    let reward_token_bal = query_token_balance(
        deps.as_ref(),
        config.lp_reward_token.to_string(),
        env.contract.address.to_string(),
    )?;
    let reward_amount = reward_token_bal
        .checked_sub(reward_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;

    if reward_amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }

    // Re-stake the `reward token`s for more yield
    // NOTE: This logic is similar to the `Deposit` entry logic, taking care of 2 cases.
    let reward_asset_info = AssetInfo::Token {
        contract_addr: config.lp_reward_token.to_string(),
    };
    let mut swap_router_swap_msgs = vec![];
    let mut loop_pair_swap_msgs = vec![];
    let mut contract_add_liquidity_msgs = vec![];
    if reward_asset_info == config.lp_pair_token0 || reward_asset_info == config.lp_pair_token1 {
        // Swap the half of `reward_token` to the lp contract pair token
        loop_pair_swap_msgs.extend_from_slice(&prepare_loop_pair_swap_msg(
            config.lp_pair_contract.as_ref(),
            &reward_asset_info,
            reward_amount.multiply_ratio(1_u128, 2_u128),
        )?);

        // Call the `(this contract::)add_liquidity` entry
        contract_add_liquidity_msgs.extend_from_slice(&prepare_contract_add_liquidity_msgs(
            deps,
            env,
            &config,
            None,
            Some(reward_asset_info),
            Some(reward_amount),
        )?);
    } else {
        let swap_amount = reward_amount.multiply_ratio(1_u128, 2_u128);
        let start_token = AssetInfo::Token {
            contract_addr: config.lp_reward_token.to_string(),
        };
        // Swap the half of input token to `lp_pair_token0`
        let operations = [
            config.reward_to_native_route.clone(),
            config.native_to_lp0_route.clone(),
        ]
        .concat();
        swap_router_swap_msgs.extend_from_slice(&prepare_swap_router_swap_msgs(
            config.swap_router.to_string(),
            start_token.clone(),
            swap_amount,
            operations,
        )?);

        // Swap the half of input token to `lp_pair_token1`
        let operations = [
            config.reward_to_native_route.clone(),
            config.native_to_lp1_route.clone(),
        ]
        .concat();
        swap_router_swap_msgs.extend_from_slice(&prepare_swap_router_swap_msgs(
            config.swap_router.to_string(),
            start_token,
            swap_amount,
            operations,
        )?);

        // Call the `(this contract::)add_liquidity` entry
        contract_add_liquidity_msgs =
            prepare_contract_add_liquidity_msgs(deps, env, &config, None, None, None)?;
    }

    Ok(Response::default()
        .add_messages(swap_router_swap_msgs)
        .add_messages(loop_pair_swap_msgs)
        .add_messages(contract_add_liquidity_msgs)
        .add_attributes(vec![attr("action", "restake_claimed_reward")]))
}

/// Contract entry: **Redeem**
///   1. Unstake/unfarm the LP tokens from the `loopswap::farming` contract
///   2. Re-stake the `lp_reward_token`
///   3. Swap the lp tokens back to the `native_token`, & send them to the `accounts_contract`
pub fn redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: u32,
    burn_shares_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

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
        let registar_config: ConfigResponse = deps.querier.query_wasm_smart(
            config.registrar_contract.to_string(),
            &RegistrarQueryMsg::Config {},
        )?;
        let accounts_contract = deps
            .api
            .addr_validate(&registar_config.accounts_contract.unwrap())?;
        beneficiary = accounts_contract;
        id = Some(endowment_id);
    }

    // First, burn the vault tokens
    // The formula of calculating the amount of LP tokens to be burnt is as follows:
    //   s = vault shares to burn
    //   T = vault shares total (before burn)
    //   a = LP tokens withdraw to Vault's balance <<< what we need to calculate given some # of Vault shares to be burned
    //   B = Vault's total LP Token balance
    //
    //   a = (s * B) / T
    execute_burn(deps.branch(), env.clone(), info, id, burn_shares_amount).map_err(|e| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {} :: {}",
                burn_shares_amount,
                id.map_or_else(|| beneficiary.to_string(), |v| format!("Endowment {}", v)),
                e,
            ),
        })
    })?;

    // Update the contract state
    let lp_2_vt_rate = Decimal::from_ratio(state.total_lp_amount, state.total_shares);
    let lp_amount = burn_shares_amount * lp_2_vt_rate;
    state.total_lp_amount -= lp_amount;
    state.total_shares -= burn_shares_amount;

    STATE.save(deps.storage, &state)?;

    // Call the `loopswap::farming::unstake_and_claim(unfarm)` entry
    let mut msgs = vec![];
    let lp_token_contract = config.lp_token;
    let flp_token_contract: String = deps.querier.query_wasm_smart(
        config.lp_staking_contract.to_string(),
        &LoopFarmingQueryMsg::QueryFlpTokenFromPoolAddress {
            pool_address: lp_token_contract.to_string(),
        },
    )?;

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: flp_token_contract,
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::UnstakeAndClaim {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens (Swap back to `native_token` & send to `beneficiary`)
    let lp_token_bal = query_token_balance(
        deps.as_ref(),
        lp_token_contract.to_string(),
        env.contract.address.to_string(),
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before: lp_token_bal,
            beneficiary,
            id,
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the `lp_reward_token`s (Re-stake them for more yield)
    let reward_token_bal = query_token_balance(
        deps.as_ref(),
        config.lp_reward_token.to_string(),
        env.contract.address.to_string(),
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: reward_token_bal,
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

/// Contract entry: **Harvest**
///   1. Claim(Harvest) the `lp_reward_token` from `loopswap::farming` contract
///   2. Convert the `lp_reward_token` to LP tokens
///   3. Re-stake the LP tokens to the `farming` contract for more yield
pub fn harvest(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if the caller is `keeper` address
    if info.sender != config.keeper {
        return Err(ContractError::Unauthorized {});
    }

    let mut msgs = vec![];

    // Call the `loopswap::farming::claim_reward` entry
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_staking_contract.to_string(),
        msg: to_binary(&LoopFarmingExecuteMsg::ClaimReward {}).unwrap(),
        funds: vec![],
    }));

    // Re-stake the `lp_reward_token`
    let reward_token_bal = query_token_balance(
        deps.as_ref(),
        config.lp_reward_token.to_string(),
        env.contract.address.to_string(),
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: reward_token_bal,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(Response::default()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "harvest")]))
}

/// Contract entry: **ReinvestToLocked** (liquid vault logic)
///   1. Burn the `vault_token`
///   2. Unstake the LP tokens from `farming` contract
///   3. Re-stake the lp_reward_token from `unstake` operation
///   4. Send the unstaked LP tokens to the sibling vault(locked)
pub fn reinvest_to_locked_execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    burn_shares_amount: Uint128, // vault tokens
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    // Check that the vault acct_type is `liquid`
    if config.acct_type != AccountType::Liquid {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "This is locked vault".to_string(),
        }));
    }

    // 0. Check that the message sender is the Accounts contract
    validate_action_caller_n_endow_id(deps.as_ref(), &config, info.sender.to_string(), id)?;
    // 1. Check that this vault has a sibling set
    if config.sibling_vault == env.contract.address {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Sibling vault not created".to_string(),
        }));
    }
    // 2. Check that sender ID has >= amount of vault tokens in it's balance
    let endowment_vt_balance = crate::queriers::query_balance(deps.as_ref(), id);
    if burn_shares_amount > endowment_vt_balance {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Insufficient balance: Needed {}, existing: {}",
                burn_shares_amount, endowment_vt_balance
            ),
        }));
    }
    // 3. Burn vault tokens an calculate the LP Tokens equivalent
    //
    // First, burn the vault tokens
    // The formula of calculating the amount of LP tokens to be burnt is as follows:
    //   s = vault shares to burn
    //   T = vault shares total (before burn)
    //   a = LP tokens withdraw to Vault's balance <<< what we need to calculate given some # of Vault shares to be burned
    //   B = Vault's total LP Token balance
    //
    //   a = (s * B) / T
    execute_burn(
        deps.branch(),
        env.clone(),
        info,
        Some(id),
        burn_shares_amount,
    )
    .map_err(|e| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {} :: {}",
                burn_shares_amount, id, e,
            ),
        })
    })?;

    // Update the contract state
    let lp_2_vt_rate = Decimal::from_ratio(state.total_lp_amount, state.total_shares);
    let lp_amount = burn_shares_amount * lp_2_vt_rate;
    state.total_lp_amount -= lp_amount;
    state.total_shares -= burn_shares_amount;

    STATE.save(deps.storage, &state)?;

    // Unfarm the LP token from "lp_staking_contract"
    let lp_token_contract = config.lp_token.clone();
    let flp_token_contract: String = deps.querier.query_wasm_smart(
        config.lp_staking_contract.to_string(),
        &LoopFarmingQueryMsg::QueryFlpTokenFromPoolAddress {
            pool_address: lp_token_contract.to_string(),
        },
    )?;

    let unstake_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: flp_token_contract,
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::UnstakeAndClaim {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    })];

    // 4. SEND LP tokens to the Locked Account (using ReinvestToLocked receive msg)
    let reinvest_to_locked_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_token.to_string(),
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
    let reward_token_bal = query_token_balance(
        deps.as_ref(),
        config.lp_reward_token.to_string(),
        env.contract.address.to_string(),
    )?;
    let restake_reward_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before: reward_token_bal,
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

/// Contract entry: **ReinvestToLocked** (locked vault logic)
///   1. Receive the LP tokens from sibling vault(liquid)
///   2. Stake the LP tokens to the `farming` contract
pub fn reinvest_to_locked_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u32,
    lp_amount: Uint128, // asset tokens (ex. LPs)
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    // 0. Check that the message sender is the Sibling vault contract
    // ensure `received token` is `lp_token`
    if info.sender != config.lp_token {
        return Err(ContractError::Unauthorized {});
    }

    // ensure this vault is "locked"
    if config.acct_type != AccountType::Locked {
        return Err(ContractError::Unauthorized {});
    }

    // ensure `msg_sender` is sibling_vault(liquid)
    let msg_sender = cw20_msg.sender;
    let sibling_config_resp: angel_core::msgs::vault::ConfigResponse =
        deps.querier.query_wasm_smart(
            config.sibling_vault.to_string(),
            &angel_core::msgs::vault::QueryMsg::Config {},
        )?;
    if msg_sender != config.sibling_vault || sibling_config_resp.acct_type != AccountType::Liquid {
        return Err(ContractError::Unauthorized {});
    }

    // ensure the `amount` matches `sent_amount`.
    let sent_amount = cw20_msg.amount;
    if lp_amount != sent_amount {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Balance does not match: Received: {}, Expected: {}",
                sent_amount, lp_amount
            ),
        }));
    }

    // 1. Treat as a Deposit for the given ID (mint vault tokens for deposited assets)
    // Prepare the messages for "loop::farming::stake" opeartion
    let lp_stake_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_token.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    })];

    // 2. Mint the vault tokens
    //
    // Compute the `vault_token` amount
    // The formula of calculating the amount of vault tokens to be minted is as follows:
    //   s = vault shares to mint <<< what we need to calculate given some # of LP tokens created from despoit
    //   T = vault shares total (before mint)
    //   a = LP tokens added to Vault's balance
    //   B = Vault's total LP Token balance (before deposit)
    //
    //   s = (a * T) / B = a * (T / B)
    let vt_mint_amount = match state.total_shares.u128() {
        0 => {
            if lp_amount < config.minimum_initial_deposit {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: format!(
                        "Received {}, should be bigger than {}.",
                        lp_amount, config.minimum_initial_deposit
                    ),
                }));
            }
            Uint128::from(INIT_VT_MINT_AMOUNT)
        }
        _ => lp_amount * Decimal::from_ratio(state.total_shares, state.total_lp_amount),
    };
    state.total_lp_amount += lp_amount;
    state.total_shares += vt_mint_amount;
    STATE.save(deps.storage, &state)?;

    // Mint the `vault_token`
    let minter_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    execute_mint(deps, env, minter_info, Some(id), vt_mint_amount).map_err(|e| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot mint the {} vault token for {}:: {}",
                vt_mint_amount, id, e
            ),
        })
    })?;

    Ok(Response::new()
        .add_messages(lp_stake_msgs)
        .add_attributes(vec![attr("action", "reinvest_to_locked_vault")]))
}

/// Contract entry: **AddLiquidity**
///   1. Add/Provide the `liquidity` to the `loopswap::pair` contract
///   2. Call the `(this contract::)stake` entry
pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: Option<u32>,
    lp_pair_token0_bal_before: Uint128,
    lp_pair_token1_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Add the `loopswap::pair::provide_liquidity` message
    let config = CONFIG.load(deps.storage)?;

    let lp_pair_token0_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token0.clone(),
    )?;
    let lp_pair_token1_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token1.clone(),
    )?;

    let token0_amount = lp_pair_token0_bal - lp_pair_token0_bal_before;
    let token1_amount = lp_pair_token1_bal - lp_pair_token1_bal_before;

    let loop_pair_provide_liquidity_msgs = prepare_loop_pair_provide_liquidity_msgs(
        config.lp_pair_token0.clone(),
        token0_amount,
        config.lp_pair_token1.clone(),
        token1_amount,
        config.lp_pair_contract.to_string(),
    )?;

    // Add the `(this contract::)stake` message
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
        contract_addr: pair_contract,
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

    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let mut harvest_to_liquid_msgs = vec![];

    // Prepare the "loop::farming::stake" msg
    let lp_token_bal = query_token_balance(
        deps.as_ref(),
        config.lp_token.to_string(),
        env.contract.address.to_string(),
    )?;
    let lp_amount = lp_token_bal
        .checked_sub(lp_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;
    if lp_amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let lp_stake_amount: Uint128;

    match endowment_id {
        // Case of `deposit` from `endowment`
        Some(endowment_id) => {
            // Compute the `vault_token` amount
            // The formula of calculating the amount of vault tokens to be minted is as follows:
            //   s = vault shares to mint <<< what we need to calculate given some # of LP tokens created from despoit
            //   T = vault shares total (before mint)
            //   a = LP tokens added to Vault's balance
            //   B = Vault's total LP Token balance (before deposit)
            //
            //   s = (a * T) / B = a * (T / B)
            let vt_mint_amount = match state.total_shares.u128() {
                0 => {
                    if lp_amount < config.minimum_initial_deposit {
                        return Err(ContractError::Std(StdError::GenericErr {
                            msg: format!(
                                "Received {}, should be bigger than {}.",
                                lp_amount, config.minimum_initial_deposit
                            ),
                        }));
                    }
                    Uint128::from(INIT_VT_MINT_AMOUNT)
                }
                _ => lp_amount * Decimal::from_ratio(state.total_shares, state.total_lp_amount),
            };
            state.total_lp_amount += lp_amount;
            state.total_shares += vt_mint_amount;

            // Mint the `vault_token`
            execute_mint(deps.branch(), env, info, Some(endowment_id), vt_mint_amount).map_err(
                |e| {
                    ContractError::Std(StdError::GenericErr {
                        msg: format!(
                            "Cannot mint the {} vault token for {} :: {}",
                            vt_mint_amount, endowment_id, e
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
            // The formula of calculating the amount of vault tokens to be minted is as follows:
            //   s = vault shares to mint <<< what we need to calculate given some # of LP tokens created from despoit
            //   T = vault shares total (before mint)
            //   a = LP tokens added to Vault's balance
            //   B = Vault's total LP Token balance (before deposit)
            //
            //   s = (a * T) / B = a * (T / B)
            let vt_mint_amount = match state.total_shares.u128() {
                0 => {
                    if lp_amount < config.minimum_initial_deposit {
                        return Err(ContractError::Std(StdError::GenericErr {
                            msg: format!(
                                "Received {}, should be bigger than {}.",
                                lp_amount, config.minimum_initial_deposit
                            ),
                        }));
                    }
                    Uint128::from(INIT_VT_MINT_AMOUNT)
                }
                _ => lp_amount * Decimal::from_ratio(state.total_shares, state.total_lp_amount),
            };
            state.total_lp_amount += lp_amount;
            state.total_shares += vt_mint_amount;

            // Compute the `ap_treasury tax portion` & store in APTAX
            let registrar_config: ConfigResponse = deps.querier.query_wasm_smart(
                config.registrar_contract.to_string(),
                &RegistrarQueryMsg::Config {},
            )?;

            let tax_rate: Decimal = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQueryMsg::Fee {
                    name: "vaults_harvest".to_string(),
                })?,
            }))?;
            let tax_mint_amount = vt_mint_amount * tax_rate;

            APTAX.update(deps.storage, |balance: Uint128| -> StdResult<_> {
                Ok(balance.checked_add(tax_mint_amount)?)
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
                    let lp_tax_amount = lp_amount * tax_rate;
                    let lp_less_tax = lp_amount - lp_tax_amount;

                    let send_liquid_ratio = registrar_config.rebalance.interest_distribution;
                    let send_liquid_lp_amount = lp_less_tax * send_liquid_ratio;
                    let vt_shares_to_burn = (vt_mint_amount - tax_mint_amount) * send_liquid_ratio;
                    state.total_shares -= vt_shares_to_burn;
                    state.total_lp_amount -= send_liquid_lp_amount;

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
                        contract_addr: config.lp_token.to_string(),
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
    STATE.save(deps.storage, &state)?;

    let farming_stake_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_token.to_string(),
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

/// Contract entry: **HarvestToLiquid**
///   1. Stake/Farm the `LP` tokens received from `sibling_vault(locked)`
pub fn harvest_to_liquid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let lp_token_contract = config.lp_token.to_string();

    // 0. Check that the message sender is the Sibling vault contract
    // ensure `received token` is `lp_token`
    if info.sender != lp_token_contract {
        return Err(ContractError::Unauthorized {});
    }

    // ensure this vault is "liquid"
    if config.acct_type != AccountType::Liquid {
        return Err(ContractError::Unauthorized {});
    }

    // ensure `msg_sender` is sibling_vault(locked)
    let msg_sender = cw20_msg.sender;
    let sibling_config_resp: angel_core::msgs::vault::ConfigResponse =
        deps.querier.query_wasm_smart(
            config.sibling_vault.to_string(),
            &angel_core::msgs::vault::QueryMsg::Config {},
        )?;
    if msg_sender != config.sibling_vault || sibling_config_resp.acct_type != AccountType::Locked {
        return Err(ContractError::Unauthorized {});
    }

    // 1. Increase the lp_token amount
    let lp_stake_amount = cw20_msg.amount;
    state.total_lp_amount += lp_stake_amount;
    STATE.save(deps.storage, &state)?;

    // 2. Stake the received lp tokens
    let farming_stake_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract,
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

/// Contract entry: **RemoveLiquidity**
///   1. Remove liquidity from lp pair contract
///   2. Swap back the lp pair tokens to the `native_token`
///   2. Call the `(this contract::)SendAsset` entry
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

    let config: Config = CONFIG.load(deps.storage)?;

    // First, compute "unfarm"ed LP token balance for "remove_liquidity"
    let lp_token_bal = query_token_balance(
        deps.as_ref(),
        config.lp_token.to_string(),
        env.contract.address.to_string(),
    )?;

    let lp_token_amount = lp_token_bal
        .checked_sub(lp_token_bal_before)
        .map_err(|e| ContractError::Std(StdError::Overflow { source: e }))?;

    // Prepare the "remove_liquidity" messages
    let withdraw_liquidity_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_token.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_pair_contract.to_string(),
            amount: lp_token_amount,
            msg: to_binary(&LoopPairExecuteMsg::WithdrawLiquidity {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    })];

    // Convert the returning token pairs to the `native_token`
    // & send back to `accounts_contract`.
    let mut swap_back_msgs = vec![];
    let lp_pair_token0_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token0.clone(),
    )?;
    let lp_pair_token1_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token1.clone(),
    )?;
    swap_back_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SwapBack {
            lp_pair_token0_bal_before: lp_pair_token0_bal,
            lp_pair_token1_bal_before: lp_pair_token1_bal,
        })
        .unwrap(),
        funds: vec![],
    }));

    let native_token_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.native_token,
    )?;
    let send_asset_msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::SendAsset {
            beneficiary,
            id,
            native_token_bal_before: native_token_bal,
        })
        .unwrap(),
        funds: vec![],
    })];

    Ok(Response::default()
        .add_messages(withdraw_liquidity_msgs)
        .add_messages(swap_back_msgs)
        .add_messages(send_asset_msgs)
        .add_attributes(vec![attr("action", "remove_liquidity")]))
}

/// Contract entry: **SendAsset**
///   1. Send the `native_token` back to the `beneficiary`
pub fn send_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Addr,
    id: Option<u32>,
    native_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config: Config = CONFIG.load(deps.storage)?;

    // Send the asset to the `beneficiary`

    let native_token_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address,
        config.native_token.clone(),
    )?;
    let send_amount = native_token_bal - native_token_bal_before;

    let mut msgs = vec![];
    match config.native_token {
        AssetInfo::NativeToken { denom } => match id {
            Some(id) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: beneficiary.to_string(),
                msg: to_binary(&angel_core::msgs::accounts::ExecuteMsg::VaultReceipt {
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
                    contract_addr,
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                        contract: beneficiary.to_string(),
                        amount: send_amount,
                        msg: to_binary(&angel_core::msgs::accounts::ReceiveMsg::VaultReceipt {
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
                contract_addr,
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

/// Contract entry: **SwapBack**
///   1. Swap lp pair tokens to the `native_token`s
pub fn swap_back(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lp_pair_token0_bal_before: Uint128,
    lp_pair_token1_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config: Config = CONFIG.load(deps.storage)?;

    // Swap the pair tokens back to the `native_token`.
    let mut swap_router_swap_msgs = vec![];
    let mut loop_pair_swap_msgs = vec![];
    let lp_pair_token0_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token0.clone(),
    )?;
    let lp_pair_token1_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address,
        config.lp_pair_token1.clone(),
    )?;

    if config.native_token == config.lp_pair_token0 || config.native_token == config.lp_pair_token1
    {
        let (input_asset_info, swap_amount) = if config.native_token == config.lp_pair_token0 {
            (
                config.lp_pair_token1,
                lp_pair_token1_bal - lp_pair_token1_bal_before,
            )
        } else {
            (
                config.lp_pair_token0,
                lp_pair_token0_bal - lp_pair_token0_bal_before,
            )
        };
        loop_pair_swap_msgs.extend_from_slice(&prepare_loop_pair_swap_msg(
            config.lp_pair_contract.as_str(),
            &input_asset_info,
            swap_amount,
        )?);
    } else {
        let swap_amount = lp_pair_token0_bal - lp_pair_token0_bal_before;
        let operations = config
            .native_to_lp0_route
            .iter()
            .rev()
            .map(|op| op.reverse_operation())
            .collect();

        swap_router_swap_msgs.extend_from_slice(&prepare_swap_router_swap_msgs(
            config.swap_router.to_string(),
            config.lp_pair_token0.clone(),
            swap_amount,
            operations,
        )?);

        let swap_amount = lp_pair_token1_bal - lp_pair_token1_bal_before;
        let operations = config
            .native_to_lp1_route
            .iter()
            .rev()
            .map(|op| op.reverse_operation())
            .collect();
        swap_router_swap_msgs.extend_from_slice(&prepare_swap_router_swap_msgs(
            config.swap_router.to_string(),
            config.lp_pair_token1,
            swap_amount,
            operations,
        )?);
    }

    Ok(Response::default()
        .add_messages(swap_router_swap_msgs)
        .add_messages(loop_pair_swap_msgs)
        .add_attributes(vec![attr("action", "swap_pair_to_native")]))
}

fn prepare_swap_router_swap_msgs(
    swap_router: String,
    start_token: AssetInfo,
    swap_amount: Uint128,
    operations: Vec<SwapOperation>,
) -> StdResult<Vec<CosmosMsg>> {
    let msgs = match start_token {
        AssetInfo::NativeToken { ref denom } => vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: swap_router,
            msg: to_binary(&SwapRouterExecuteMsg::ExecuteSwapOperations {
                operations,
                minimum_receive: None,
                endowment_id: 1,                // Placeholder value
                acct_type: AccountType::Locked, // Placeholder value
            })
            .unwrap(),
            funds: coins(swap_amount.u128(), denom.to_string()),
        })],
        AssetInfo::Token { ref contract_addr } => vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: swap_router,
                amount: swap_amount,
                msg: to_binary(&SwapRouterExecuteMsg::ExecuteSwapOperations {
                    operations,
                    minimum_receive: None,
                    endowment_id: 1,                // Placeholder value
                    acct_type: AccountType::Locked, // Placeholder value
                })
                .unwrap(),
            })
            .unwrap(),
            funds: vec![],
        })],
    };
    Ok(msgs)
}

fn prepare_contract_add_liquidity_msgs(
    deps: DepsMut,
    env: Env,
    config: &Config,
    endowment_id: Option<u32>,
    deduct_token: Option<AssetInfo>,
    deduct_amount: Option<Uint128>,
) -> Result<Vec<CosmosMsg>, StdError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    let mut lp_pair_token0_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token0.clone(),
    )?;
    let mut lp_pair_token1_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        config.lp_pair_token1.clone(),
    )?;

    if let Some(deduct_token_info) = deduct_token {
        if deduct_token_info == config.lp_pair_token0 {
            lp_pair_token0_bal -= deduct_amount.expect("Deduct amount not set.");
        } else if deduct_token_info == config.lp_pair_token1 {
            lp_pair_token1_bal -= deduct_amount.expect("Deduct amount not set.");
        } else {
            return Err(StdError::GenericErr {
                msg: "Cannot deduct token amount of native or reward token when they are either of lp pair tokens".to_string(),
            });
        }
    }
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::AddLiquidity {
            endowment_id,
            lp_pair_token0_bal_before: lp_pair_token0_bal,
            lp_pair_token1_bal_before: lp_pair_token1_bal,
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
    if let Some(ref accounts_contract) = registar_config.accounts_contract {
        if caller != *accounts_contract {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    // Check that the "deposit-endowment-id" is an Accounts SC
    let _endowments_rsp: EndowmentDetailsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: registar_config.accounts_contract.unwrap(),
            msg: to_binary(&angel_core::msgs::accounts::QueryMsg::Endowment { id: endowment_id })
                .unwrap(),
        }))?;

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
        return Err(ContractError::ZeroAmount {});
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
            Ok(balance.checked_add(amount)?)
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
        return Err(ContractError::ZeroAmount {});
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
        AssetInfo::NativeToken { denom } => query_balance(deps, account_addr.to_string(), denom),
        AssetInfo::Token { contract_addr } => {
            query_token_balance(deps, contract_addr, account_addr.to_string())
        }
    }
}
