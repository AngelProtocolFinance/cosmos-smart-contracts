use cosmwasm_std::{
    attr, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, Fraction,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use terraswap::asset::{Asset, AssetInfo};

use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{
    AccountWithdrawMsg, ExecuteMsg, LoopFarmingExecuteMsg, LoopPairExecuteMsg, RemoveLiquidAction,
    UpdateConfigMsg,
};
use angel_core::responses::registrar::{ConfigResponse, EndowmentListResponse};
use angel_core::structs::EndowmentEntry;
use terraswap::querier::{query_balance, query_pair_info_from_pair, query_token_balance};

use crate::state::{Config, BALANCES, CONFIG, TOKEN_INFO};

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

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Update the config
    config.lp_staking_contract = match msg.lp_staking_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.lp_staking_contract,
    };

    config.pair_contract = match msg.pair_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.pair_contract,
    };

    config.keeper = match msg.keeper {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.keeper,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

/// Contract entry: **deposit**
///   1. Add the `swap` message of the `loopswap pair` contract
///   2. Call the `(this contract::)add_liquidity` action afterwards
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

    // Validations
    validate_action_caller_n_endow_id(deps.as_ref(), &config, msg_sender.clone(), endowment_id)?;

    // Check if the "deposit_asset_info" is valid
    let pair_info_query: terraswap::asset::PairInfo =
        query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?;
    if !pair_info_query.asset_infos.contains(&deposit_asset_info) {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    // Check if the "deposit_amount" is zero
    if deposit_amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // Add the "loopswap::pair::swap" message
    let input_amount = deposit_amount.multiply_ratio(1_u128, 2_u128);
    let loop_pair_swap_msgs = prepare_loop_pair_swap_msg(
        &config.pair_contract.to_string(),
        &deposit_asset_info,
        input_amount,
    )?;

    // Prepare the messages for "(this contract::)add_liquidity" opeartion
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

/// Contract entry: **claim**
///   1. Only callable by `accounts` contract
///   2. `Claim` the `reward token` from `loopswap::farming` contract
///   3. Compute and send the tax(USDC) to AP treasury
///   4. Re-stake the leftover by converting it to LP token & `loopswap::farming::stake`
pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Validations
    let registar_config: ConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &RegistrarQueryMsg::Config {},
    )?;
    if let Some(accounts_contract) = registar_config.accounts_contract {
        if info.sender.to_string() != accounts_contract {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        return Err(ContractError::Unauthorized {});
    }

    let msgs = prepare_claim_harvest_msgs(deps.as_ref(), env, &config)?;

    Ok(Response::default()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "claim")]))
}

/// Contract entry: **distribute_claim**
///   1. Compute the amount of reward token(`LOOP`) obtained from `claim/harvest`
///   2. Compute the `AP tax` & send it to the `AP treasury`
///   3. Re-stake the left `reward token`s
pub fn distribute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config: Config = CONFIG.load(deps.storage)?;

    // Compute the reward token amount
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

    // Handle the `AP tax`
    let mut tax_send_msgs: Vec<CosmosMsg> = vec![];
    let registrar_config: ConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &RegistrarQueryMsg::Config {},
    )?;
    let tax_rate = registrar_config.tax_rate;
    let ap_treasury = deps.api.addr_validate(&registrar_config.treasury)?;
    let tax_amount = reward_amount.multiply_ratio(tax_rate.numerator(), tax_rate.denominator());
    tax_send_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::Swap {
            beneficiary: Some(ap_treasury),
            in_asset_info: AssetInfo::Token {
                contract_addr: config.lp_reward_token.to_string(),
            },
            in_asset_bal_before: reward_token_bal_query.balance - tax_amount,
        })
        .unwrap(),
        funds: vec![],
    }));

    // Re-stake the leftover `reward token`s
    let less_tax = reward_amount - tax_amount;
    let loop_pair_swap_msgs = prepare_loop_pair_swap_msg(
        &config.pair_contract.to_string(),
        &AssetInfo::Token {
            contract_addr: config.lp_reward_token.to_string(),
        },
        less_tax.multiply_ratio(1_u128, 2_u128),
    )?;

    // Prepare the messages for "(this contract::)add_liquidity" opeartion
    let contract_add_liquidity_msgs = prepare_contract_add_liquidity_msgs(
        deps,
        env,
        &config,
        None,
        AssetInfo::Token {
            contract_addr: config.lp_reward_token.to_string(),
        },
    )?;

    Ok(Response::default()
        .add_messages(tax_send_msgs)
        .add_messages(loop_pair_swap_msgs)
        .add_messages(contract_add_liquidity_msgs)
        .add_attributes(vec![attr("action", "distribute_claim")]))
}

fn prepare_claim_harvest_msgs(deps: Deps, env: Env, config: &Config) -> StdResult<Vec<CosmosMsg>> {
    // Performs the "claim"
    let mut msgs = vec![];
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.lp_staking_contract.to_string(),
        msg: to_binary(&LoopFarmingExecuteMsg::ClaimReward {}).unwrap(),
        funds: vec![],
    }));

    // Handle the returning reward token(LOOP)
    let reward_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.lp_reward_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::DistributeClaim {
            reward_token_bal_before: reward_token_bal_query.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(msgs)
}

