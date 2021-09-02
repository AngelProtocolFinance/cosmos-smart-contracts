use crate::state::{CONFIG, ENDOWMENT, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::messages::vault::{AccountTransferMsg, QueryMsg as VaultQuerier};
use angel_core::responses::registrar::{ConfigResponse, VaultDetailResponse, VaultListResponse};
use angel_core::structs::{
    BalanceResponse, FundingSource, GenericBalance, StrategyComponent, YieldVault,
};
use angel_core::utils::{
    deduct_tax, deposit_to_vaults, ratio_adjusted_balance, redeem_from_vaults, withdraw_from_vaults,
};
use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20Coin};

pub fn update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update their address in the configs
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let new_admin = deps.api.addr_validate(&new_admin)?;
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_admin;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the registrar contract can update it's address in the config
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    let new_registrar = deps.api.addr_validate(&new_registrar)?;
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.registrar_contract = new_registrar;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_endowment_settings(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentSettingsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate SC address strings passed
    let beneficiary = deps.api.addr_validate(&msg.beneficiary)?;
    let owner = deps.api.addr_validate(&msg.owner)?;

    ENDOWMENT.update(deps.storage, |mut endowment| -> StdResult<_> {
        endowment.owner = owner;
        endowment.beneficiary = beneficiary;
        endowment.split_to_liquid = msg.split_to_liquid;
        Ok(endowment)
    })?;

    Ok(Response::default())
}

pub fn update_endowment_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentStatusMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the Registrar SC can update these status configs
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.deposit_approved = msg.deposit_approved;
        config.withdraw_approved = msg.withdraw_approved;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_strategies(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    strategies: Vec<Strategy>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENT.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    if config.pending_redemptions != None {
        return Err(ContractError::RedemptionInProgress {});
    }

    let mut addresses: Vec<Addr> = strategies
        .iter()
        .map(|strategy| deps.api.addr_validate(&strategy.vault).unwrap())
        .collect();
    addresses.dedup();

    if addresses.len() < strategies.len() {
        return Err(ContractError::StrategyComponentsNotUnique {});
    };

    let mut locked_percentages_sum = Decimal::zero();
    let mut liquid_percentages_sum = Decimal::zero();

    for strategy_component in strategies.iter() {
        locked_percentages_sum = locked_percentages_sum + strategy_component.locked_percentage;
        liquid_percentages_sum = liquid_percentages_sum + strategy_component.liquid_percentage;
    }

    if locked_percentages_sum != Decimal::one() {
        return Err(ContractError::InvalidStrategyAllocation {});
    }

    if liquid_percentages_sum > Decimal::one() {
        return Err(ContractError::InvalidStrategyAllocation {});
    }

    // redeem all existing strategies from the Endowment's old sources
    // before updating endowment with new sources
    let redeem_messages = redeem_from_vaults(
        deps.as_ref(),
        env.contract.address.to_string(),
        config.registrar_contract.to_string(),
        endowment.strategies,
    )?;

    config.pending_redemptions = Some(redeem_messages.len() as u64);
    CONFIG.save(deps.storage, &config)?;

    // update endowment strategies attribute with all newly passed strategies
    let mut new_strategies = vec![];
    for strategy in strategies {
        new_strategies.push(StrategyComponent {
            vault: deps.api.addr_validate(&strategy.vault.clone())?,
            locked_percentage: strategy.locked_percentage,
            liquid_percentage: strategy.liquid_percentage,
        });
    }
    endowment.strategies = new_strategies;
    ENDOWMENT.save(deps.storage, &endowment)?;

    Ok(
        Response::new().add_submessages(redeem_messages), // .add_submessages(deposit_messages)
    )
}

