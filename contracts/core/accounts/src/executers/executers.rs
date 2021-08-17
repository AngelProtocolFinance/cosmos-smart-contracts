use crate::state::{ACCOUNTS, CONFIG, ENDOWMENT};
use angel_core::accounts_msg::*;
use angel_core::error::ContractError;
use angel_core::structs::{GenericBalance, SplitDetails, StrategyComponent};
use cosmwasm_std::{
    coin, from_binary, to_binary, Addr, BankMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Balance, Cw20ExecuteMsg, Cw20ReceiveMsg};

pub fn update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update their address in the configs
    if info.sender != config.admin_addr {
        return Err(ContractError::Unauthorized {});
    }
    let new_admin = deps.api.addr_validate(&new_admin)?;
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.admin_addr = new_admin;
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
    if info.sender != config.admin_addr {
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

pub fn update_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    account_type: String,
    strategy: Vec<StrategyComponent>,
) -> Result<Response, ContractError> {
    let endowment = ENDOWMENT.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut addresses: Vec<Addr> = strategy.iter().map(|a| a.portal.clone()).collect();
    addresses.sort();
    addresses.dedup();

    if addresses.len() < strategy.len() {
        return Err(ContractError::StrategyComponentsNotUnique {});
    };

    let mut invested_percentages_sum = Decimal::zero();
    for strategy_component in strategy.iter() {
        invested_percentages_sum = invested_percentages_sum + strategy_component.percentage;
    }

    if invested_percentages_sum != Decimal::percent(100) {
        return Err(ContractError::InvalidStrategyAllocation {});
    }

    // this fails if no account is there
    let mut account = ACCOUNTS.load(deps.storage, account_type.clone())?;

    // update account strategy attribute with the newly passed strategy
    account.strategy = strategy;

    // and save
    ACCOUNTS.save(deps.storage, account_type, &account)?;

    Ok(Response::default())
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // check that the sending token contract is an Approved Token
    if config.accepted_tokens.cw20_valid(info.sender.to_string()) != true {
        return Err(ContractError::Unauthorized {});
    }
    if cw20_msg.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }
    let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
    let msg = from_binary(&cw20_msg.msg)?;
    match msg {
        ReceiveMsg::Deposit(msg) => deposit(deps, env, sender_addr, cw20_msg.amount, msg),
        _ => vault_receipt(deps, env, sender_addr, cw20_msg.amount),
    }
}

pub fn vault_receipt(
    deps: DepsMut,
    _env: Env,
    _sender_addr: Addr,
    balance: Uint128,
) -> Result<Response, ContractError> {
    // this fails if no account is there
    let mut account = ACCOUNTS.load(deps.storage, "locked".to_string())?;

    // this lookup fails if the token deposit was not coming from an Asset Vault SC
    // let portals = VAULTS.load(deps.storage, sender_addr.to_string())?;

    if balance.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    account
        .balance
        .add_tokens(Balance::from(vec![coin(u128::from(balance), "uusd")]));

    // and save
    ACCOUNTS.save(deps.storage, "locked".to_string(), &account)?;

    let res = Response::new()
        .add_attribute("action", "vault_receipt")
        .add_attribute("account_type", "locked");
    Ok(res)
}

pub fn deposit(
    deps: DepsMut,
    _env: Env,
    sender_addr: Addr,
    balance: Uint128,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the Endowment has been approved to receive deposits
    if config.deposit_approved == false {
        return Err(ContractError::Unauthorized {});
    }

    // check that the split %s sum to 1
    if msg.locked_percentage + msg.liquid_percentage != Decimal::one() {
        return Err(ContractError::InvalidSplit {});
    }

    let locked_percentage = msg.locked_percentage;
    let liquid_percentage = msg.liquid_percentage;

    // MVP LOGIC: Only index fund SC (aka TCA Member donations are accepted)
    // fails if the token deposit was not coming from the Index Fund SC
    if sender_addr != config.index_fund_contract {
        // let splits = ENDOWMENT.load(deps.storage)?.split_to_liquid;
        // let new_splits = check_splits(splits, locked_percentage, liquid_percentage);
        // locked_percentage = new_splits.0;
        // liquid_percentage = new_splits.1;
        return Err(ContractError::Unauthorized {});
    }

    // update locked account balance
    let mut locked_account = ACCOUNTS.load(deps.storage, "locked".to_string())?;
    locked_account.balance.add_tokens(Balance::from(vec![coin(
        u128::from(balance * locked_percentage),
        "uusd",
    )]));
    ACCOUNTS.save(deps.storage, "locked".to_string(), &locked_account)?;

    // update liquid account balance
    let mut liquid_account = ACCOUNTS.load(deps.storage, "liquid".to_string())?;
    liquid_account.balance.add_tokens(Balance::from(vec![coin(
        u128::from(balance * liquid_percentage),
        "uusd",
    )]));
    ACCOUNTS.save(deps.storage, "liquid".to_string(), &liquid_account)?;

    Ok(Response::default())
}

pub fn check_splits(
    endowment_splits: SplitDetails,
    user_locked: Decimal,
    user_liquid: Decimal,
) -> (Decimal, Decimal) {
    // check that the split provided by a non-TCA address meets the default
    // split requirements set by the Endowment Account
    if user_liquid > endowment_splits.max || user_liquid < endowment_splits.min {
        return (
            Decimal::one() - endowment_splits.default,
            endowment_splits.default,
        );
    } else {
        return (user_locked, user_liquid);
    }
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
    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // send all tokens out to the index fund sc
        let _messages = send_tokens(&config.index_fund_contract, &account.balance)?;
    }

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

    let mut messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // send all tokens out to the index fund sc
        messages.append(&mut send_tokens(&beneficiary_addr, &account.balance)?);
    }

    let mut res = Response::new()
        .add_attribute("action", "terminate")
        .add_attribute("to", beneficiary_addr);
    res.messages = messages;

    Ok(res)
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
    let mut messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // send all tokens out to the index fund sc
        messages.append(&mut send_tokens(
            &config.index_fund_contract,
            &account.balance,
        )?);
    }

    let mut res = Response::new()
        .add_attribute("action", "terminate")
        .add_attribute("fund_id", format!("{}", fund))
        .add_attribute("to", config.index_fund_contract);
    res.messages = messages;
    Ok(res)
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<SubMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}
