use cosmwasm_std::{
    attr, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, Fraction,
    MessageInfo, Order, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw_controllers::ClaimsResponse;
use terraswap::asset::{Asset, AssetInfo};

use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{
    AccountWithdrawMsg, ExecuteMsg, RemoveLiquidAction, UpdateConfigMsg,
};
use angel_core::responses::registrar::{ConfigResponse, EndowmentListResponse};
use angel_core::structs::EndowmentEntry;
use angel_core::utils::query_denom_balance;
use terraswap::querier::{query_balance, query_pair_info_from_pair, query_token_balance};

use crate::state::{Config, PendingInfo, BALANCES, CONFIG, PENDING, TOKEN_INFO};

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
    config.loop_factory_contract = match msg.loop_factory_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.loop_factory_contract,
    };

    config.loop_farming_contract = match msg.loop_farming_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.loop_farming_contract,
    };

    config.loop_pair_contract = match msg.loop_pair_contract {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.loop_pair_contract,
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
        query_pair_info_from_pair(&deps.querier, config.loop_pair_contract)?;
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
        &config.loop_pair_contract.to_string(),
        &deposit_asset_info,
        input_amount,
    )?;

    // Prepare the messages for "(this contract::)add_liquidity" opeartion
    let contract_add_liquidity_msgs = prepare_contract_add_liquidity_msgs(
        deps,
        env,
        &config,
        endowment_id,
        deposit_asset_info,
        deposit_amount,
    )?;

    Ok(Response::default()
        .add_messages(loop_pair_swap_msgs)
        .add_messages(contract_add_liquidity_msgs)
        .add_attribute("action", "deposit")
        .add_attribute("sender", msg_sender)
        .add_attribute("endow_id", endowment_id.to_string())
        .add_attribute("deposit_amount", deposit_amount))
}

/// Claim: Call the `claim` entry of "staking" contract
pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Validations
    // Check if sender address is the "accounts_contract"
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

    // First, check if there is any possible claim in "staking" contract
    let claims_resp: ClaimsResponse = deps.querier.query_wasm_smart(
        config.staking_addr.to_string(),
        &DaoStakeCw20QueryMsg::Claims {
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
        msg: to_binary(&DaoStakeCw20ExecuteMsg::Claim {}).unwrap(),
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
        msg: to_binary(&ExecuteMsg::DistributeClaim {
            lp_token_bal_before: lp_token_bal.balance,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
}

pub fn distribute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    // First, compute the "claim"ed LP tokens
    // Query the "lp_token" balance
    let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        config.pool_lp_token_addr.to_string(),
        &cw20::Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    let total_claimed_amount = lp_token_bal
        .balance
        .checked_div(lp_token_bal_before)
        .unwrap();

    // Filter the pending infoes available for claim.
    let claimable_infoes = PENDING
        .range(deps.storage, None, None, Order::Ascending)
        .take_while(|res| {
            let (_, info) = res.as_ref().unwrap();
            info.release_at.is_expired(&env.block)
        })
        .map(|res| res.unwrap())
        .collect::<Vec<(u32, PendingInfo)>>();

    let total_expected_amount: Uint128 = claimable_infoes.iter().map(|(_, info)| info.amount).sum();

    let reward_amount = total_claimed_amount
        .checked_div(total_expected_amount)
        .unwrap();

    todo!("Add the logic of sending the withdraw amount to own beneficiaries & computing the portion of reward for every endowment and re-staking them");

    Ok(Response::default())
}

/// Withdraw: Takes in an amount of vault tokens
/// to withdraw from the vault for USDC to send back to a beneficiary
pub fn withdraw(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

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
        msg.amount,
    )
    .map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {}",
                msg.amount,
                msg.endowment_id.to_string()
            ),
        })
    })?;

    // Update the "total_shares" value
    config.total_shares -= msg.amount;
    CONFIG.save(deps.storage, &config)?;

    // Perform the "unstaking"
    let mut res = Response::default();
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.staking_addr.to_string(),
        msg: to_binary(&DaoStakeCw20ExecuteMsg::Unstake { amount: msg.amount }).unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens if exists
    let staking_contract_config: DaoStakeCw20GetConfigResponse = deps
        .querier
        .query_wasm_smart(config.staking_addr, &DaoStakeCw20QueryMsg::GetConfig {})?;
    match staking_contract_config.unstaking_duration {
        None => {
            // Query the "lp_token" balance
            let lp_token_bal: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                config.pool_lp_token_addr.to_string(),
                &cw20::Cw20QueryMsg::Balance {
                    address: env.contract.address.to_string(),
                },
            )?;
            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::RemoveLiquidity {
                    lp_token_bal_before: lp_token_bal.balance,
                    action: RemoveLiquidAction::Withdraw {
                        beneficiary: msg.beneficiary,
                    },
                })
                .unwrap(),
                funds: vec![],
            }));
        }
        Some(duration) => {
            // Save the pending_info in the PENDING map
            let pending_info = PendingInfo {
                typ: "withdraw".to_string(),
                endowment_id: msg.endowment_id, // ID of org. sending Accounts SC
                beneficiary: msg.beneficiary,   // return to the beneficiary
                amount: msg.amount,
                release_at: duration.after(&env.block),
            };
            PENDING.save(deps.storage, config.next_pending_id, &pending_info)?;

            // Update the "next_pending_id" in CONFIG
            CONFIG.update(deps.storage, |mut c| -> StdResult<_> {
                c.next_pending_id += 1;
                Ok(c)
            })?;
        }
    }

    Ok(res)
}