/// Contract entry: **withdraw**
///   1. Unstake/Unfarm the LP tokens from the `loopswap::farming` contract
///   2. Handle the receiving LP token & reward token(LOOP)
pub fn withdraw(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let burn_shares_amount = msg.amount;

    // Validations
    validate_action_caller_n_endow_id(
        deps.as_ref(),
        &config,
        info.sender.to_string(),
        msg.endowment_id,
    )?;

    // First, burn the vault tokens
    execute_burn(
        deps.branch(),
        env.clone(),
        info,
        msg.endowment_id,
        burn_shares_amount,
    )
    .map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {}",
                burn_shares_amount,
                msg.endowment_id.to_string()
            ),
        })
    })?;

    // Update the config
    let lp_amount = burn_shares_amount.multiply_ratio(config.total_lp_amount, config.total_shares);
    config.total_lp_amount -= lp_amount;
    config.total_shares -= burn_shares_amount;

    CONFIG.save(deps.storage, &config)?;

    // Perform the "loopswap::farming::unstake_and_claim(unfarm)" message
    let mut msgs = vec![];
    let lp_token_contract =
        query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?.liquidity_token;

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::UnstakeAndClaim {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens & farm reward token(LOOP)
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
            action: RemoveLiquidAction::Withdraw {
                beneficiary: msg.beneficiary.clone(),
            },
        })
        .unwrap(),
        funds: vec![],
    }));

    // Handle the reward LOOP tokens
    // IMPORTANT: Atm, the reward token of loopswap farming contract is LOOP token
    // Hence, we assume that the `config.pair_contract` has the token pairs in which
    // there is LOOP token, like LOOP-USDC, LOOP-JUNO.
    let reward_token_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.lp_reward_token.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::Swap {
            beneficiary: Some(msg.beneficiary),
            in_asset_info: terraswap::asset::AssetInfo::Token {
                contract_addr: config.lp_reward_token.to_string(),
            },
            in_asset_bal_before: reward_token_bal_query.balance,
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
///   1. Only callable by `config.keeper` address
///   2. `Claim` the `reward token` from `loopswap::farming` contract
///   3. Compute and send the tax(USDC) to AP treasury
///   4. Re-stake the leftover by converting it to LP token & `loopswap::farming::stake`
pub fn harvest(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Validations
    if info.sender != config.keeper {
        return Err(ContractError::Unauthorized {});
    }

    let msgs = prepare_claim_harvest_msgs(deps.as_ref(), env, &config)?;

    Ok(Response::default()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "harvest")]))
}

/// Contract entry: **add_liquidity**
///   1. Add/Provide the `liquidity` to the `loopswap pair` contract
///   2. Call the `stake` action afterwards
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

    let asset_infos =
        query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?.asset_infos;

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
        config.pair_contract.to_string(),
    )?;

    // Add the "(this contract::)stake" message
    let contract_stake_msgs =
        prepare_contract_stake_msgs(deps.as_ref(), env, endowment_id, config.pair_contract)?;

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
///   1. Stake the `LP` tokens received from `provide_liquidity`
///   2. Mint the `vault token`s
pub fn stake_lp_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: Option<u32>,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Prepare the "loop::farming::stake" msg
    let mut config = CONFIG.load(deps.storage)?;
    let lp_token_contract =
        query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?.liquidity_token;
    let lp_bal_query: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        lp_token_contract.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let lp_stake_amount = lp_bal_query.balance - lp_token_bal_before;
    if lp_stake_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lp_token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
            contract: config.lp_staking_contract.to_string(),
            amount: lp_stake_amount,
            msg: to_binary(&LoopFarmingExecuteMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    // Mint the `vault_token`
    config.total_lp_amount += lp_stake_amount;

    if let Some(endowment_id) = endowment_id {
        let mint_amount: Uint128;
        if config.total_shares.is_zero() {
            mint_amount = Uint128::from(1000000_u128);
        } else {
            mint_amount =
                lp_stake_amount.multiply_ratio(config.total_shares, config.total_lp_amount);
        }
        config.total_shares += mint_amount;

        CONFIG.save(deps.storage, &config)?;

        execute_mint(deps, env, info, endowment_id, mint_amount).map_err(|_| {
            ContractError::Std(StdError::GenericErr {
                msg: format!(
                    "Cannot mint the {} vault token for {}",
                    mint_amount, endowment_id
                ),
            })
        })?;
    }

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![attr("action", "stake_lp_token")]))
}

pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lp_token_bal_before: Uint128,
    action: RemoveLiquidAction,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;
    let pair_info = query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?;
    let lp_token_contract = pair_info.liquidity_token;
    let asset_infos = pair_info.asset_infos;
    let pair_contract = config.pair_contract;

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
    let mut consequence_msgs = vec![];
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
    match action {
        RemoveLiquidAction::Withdraw { beneficiary } => {
            consequence_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::SendAsset {
                    beneficiary: beneficiary.clone(),
                    asset_info: asset_infos[0].clone(),
                    asset_bal_before: asset_0_bal_before,
                })
                .unwrap(),
                funds: vec![],
            }));
            consequence_msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::SendAsset {
                    beneficiary,
                    asset_info: asset_infos[1].clone(),
                    asset_bal_before: asset_1_bal_before,
                })
                .unwrap(),
                funds: vec![],
            }));
        }
        RemoveLiquidAction::Claim {} => {
            todo!("Need to add claim logic")
        }
        RemoveLiquidAction::Harvest => {
            // res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            //     contract_addr: env.contract.address.to_string(),
            //     msg: to_binary(&ExecuteMsg::HarvestSwap {
            //         token1_denom_bal_before: token1_denom_bal,
            //         token2_denom_bal_before: token2_denom_bal,
            //     })
            //     .unwrap(),
            //     funds: vec![],
            // }));
        }
    }

    Ok(Response::default()
        .add_messages(withdraw_liquidity_msgs)
        .add_messages(consequence_msgs)
        .add_attributes(vec![attr("action", "remove_liquidity")]))
}

pub fn send_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Addr,
    asset_info: AssetInfo,
    asset_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // Send the asset to the `beneficiary`
    let asset_bal = query_asset_balance(deps.as_ref(), env.contract.address, asset_info.clone())?;
    let send_amount = asset_bal
        .checked_sub(asset_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;

    let mut msgs = vec![];
    match asset_info {
        AssetInfo::NativeToken { denom } => msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: beneficiary.to_string(),
            amount: coins(send_amount.u128(), denom),
        })),
        AssetInfo::Token { contract_addr } => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                recipient: beneficiary.to_string(),
                amount: send_amount,
            })
            .unwrap(),
            funds: vec![],
        })),
    };

    Ok(Response::default()
        .add_messages(msgs)
        .add_attributes(vec![attr("action", "send_asset")]))
}

/// Prepare the messages for `(this contract::)add_liquidity` operation
fn prepare_contract_add_liquidity_msgs(
    deps: DepsMut,
    env: Env,
    config: &Config,
    endowment_id: Option<u32>,
    deposit_asset_info: AssetInfo,
) -> Result<Vec<CosmosMsg>, StdError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    let pair_info_query: terraswap::asset::PairInfo =
        query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?;

    let (in_asset_info, out_asset_info) = if deposit_asset_info == pair_info_query.asset_infos[0] {
        (
            pair_info_query.asset_infos[0].clone(),
            pair_info_query.asset_infos[1].clone(),
        )
    } else {
        (
            pair_info_query.asset_infos[1].clone(),
            pair_info_query.asset_infos[0].clone(),
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

/// Prepare the `swap` message for the `loopswap pair` contract
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

/// Contract entry: **swap**
///
/// IMPORTANT: Atm, the reward token of loopswap farming contract is LOOP token
/// Hence, we assume that the `config.pair_contract` has the token pairs
/// in which there is LOOP token, like LOOP-USDC, LOOP-JUNO.
pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Option<Addr>,
    in_asset_info: AssetInfo,
    in_asset_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config: Config = CONFIG.load(deps.storage)?;
    let pair_contract = config.pair_contract.to_string();
    let pair_info = query_pair_info_from_pair(&deps.querier, config.pair_contract.clone())?;
    let asset_infos = pair_info.asset_infos;
    let out_asset_info = if in_asset_info == asset_infos[0] {
        asset_infos[1].clone()
    } else {
        asset_infos[0].clone()
    };
    let out_asset_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        out_asset_info.clone(),
    )?;

    // First, compute the "swap_amount"
    let asset_bal = query_asset_balance(
        deps.as_ref(),
        env.contract.address.clone(),
        in_asset_info.clone(),
    )?;
    let swap_amount = asset_bal
        .checked_sub(in_asset_bal_before)
        .map_err(|e| ContractError::Std(StdError::overflow(e)))?;

    // Prepare the "loopswap::pair::swap" msg
    let swap_msgs = prepare_loop_pair_swap_msg(&pair_contract, &in_asset_info, swap_amount)?;

    // Handle the "beneficiary" if any
    let mut send_asset_msg = vec![];
    if let Some(beneficiary) = beneficiary {
        send_asset_msg.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::SendAsset {
                beneficiary,
                asset_info: out_asset_info,
                asset_bal_before: out_asset_bal,
            })
            .unwrap(),
            funds: vec![],
        }));
    };

    Ok(Response::default()
        .add_messages(swap_msgs)
        .add_messages(send_asset_msg)
        .add_attributes(vec![attr("action", "swap")]))
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
    endowment_id: u32,
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
    BALANCES.update(
        deps.storage,
        endowment_id,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(())
}

/// Custom `burn` function for `vault token`
fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    endowment_id: u32,
    amount: Uint128,
) -> Result<(), ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // lower balance
    BALANCES.update(
        deps.storage,
        endowment_id,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    Ok(())
}

/// Query the `asset` balance of `account_addr`
///
/// **NOTE**: `asset_info` is `terraswap::asset::AssetInfo`.
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
