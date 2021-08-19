use crate::state::{InvestmentHolding, ACCOUNTS, CONFIG, ENDOWMENT, INVESTMENTS};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::registrar::QueryMsg as VaultQuerier;
use angel_core::messages::vault::AccountTransferMsg;
use angel_core::responses::registrar::{VaultDetailResponse, VaultListResponse};
use angel_core::structs::{GenericBalance, SplitDetails, StrategyComponent, YieldVault};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

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
    strategies: Vec<StrategyComponent>,
) -> Result<Response, ContractError> {
    let mut endowment = ENDOWMENT.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut addresses: Vec<Addr> = strategies.iter().map(|a| a.vault.clone()).collect();
    addresses.sort();
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

    // update endowment strategies attribute with the newly passed strategies list
    endowment.strategies = strategies;
    ENDOWMENT.save(deps.storage, &endowment)?;

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
        ReceiveMsg::Deposit(msg) => deposit(deps, env, info, sender_addr, cw20_msg.amount, msg),
        ReceiveMsg::VaultReceipt(msg) => {
            vault_receipt(deps, info, sender_addr, cw20_msg.amount, msg)
        }
    }
}

pub fn vault_receipt(
    deps: DepsMut,
    info: MessageInfo,
    sender_addr: Addr,
    balance: Uint128,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the deposit token came from an approved Vault SC
    let vaults_rsp: VaultListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&VaultQuerier::ApprovedVaultList {})?,
        }))?;
    let vaults: Vec<YieldVault> = vaults_rsp.vaults;
    let pos = vaults
        .iter()
        .position(|p| p.address.to_string() == sender_addr.to_string());
    // reject if the sender was found in the list of vaults
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    if balance.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    if info.funds.len() > 1 {
        return Err(ContractError::TokenTypes {});
    }

    if info.funds[0].denom == "uusd" {
        // funds go into general Account balance
        if msg.locked > Uint256::zero() {
            let mut account = ACCOUNTS.load(deps.storage, "locked".to_string())?;
            account.ust_balance += msg.locked;
            ACCOUNTS.save(deps.storage, "locked".to_string(), &account)?;
        }
        if msg.liquid > Uint256::zero() {
            let mut account = ACCOUNTS.load(deps.storage, "liquid".to_string())?;
            account.ust_balance += msg.liquid;
            ACCOUNTS.save(deps.storage, "liquid".to_string(), &account)?;
        }
    } else {
        // funds go into invested coin balances (per vault)
        let mut investment = INVESTMENTS
            .load(deps.storage, sender_addr.to_string())
            .unwrap_or(InvestmentHolding {
                denom: info.funds[0].clone().denom,
                locked: Uint256::zero(),
                liquid: Uint256::zero(),
            });
        investment.locked += msg.locked;
        investment.liquid += msg.liquid;
        INVESTMENTS.save(deps.storage, sender_addr.to_string(), &investment)?;
    }

    Ok(Response::new()
        .add_attribute("action", "vault_receipt")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount_received", info.funds[0].amount))
}

pub fn deposit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
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

    let locked_split = msg.locked_percentage;
    let liquid_split = msg.liquid_percentage;

    // MVP LOGIC: Only index fund SC (aka TCA Member donations are accepted)
    // fails if the token deposit was not coming from the Index Fund SC
    if sender_addr != config.index_fund_contract {
        // let splits = ENDOWMENT.load(deps.storage)?.split_to_liquid;
        // let new_splits = check_splits(splits, locked_percentage, liquid_percentage);
        // locked_split = new_splits.0;
        // liquid_split = new_splits.1;
        return Err(ContractError::Unauthorized {});
    }

    let endowment = ENDOWMENT.load(deps.storage)?;
    let mut messages: Vec<SubMsg> = vec![];

    // Invest the funds according to the Strategy
    for strategy in endowment.strategies.iter() {
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&VaultQuerier::Vault {
                    vault_addr: strategy.vault.to_string(),
                })?,
            }))?;
        let yield_vault: YieldVault = vault_config.vault;

        let transfer_msg = AccountTransferMsg {
            locked: Uint256::from(balance * locked_split * strategy.locked_percentage),
            liquid: Uint256::from(balance * liquid_split * strategy.liquid_percentage),
        };

        // create a deposit message for X Vault, noting amounts for Locked / Liquid
        // funds payload contains both amounts for locked and liquid accounts
        messages.push(SubMsg {
            id: 42,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: yield_vault.address.to_string(),
                msg: to_binary(&transfer_msg).unwrap(),
                funds: vec![Coin {
                    amount: balance,
                    denom: "uusd".to_string(),
                }],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        })
    }

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "account_deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", balance))
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
        // TO DO: send all tokens out to the index fund sc
        // let _messages = send_tokens(&config.index_fund_contract, &account.ust_balance)?;
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
        // TO DO: send all tokens out to the index fund sc
        // messages.append(&mut send_tokens(&beneficiary_addr, &account.ust_balance)?);
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
        // TO DO: send all tokens out to the index fund sc
        // messages.append(&mut send_tokens(
        //     &config.index_fund_contract,
        //     &account.ust_balance,
        // )?);
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

pub fn redeem(
    deps: DepsMut,
    _env: Env,
    sender: String,
    _amount: Uint128,
    _denom: String,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;

    let endowment = ENDOWMENT.load(deps.storage)?;
    // check that sender is the owner
    if sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
    }

    //     let exchange_rate = adapter::exchange_rate(deps, &config.yield_adapter, &config.input_denom)?;
    //     let return_amount = deduct_tax(
    //         deps,
    //         Coin {
    //             denom: config.input_denom.clone(),
    //             amount: amount.into(),
    //         },
    //     )?;

    //     Ok(Response::new()
    //         messages: [
    //             vec![CosmosMsg::Wasm(WasmMsg::Execute {
    //                 contract_addr: deps.api.human_address(&config.dp_token)?,
    //                 msg: to_binary(&Cw20HandleMsg::Burn { amount })?,
    //                 send: vec![],
    //             })],
    //             adapter::redeem(
    //                 deps,
    //                 &config.yield_adapter,
    //                 Uint256::from(amount).div(exchange_rate).into(),
    //             )?,
    //             vec![CosmosMsg::Bank(BankMsg::Send {
    //                 from_address: env.contract.address,
    //                 to_address: sender,
    //                 amount: vec![return_amount.clone()],
    //             })],
    //         ]
    //         .concat(),
    //         log: vec![
    //             log("action", "redeem"),
    //             log("sender", env.message.sender),
    //             log("burn_amount", amount),
    //             log("redeem_amount", return_amount.amount),
    //         ],
    //         data: None,
    //     )
    Ok(Response::default())
}
