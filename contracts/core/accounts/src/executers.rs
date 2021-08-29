use crate::state::{ACCOUNTS, CONFIG, ENDOWMENT, PENDING_REDEMPTIONS};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::messages::vault::{AccountTransferMsg, QueryMsg as VaultQuerier};
use angel_core::responses::registrar::{ConfigResponse, VaultDetailResponse, VaultListResponse};
use angel_core::responses::vault::VaultBalanceResponse;
use angel_core::structs::{FundingSource, StrategyComponent, YieldVault};
use angel_core::utils::{deduct_tax, redeem_from_vaults, vault_account_balance, vault_fx_rate};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QueryRequest, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ReceiveMsg;

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
    let config = CONFIG.load(deps.storage)?;
    let mut endowment = ENDOWMENT.load(deps.storage)?;

    if info.sender != endowment.owner {
        return Err(ContractError::Unauthorized {});
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
    let mut retrieved_funds = VaultBalanceResponse {
        denom: "uusd".to_string(),
        locked: Uint128::zero(),
        liquid: Uint128::zero(),
    };

    let mut old_sources: Vec<FundingSource> = vec![];
    for strategy in endowment.strategies.iter() {
        let fx_rate = Decimal256::percent(91); // vault_fx_rate(deps.as_ref(), strategy.vault.to_string());
        let vault_balances: VaultBalanceResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: strategy.vault.to_string(),
                msg: to_binary(&VaultQuerier::Balance {
                    address: env.contract.address.to_string(),
                })?,
            }))?;
        // calculate the amount of UST to redeem from locked/liquid and add to a growing retrieved total
        let redeem_locked =
            vault_balances.locked * Decimal::from(fx_rate) * strategy.locked_percentage;
        let redeem_liquid =
            vault_balances.liquid * Decimal::from(fx_rate) * strategy.liquid_percentage;
        retrieved_funds.locked += redeem_locked;
        retrieved_funds.liquid += redeem_liquid;
        // add new submessage to vector
        old_sources.push(FundingSource {
            vault: strategy.vault.to_string(),
            locked: vault_balances.locked * Decimal::from(fx_rate) * strategy.locked_percentage,
            liquid: vault_balances.liquid * Decimal::from(fx_rate) * strategy.liquid_percentage,
        });
    }

    let redeem_messages = redeem_from_vaults(
        deps.as_ref(),
        config.next_transfer_id,
        env.contract.address.to_string(),
        config.registrar_contract.to_string(),
        old_sources,
    )?;

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

    // TO DO: DEPOSIT MSGS SHOULD BE DONE AFTER ALL RECV SUBMSG CALLS HAVE COMPLETED
    // COULD BE HANDLED BY A REPLY FUNC ??
    // // build deposit messages for the new strategies to re-distribute available funds according to the new strategy
    // let after_taxes_locked: Coin = deduct_tax(
    //     deps.as_ref(),
    //     Coin {
    //         denom: "uusd".to_string(),
    //         amount: retrieved_funds.locked,
    //     },
    // )
    // .unwrap();
    // let after_taxes_liquid: Coin = deduct_tax(
    //     deps.as_ref(),
    //     Coin {
    //         denom: "uusd".to_string(),
    //         amount: retrieved_funds.liquid,
    //     },
    // )
    // .unwrap();

    // let mut deposit_messages: Vec<SubMsg> = vec![];
    // for strategy in endowment.strategies.iter() {
    //     let transfer_msg = AccountTransferMsg {
    //         locked: Uint256::from(after_taxes_locked.amount * strategy.locked_percentage),
    //         liquid: Uint256::from(after_taxes_liquid.amount * strategy.liquid_percentage),
    //     };

    //     // create a deposit message for X Vault, noting amounts for Locked / Liquid
    //     // funds payload contains both amounts for locked and liquid accounts
    //     deposit_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: strategy.vault.to_string(),
    //         msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Deposit(
    //             transfer_msg,
    //         ))
    //         .unwrap(),
    //         funds: vec![Coin {
    //             amount: after_taxes_locked.amount + after_taxes_liquid.amount,
    //             denom: "uusd".to_string(),
    //         }],
    //     })))
    // }

    Ok(
        Response::new().add_submessages(redeem_messages), // .add_submessages(deposit_messages)
    )
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // check that the sending token contract is an Approved Token
    if !config.accepted_tokens.cw20_valid(info.sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }
    if cw20_msg.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }
    let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
    let msg = from_binary(&cw20_msg.msg)?;
    match msg {
        ReceiveMsg::Deposit(msg) => deposit(deps, env, info, sender_addr, msg),
        ReceiveMsg::VaultReceipt(msg) => vault_receipt(deps, info, sender_addr, msg),
    }
}