// Here is rough Harvest earnings logic:
// 1. We should harvest the earnings from the vault's LP staking rewards.
//    (usually paid out as JUNO & RAW tokens in the case of JunoSwap).
// 2. All rewards are converted into USDC (ie. reward_usdc)
// 3. Send taxes owed to Treasury Wallet: reward_usdc * registrar_config.tax_rate
//      less_taxes = reward_usdc - taxes_owned
// 4. For each Accounts' BALANCE ratio of the total_supply:
//      Calculate the amount of harvested rewards owed to N account: acct_owned = (account_balance / total_supply) * less_taxes
//      Send some back to the Accounts contract to liquid: acct_owned * config.harvest_to_liquid
// 5. All leftover rewards not taxed or sent to liquid accounts should be converted
//      into the Vault's underlying LP token (ie, reward_lp_tokens) and staked.

// One other important point:
// This harvest endpoint should only callable by a single config.keeper address.
// We should add this to Config and pass it in the Instantiate Message, as well as
// make sure the config.owner can update that keeper value in the config.
pub fn harvest(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Validations
    if info.sender != config.keeper {
        return Err(ContractError::Unauthorized {});
    }

    // First, check if any staking reward does exist
    let claims_resp: ClaimsResponse = deps.querier.query_wasm_smart(
        config.staking_addr.to_string(),
        &DaoStakeCw20QueryMsg::Claims {
            address: env.contract.address.to_string(),
        },
    )?;
    if claims_resp.claims.len() == 0 {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Nothing to claim".to_string(),
        }));
    }

    // If any staking reward, ask it for harvest
    let mut res = Response::default();
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.staking_addr.to_string(),
        msg: to_binary(&DaoStakeCw20ExecuteMsg::Claim {}).unwrap(),
        funds: vec![],
    }));

    // Call the "remove_liquidity" entry with the reward_lp_tokens &
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
            action: RemoveLiquidAction::Harvest,
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
}

