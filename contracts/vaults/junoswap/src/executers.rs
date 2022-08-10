use cosmwasm_std::{
    attr, coins, to_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, Fraction, MessageInfo,
    QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::Denom;
use cw_controllers::ClaimsResponse;

use angel_core::errors::vault::ContractError;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::messages::vault::{
    AccountWithdrawMsg, ExecuteMsg, RemoveLiquidAction, TokenSelect, UpdateConfigMsg,
    WasmSwapExecuteMsg, WasmSwapQueryMsg,
};
use angel_core::responses::registrar::{ConfigResponse, EndowmentListResponse};
use angel_core::responses::vault::{InfoResponse, Token2ForToken1PriceResponse};
use angel_core::structs::EndowmentEntry;
use angel_core::utils::query_denom_balance;

use crate::state::{Config, CONFIG, REMNANTS};

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

    config.pool_addr = match msg.swap_pool_addr {
        Some(ref addr) => deps.api.addr_validate(&addr)?,
        None => config.pool_addr,
    };

    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(&config.pool_addr, &WasmSwapQueryMsg::Info {})?;

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

    config.keeper = match msg.keeper {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.keeper,
    };

    CONFIG.save(deps.storage, &config)?;

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
    let config = CONFIG.load(deps.storage)?;

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

    // Perform the whole "deposit" action
    let mut res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", depositor.to_string())
        .add_attribute("deposit_amount", deposit_amount);

    res = res.add_messages(create_deposit_msgs(
        deps,
        env,
        &config,
        depositor,
        deposit_denom,
        deposit_amount,
    ));

    Ok(res)
}

fn create_deposit_msgs(
    deps: DepsMut,
    env: Env,
    config: &Config,
    depositor: String,
    deposit_denom: Denom,
    deposit_amount: Uint128,
) -> Vec<CosmosMsg> {
    let mut res: Vec<CosmosMsg> = vec![];

    // 1. Add the swap msg
    swap_msg(&config, &deposit_denom, deposit_amount)
        .expect("Cannot create swap msg")
        .into_iter()
        .for_each(|msg| res.push(msg));

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
    res.push(CosmosMsg::Wasm(WasmMsg::Execute {
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

    res
}

/// Claim: Call the `claim` entry of "staking" contract
pub fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    beneficiary: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

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
        &stake_cw20::msg::QueryMsg::Claims {
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
        msg: to_binary(&stake_cw20::msg::ExecuteMsg::Claim {}).unwrap(),
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
            action: RemoveLiquidAction::Claim { beneficiary },
        })
        .unwrap(),
        funds: vec![],
    }));

    Ok(res)
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
    cw20_base::contract::execute_burn(deps.branch(), env.clone(), account_info, msg.amount)
        .map_err(|_| {
            ContractError::Std(StdError::GenericErr {
                msg: format!(
                    "Cannot burn the {} vault tokens from {}",
                    msg.amount,
                    info.sender.to_string()
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
        msg: to_binary(&stake_cw20::msg::ExecuteMsg::Unstake { amount: msg.amount }).unwrap(),
        funds: vec![],
    }));

    // Handle the returning lp tokens if exists
    let staking_contract_config: stake_cw20::msg::GetConfigResponse =
        deps.querier.query_wasm_smart(
            config.staking_addr,
            &stake_cw20::msg::QueryMsg::GetConfig {},
        )?;
    if staking_contract_config.unstaking_duration.is_none() {
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

    // TODO: Add the withdraw information into the `PENDING` map

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

    // Validation
    if info.sender != config.keeper {
        return Err(ContractError::Unauthorized {});
    }

    // First, check if any staking reward does exist
    let claims_resp: ClaimsResponse = deps.querier.query_wasm_smart(
        config.staking_addr.to_string(),
        &stake_cw20::msg::QueryMsg::Claims {
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
        msg: to_binary(&stake_cw20::msg::ExecuteMsg::Claim {}).unwrap(),
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
    _info: MessageInfo,
    token1_denom_bal_before: Uint128,
    token2_denom_bal_before: Uint128,
) -> Result<Response, ContractError> {
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
        let acct_bal = cw20_base::state::BALANCES
            .load(deps.storage, &endowment.address)
            .unwrap_or_default();
        let acct_owed = less_taxes * acct_bal / config.total_shares;
        let liquid_amt = acct_owed * config.harvest_to_liquid.numerator()
            / config.harvest_to_liquid.denominator();
        match config.output_token_denom {
            Denom::Native(ref denom) => {
                res = res.add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: endowment.address.to_string(),
                    amount: coins(liquid_amt.u128(), denom.to_string()),
                }));
            }
            Denom::Cw20(ref token_addr) => {
                res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: token_addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                        recipient: endowment.address.to_string(),
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
        depositor,
        deposit_denom,
        restake_amt,
    ));

    Ok(res)
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
    let config = CONFIG.load(deps.storage)?;

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
        &WasmSwapQueryMsg::Token2ForToken1Price { token2_amount },
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
        msg: to_binary(&WasmSwapExecuteMsg::AddLiquidity {
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
            msg: to_binary(&stake_cw20::msg::ReceiveMsg::Stake {}).unwrap(),
        })
        .unwrap(),
        funds: vec![],
    });

    // Mint the `vault_token`
    config.total_shares += stake_amount;
    CONFIG.save(deps.storage, &config)?;

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
    action: RemoveLiquidAction,
) -> Result<Response, ContractError> {
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
        RemoveLiquidAction::Claim { beneficiary }
        | RemoveLiquidAction::Withdraw { beneficiary } => {
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
    _info: MessageInfo,
    token1_denom_bal_before: Uint128,
    token2_denom_bal_before: Uint128,
    beneficiary: Addr,
) -> Result<Response, ContractError> {
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

//
fn swap_msg(
    config: &Config,
    deposit_denom: &Denom,
    deposit_amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    let input_token = if deposit_denom == &config.input_denoms[0] {
        TokenSelect::Token1
    } else {
        TokenSelect::Token2
    };
    let input_amount = deposit_amount.checked_div(Uint128::from(2_u128))?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    if let Denom::Cw20(contract_addr) = deposit_denom {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&cw20::Cw20ExecuteMsg::IncreaseAllowance {
                spender: config.pool_addr.to_string(),
                amount: input_amount,
                expires: None,
            })
            .unwrap(),
            funds: vec![],
        }));
    }

    let funds = match deposit_denom {
        Denom::Native(denom) => {
            vec![Coin {
                denom: denom.to_string(),
                amount: input_amount,
            }]
        }
        Denom::Cw20(_) => {
            vec![]
        }
    };

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.pool_addr.to_string(),
        msg: to_binary(&WasmSwapExecuteMsg::Swap {
            input_token,
            input_amount,
            min_output: Uint128::zero(), // Here, we set the zero temporarily. Need to be fixed afterwards.
            expiration: None,
        })?,
        funds,
    }));

    Ok(msgs)
}