pub fn vault_receipt(
    deps: DepsMut,
    info: MessageInfo,
    sender_addr: Addr,
    msg: AccountTransferMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the deposit token came from an approved Vault SC
    let vaults_rsp: VaultListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::ApprovedVaultList {})?,
        }))?;
    let vaults: Vec<YieldVault> = vaults_rsp.vaults;
    let pos = vaults.iter().position(|p| p.address == sender_addr);
    // reject if the sender was found in the list of vaults
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

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

    // update the pending redemptions storage value
    let pending = PENDING_REDEMPTIONS.load(deps.storage, msg.transfer_id.to_string())? - 1;
    if pending == 0 {
        PENDING_REDEMPTIONS.remove(deps.storage, msg.transfer_id.to_string())
    } else {
        PENDING_REDEMPTIONS.save(deps.storage, msg.transfer_id.to_string(), &pending)?;
    }

    Ok(Response::new()
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
    let mut config = CONFIG.load(deps.storage)?;

    // check that the Endowment has been approved to receive deposits
    if !config.deposit_approved {
        return Err(ContractError::Unauthorized {});
    }

    // check that the split %s sum to 1
    if msg.locked_percentage + msg.liquid_percentage != Decimal::one() {
        return Err(ContractError::InvalidSplit {});
    }

    let deposit_amount: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == *"uusd")
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let after_tax: Coin = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: "uusd".to_string(),
            amount: deposit_amount - Uint128::from(1_u128),
        },
    )
    .unwrap();

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

    let endowment = ENDOWMENT.load(deps.storage)?;
    let mut messages: Vec<SubMsg> = vec![];

    // Invest the funds according to the Strategy
    for strategy in endowment.strategies.iter() {
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: strategy.vault.to_string(),
                })?,
            }))?;
        let yield_vault: YieldVault = vault_config.vault;

        let transfer_msg = AccountTransferMsg {
            transfer_id: config.next_transfer_id,
            locked: Uint256::from(after_tax.amount * locked_split * strategy.locked_percentage),
            liquid: Uint256::from(after_tax.amount * liquid_split * strategy.liquid_percentage),
        };

        // create a deposit message for X Vault, noting amounts for Locked / Liquid
        // funds payload contains both amounts for locked and liquid accounts
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: yield_vault.address.to_string(),
            msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Deposit(
                transfer_msg,
            ))
            .unwrap(),
            funds: vec![Coin {
                amount: after_tax.amount,
                denom: "uusd".to_string(),
            }],
        })))
    }
    config.next_transfer_id += Uint256::one();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "account_deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", deposit_amount))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sources: Vec<FundingSource>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

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
    let redeem_messages = redeem_from_vaults(
        deps.as_ref(),
        config.next_transfer_id,
        env.contract.address.to_string(),
        config.registrar_contract.to_string(),
        sources,
    )?;
    PENDING_REDEMPTIONS.save(
        deps.storage,
        config.next_transfer_id.to_string(),
        &(redeem_messages.len() as u64),
    )?;
    config.next_transfer_id += Uint256::one();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_submessages(redeem_messages)
        // TO DO: MOVE FINAL BANK TRANSFER TO A REPLY FUNC AFTER RECV UST BACK FROM VAULT??
        // .add_submessage(SubMsg::new(BankMsg::Send {
        //     to_address: endowment.beneficiary.into(),
        //     amount: vec![Coin {
        //         amount: Uint128::from(redeem.total),
        //         denom: "uusd".to_string(),
        //     }],
        // }))
        .add_attribute("action", "withdrawal")
        .add_attribute("sender", env.contract.address))
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

    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let _account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // TO DO: send all tokens out to the index fund sc
        // let _messages = send_tokens(&config.index_fund, &account.ust_balance)?;
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

    let messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let _account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
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

    // Get the Index Fund SC address from the Registrar SC
    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let index_fund: String = registrar_config.index_fund;

    let messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let _account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // TO DO: send all tokens out to the index fund sc
        // messages.append(&mut send_tokens(
        //     &index_fund,
        //     &account.ust_balance,
        // )?);
    }

    let mut res = Response::new()
        .add_attribute("action", "terminate")
        .add_attribute("fund_id", format!("{}", fund))
        .add_attribute("to", index_fund);
    res.messages = messages;
    Ok(res)
}