pub fn harvest_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token1_denom_bal_before: Uint128,
    token2_denom_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;
    // Compute the token balances
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

    // "Swap"ping into the `config.output_token_denom` token(i.e: USDC)

    // Check if `output_token_denom` is `Token1` or `Token2`
    // Also, determine which token to send directly, which token to `SwapAndSendTo`
    let swap_input_token_denom: Denom;
    let swap_input_token_amt: Uint128;

    let swap_input_token: TokenSelect;

    let output_token_bal_before: Uint128;

    if config.output_token_denom == config.input_denoms[0] {
        swap_input_token_denom = config.input_denoms[1].clone();
        swap_input_token_amt = token2_amt;

        swap_input_token = TokenSelect::Token2;

        output_token_bal_before = token1_denom_bal_before;
    } else {
        swap_input_token_denom = config.input_denoms[0].clone();
        swap_input_token_amt = token1_amt;

        swap_input_token = TokenSelect::Token1;

        output_token_bal_before = token2_denom_bal_before;
    };

    let mut res = Response::default();
    let mut msgs: Vec<CosmosMsg> = vec![];
    match swap_input_token_denom {
        Denom::Native(denom) => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.pool_addr.to_string(),
            msg: to_binary(&WasmSwapExecuteMsg::Swap {
                input_token: swap_input_token,
                input_amount: swap_input_token_amt,
                min_output: Uint128::zero(),
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
                msg: to_binary(&WasmSwapExecuteMsg::Swap {
                    input_token: swap_input_token,
                    input_amount: swap_input_token_amt,
                    min_output: Uint128::zero(),
                    expiration: None,
                })
                .unwrap(),
                funds: vec![],
            }))
        }
    }

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::DistributeHarvest {
            output_token_bal_before,
        })
        .unwrap(),
        funds: vec![],
    }));

    res = res.add_messages(msgs);
    Ok(res)
}

pub fn distribute_harvest(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    output_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::default();

    let config = CONFIG.load(deps.storage)?;
    let registrar_config: ConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.to_string(),
        &RegistrarQueryMsg::Config {},
    )?;

    // First, compute the token amount
    let output_token_bal = query_denom_balance(
        &deps,
        &config.output_token_denom,
        env.contract.address.to_string(),
    );
    let total_reward_amt = output_token_bal - output_token_bal_before;

    // Send taxes owed to Treasury wallet: reward_usdc * registrar_config.tax_rate
    let tax_amt = (total_reward_amt * registrar_config.tax_rate.numerator())
        / registrar_config.tax_rate.denominator();

    let less_taxes = total_reward_amt - tax_amt;
    let mut restake_amt = Uint128::zero();

    // Compute the Accounts' BALANCE ratio of the total_supply
    // Send some back to the Accounts contract to liquid: acct_owned * config.harvest_to_liquid
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
    for endowment in endowments.iter() {
        let acct_bal = BALANCES
            .load(deps.storage, endowment.id)
            .unwrap_or_default();
        let acct_owed = less_taxes * acct_bal / config.total_shares;
        let liquid_amt = acct_owed * config.harvest_to_liquid.numerator()
            / config.harvest_to_liquid.denominator();
        match config.output_token_denom {
            Denom::Native(ref denom) => {
                res = res.add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: "fake_recipient".to_string(), // FIXME!
                    amount: coins(liquid_amt.u128(), denom.to_string()),
                }));
            }
            Denom::Cw20(ref token_addr) => {
                res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: token_addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                        recipient: "fake_recipient".to_string(), // FIXME!
                        amount: liquid_amt,
                    })
                    .unwrap(),
                    funds: vec![],
                }))
            }
        }
        restake_amt -= liquid_amt;
    }

    // All leftover rewards not taxed or liquided would be sent to be staked(convert to lp_token & staked)
    let depositor = env.contract.address.to_string();
    let deposit_denom = config.output_token_denom.clone();
    res = res.add_messages(create_deposit_msgs(
        deps,
        env,
        &config,
        0_u32, // Temporary value. Need to be fixed.
        deposit_denom,
        restake_amt,
    ));

    Ok(res)
}