pub fn vault_receipt(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_addr: Addr,
    msg: AccountTransferMsg,
    balance: Balance,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;

    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    // check that the deposit token came from an approved Vault SC
    let vaults_rsp: VaultListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::ApprovedVaultList {})?,
        }))?;
    let vaults: Vec<YieldVault> = vaults_rsp.vaults;
    let pos = vaults.iter().position(|p| p.address == sender_addr);
    // reject if the sender was not found in the list of vaults
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // funds go into state balances (locked/liquid)
    let total = msg.locked + msg.liquid;
    state
        .balances
        .locked_balance
        .add_tokens(ratio_adjusted_balance(balance.clone(), msg.locked, total));
    state
        .balances
        .liquid_balance
        .add_tokens(ratio_adjusted_balance(balance, msg.liquid, total));
    STATE.save(deps.storage, &state)?;

    let mut deposit_submessages: Vec<SubMsg> = vec![];
    match config.pending_redemptions {
        // last redemption, remove pending u64, and build deposit submsgs
        Some(1) => {
            config.pending_redemptions = None;
            let mut state = STATE.load(deps.storage)?;
            // let ust_locked = state.balances.locked_balance.get_ust();
            // let ust_liquid = state.balances.liquid_balance.get_ust();
            deposit_submessages = redeem_from_vaults(
                deps.as_ref(),
                env.contract.address.to_string(),
                config.registrar_contract.to_string(),
                endowment.strategies,
            )?;
            // set UST balances available to zero
            state
                .balances
                .locked_balance
                .set_token_balances(Balance::from(vec![Coin {
                    amount: Uint128::zero(),
                    denom: "uusd".to_string(),
                }]));
            state
                .balances
                .liquid_balance
                .set_token_balances(Balance::from(vec![Coin {
                    amount: Uint128::zero(),
                    denom: "uusd".to_string(),
                }]));
            STATE.save(deps.storage, &state)?;
        }
        // subtract one redemption and hold off on doing deposits
        Some(_) => config.pending_redemptions = Some(config.pending_redemptions.unwrap() - 1),
        None => (),
    };

    Ok(Response::new()
        .add_submessages(deposit_submessages)
        .add_attribute("action", "vault_receipt")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender_addr: Addr,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the Endowment has been approved to receive deposits
    if !config.deposit_approved {
        return Err(ContractError::Unauthorized {});
    }

    // check that the split %s sum to 1
    if msg.locked_percentage + msg.liquid_percentage != Decimal::one() {
        return Err(ContractError::InvalidSplit {});
    }

    let deposit_amount: Coin = Coin {
        denom: "uusd".to_string(),
        amount: info
            .funds
            .iter()
            .find(|c| c.denom == *"uusd")
            .map(|c| c.amount)
            .unwrap_or(Uint128::zero()),
    };

    if deposit_amount.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    let after_tax: Coin = deduct_tax(deps.as_ref(), deposit_amount.clone()).unwrap();

    let locked_split = msg.locked_percentage;
    let liquid_split = msg.liquid_percentage;

    // MVP LOGIC: Only index fund SC (aka TCA Member donations are accepted)
    // Get the Index Fund SC address from the Registrar SC
    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    // fails if the token deposit was not coming from the Index Fund SC
    if sender_addr != registrar_config.index_fund {
        // let splits = ENDOWMENT.load(deps.storage)?.split_to_liquid;
        // let new_splits = check_splits(splits, locked_split, liquid_split);
        // locked_split = new_splits.0;
        // liquid_split = new_splits.1;
        return Err(ContractError::Unauthorized {});
    }

    let ust_locked = Coin {
        amount: after_tax.amount * locked_split,
        denom: "uusd".to_string(),
    };
    let ust_liquid = Coin {
        amount: after_tax.amount * liquid_split,
        denom: "uusd".to_string(),
    };

    // update total donations recieved for a charity
    let mut state = STATE.load(deps.storage)?;
    let endowment = ENDOWMENT.load(deps.storage)?;
    state.donations_received += deposit_amount.amount.clone();
    STATE.save(deps.storage, &state)?;

    // build deposit messages for each of the sources/amounts
    let deposit_messages = deposit_to_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        ust_locked,
        ust_liquid,
        &endowment.strategies,
    )?;

    Ok(Response::new()
        .add_submessages(deposit_messages)
        .add_attribute("action", "account_deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", deposit_amount.amount.to_string()))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sources: Vec<FundingSource>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let endowment = ENDOWMENT.load(deps.storage)?;
    // check that sender is the owner or the beneficiary
    if info.sender != endowment.owner || info.sender != endowment.beneficiary {
        return Err(ContractError::Unauthorized {});
    }

    // check if locked tokens are requested and
    // reject if endowment cannot withdraw from locked before maturity
    for source in sources.iter() {
        if source.locked > Uint128::zero() && !endowment.withdraw_before_maturity {
            return Err(ContractError::InaccessableLockedBalance {});
        }
    }

    // build redeem messages for each of the sources/amounts
    let withdraw_messages = withdraw_from_vaults(
        deps.as_ref(),
        config.registrar_contract.to_string(),
        &env.contract.address,
        sources,
    )?;

    Ok(Response::new()
        .add_submessages(withdraw_messages)
        .add_attribute("action", "withdrawal")
        .add_attribute("sender", env.contract.address.to_string())
        .add_attribute("beneficiary", endowment.beneficiary.to_string()))
}

pub fn liquidate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // Get the Index Fund SC address from the Registrar SC
    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let _index_fund: String = registrar_config.index_fund;

    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    // TO DO: send all tokens out to the index fund sc
    // let _messages = send_tokens(&config.index_fund, &account.ust_balance)?;

    Ok(Response::new()
        .add_attribute("action", "liquidate")
        .add_attribute("to", beneficiary_addr))
}

pub fn terminate_to_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    let messages = vec![];

    // TO DO: send all tokens out to the index fund sc
    // messages.append(&mut send_tokens(&beneficiary_addr, &account.ust_balance)?);

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "terminate")
        .add_attribute("to", beneficiary_addr))
}

pub fn terminate_to_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fund: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // Get the Index Fund SC address from the Registrar SC
    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let index_fund: String = registrar_config.index_fund;

    let messages = vec![];
    // TO DO: send all tokens out to the index fund sc
    // messages.append(&mut send_tokens(
    //     &index_fund,
    //     &account.ust_balance,
    // )?);

    Ok(Response::new()
        .add_attribute("action", "terminate")
        .add_attribute("fund_id", format!("{}", fund))
        .add_attribute("to", index_fund)
        .add_submessages(messages))
}
