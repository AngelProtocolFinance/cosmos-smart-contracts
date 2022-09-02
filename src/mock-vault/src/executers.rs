use crate::errors::ContractError;
use crate::msg::{
    AccountWithdrawMsg, EndowmentDetailResponse, EndowmentListResponse, QueryMsg, UpdateConfigMsg,
};
use crate::state::{EndowmentEntry, EndowmentStatus, GenericBalance, BALANCES, CONFIG, TOKEN_INFO};
use cosmwasm_std::{
    to_binary, Addr, Decimal256, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult,
    WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified};

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

    config.moneymarket = match msg.moneymarket {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.moneymarket,
    };

    config.yield_token = match msg.yield_token {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.yield_token,
    };

    config.input_denom = match msg.input_denom {
        Some(denom) => denom,
        None => config.input_denom,
    };
    config.tax_per_block = msg.tax_per_block.unwrap_or(config.tax_per_block);
    config.harvest_to_liquid = msg.harvest_to_liquid.unwrap_or(config.harvest_to_liquid);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn deposit_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _balance: Balance,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only accept max of 1 deposit coin/token per donation
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    // check that the depositor is an Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&QueryMsg::EndowmentList {
                name: None,
                owner: None,
                status: None,
                tier: None,
                un_sdg: None,
                endow_type: None,
            })?,
        }))?;
    let endowments: Vec<EndowmentEntry> = endowments_rsp.endowments;
    let pos = endowments.iter().position(|p| p.address == info.sender);
    // reject if the sender was found in the list of endowments
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

    // increase the total supply of deposit tokens
    let deposit_amount = info.funds[0].amount;
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply += deposit_amount;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or_else(|_| GenericBalance::default());

    // update investment holdings balances
    investment.add_tokens(Balance::Cw20(Cw20CoinVerified {
        amount: deposit_amount,
        address: env.contract.address.clone(),
    }));
    BALANCES.save(deps.storage, &info.sender, &investment)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender)
        .add_attribute("deposit_amount", deposit_amount))
}

/// Redeem Stable: Take in an amount of locked/liquid deposit tokens
/// to redeem from the vault for stablecoins to send back to the the Accounts SC
pub fn redeem_stable(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _account_addr: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the depositor is a Registered Accounts SC
    let endowments_rsp: EndowmentListResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&QueryMsg::EndowmentList {
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
        .position(|p| p.address == info.sender.clone());

    // reject if the sender was found not in the list of endowments
    // OR if the sender is not the Registrar SC (ie. we're closing the endowment)
    if pos == None && info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new())
}

/// Withdraw Stable: Takes in an amount of locked/liquid deposit tokens
/// to withdraw from the vault for UST to send back to a beneficiary
pub fn withdraw_stable(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccountWithdrawMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the tx sender is an Accounts SC
    // Also, it gets some Endowment info
    // If tx sender is an invalid Account or wrong address,
    // this rejects the tx by sending "Unauthroized" error.
    let _endow_detail_resp: EndowmentDetailResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&QueryMsg::Endowment {
                endowment_addr: info.sender.to_string(),
            })?,
        }))
        .map_err(|_| ContractError::Unauthorized {})?;

    // reduce the total supply of CW20 deposit tokens
    let withdraw_total = msg.amount;
    let mut token_info = TOKEN_INFO.load(deps.storage)?;
    token_info.total_supply -= withdraw_total;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    let mut investment = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or_else(|_| GenericBalance::default());

    // check the account has enough balance to cover the withdraw
    let balance = investment.get_token_amount(env.contract.address.clone());
    if balance.amount < msg.amount {
        return Err(ContractError::CannotExceedCap {});
    }

    // update investment holdings balances
    investment.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
        amount: msg.amount,
        address: env.contract.address.clone(),
    }));
    BALANCES.save(deps.storage, &info.sender, &investment)?;

    Ok(Response::new()
        .add_attribute("action", "withdraw_from_vault")
        .add_attribute("sender", info.sender)
        .add_attribute("withdraw_amount", withdraw_total))
}

pub fn harvest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _last_earnings_harvest: u64,
    _last_harvest_fx: Option<Decimal256>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the tx sender is an approved Accounts SC
    let res: StdResult<EndowmentDetailResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&QueryMsg::Endowment {
                endowment_addr: info.sender.to_string(),
            })?,
        }));
    if res.is_err() || res.unwrap().endowment.status != EndowmentStatus::Approved {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_attribute("action", "harvest_redeem_from_vault")
        .add_attribute("sender", info.sender))
}