/// Contract entry: **add_liquidity**
///   1. Add/Provide the `liquidity` to the `loopswap pair` contract
///   2. Call the `stake` action afterwards
pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: u32,
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
        query_pair_info_from_pair(&deps.querier, config.loop_pair_contract)?.asset_infos;

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
        config.loop_pair_contract.to_string(),
    )?;

    // Add the "(this contract::)stake" message
    let contract_stake_msgs =
        prepare_contract_stake_msgs(deps.as_ref(), env, endowment_id, config.loop_pair_contract)?;

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
    loop_pair_contract: String,
) -> StdResult<Vec<CosmosMsg>> {
    let mut funds = vec![];
    let mut msgs = vec![];
    match token1_asset_info {
        AssetInfo::NativeToken { denom } => funds.push(Coin {
            denom,
            amount: token1_amount,
        }),
        AssetInfo::Token { contract_addr } => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: loop_pair_contract.to_string(),
                amount: token1_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    match token2_asset_info {
        AssetInfo::NativeToken { denom } => funds.push(Coin {
            denom,
            amount: token2_amount,
        }),
        AssetInfo::Token { contract_addr } => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: loop_pair_contract,
                amount: token2_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        })),
    }

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: loop_pair_contract.to_string(),
        msg: to_binary(&terraswap::pair::ExecuteMsg::ProvideLiquidity {
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
            slippage_tolerance: None,
            receiver: None,
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
    endowment_id: u32,
    loop_pair_contract: Addr,
) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    let lp_token_contract =
        query_pair_info_from_pair(&deps.querier, loop_pair_contract)?.liquidity_token;
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

pub fn stake_lp_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: u32,
    lp_token_bal_before: Uint128,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CONFIG.load(deps.storage)?;

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
            msg: to_binary(&DaoStakeCw20ReceiveMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    // Mint the `vault_token`
    config.total_shares += stake_amount;
    CONFIG.save(deps.storage, &config)?;

    execute_mint(deps, env, info, endowment_id, stake_amount).map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot mint the {} vault token for {}",
                stake_amount, endowment_id
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
    info: MessageInfo,
    lp_token_bal_before: Uint128,
    action: RemoveLiquidAction,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;

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
        contract_addr: config.pool_lp_token_addr.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
            spender: config.pool_addr.to_string(),
            amount: lp_token_amt,
            expires: None,
        })
        .unwrap(),
        funds: vec![],
    }));
    res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_addr.to_string(),
        msg: to_binary(&WasmSwapExecuteMsg::RemoveLiquidity {
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
    match action {
        RemoveLiquidAction::Claim {} => {
            todo!("Need to add claim logic")
        }
        RemoveLiquidAction::Withdraw { beneficiary } => {
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
        }
        RemoveLiquidAction::Harvest => {
            res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::HarvestSwap {
                    token1_denom_bal_before: token1_denom_bal,
                    token2_denom_bal_before: token2_denom_bal,
                })
                .unwrap(),
                funds: vec![],
            }));
        }
    }

    Ok(res)
}

pub fn swap_and_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token1_denom_bal_before: Uint128,
    token2_denom_bal_before: Uint128,
    beneficiary: Addr,
) -> Result<Response, ContractError> {
    // Validations
    if info.sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;

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
            msg: to_binary(&WasmSwapExecuteMsg::SwapAndSendTo {
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
                msg: to_binary(&WasmSwapExecuteMsg::SwapAndSendTo {
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

/// Prepare the messages for `(this contract::)add_liquidity` operation
fn prepare_contract_add_liquidity_msgs(
    deps: DepsMut,
    env: Env,
    config: &Config,
    endowment_id: u32,
    deposit_asset_info: AssetInfo,
    deposit_amount: Uint128,
) -> Result<Vec<CosmosMsg>, StdError> {
    let mut msgs: Vec<CosmosMsg> = vec![];

    let pair_info_query: terraswap::asset::PairInfo =
        query_pair_info_from_pair(&deps.querier, config.loop_pair_contract)?;

    let (in_asset_info, out_asset_info) = if deposit_asset_info == pair_info_query.asset_infos[0] {
        (
            pair_info_query.asset_infos[0],
            pair_info_query.asset_infos[1],
        )
    } else {
        (
            pair_info_query.asset_infos[1],
            pair_info_query.asset_infos[0],
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
    let funds = match input_asset_info {
        AssetInfo::NativeToken { denom } => {
            vec![Coin {
                denom: denom.to_string(),
                amount: input_amount,
            }]
        }
        _ => {
            vec![]
        }
    };

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: pair_contract.to_string(),
        msg: to_binary(&terraswap::pair::ExecuteMsg::Swap {
            offer_asset: Asset {
                info: *input_asset_info,
                amount: input_amount,
            },
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
        funds,
    }));

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
