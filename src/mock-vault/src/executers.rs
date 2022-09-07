use crate::errors::ContractError;
use crate::msg::{
    ExecuteMsg::VaultReceipt, QueryMsg::Config, RegistrarConfigResponse, UpdateConfigMsg,
};
use crate::state::{BALANCES, CONFIG, TOKEN_INFO};
use cosmwasm_std::{
    coins, to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, WasmMsg,
};
use cw20::Denom;

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
    _msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the SC admin can update these configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::default())
}

pub fn deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg_sender: String,
    endowment_id: u32,
    _deposit_denom: Denom,
    deposit_amount: Uint128,
) -> Result<Response, ContractError> {
    let _config = CONFIG.load(deps.storage)?;
    // Check if the "deposit_amount" is zero or not
    if deposit_amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }

    // First, burn the vault tokens
    execute_mint(
        deps.branch(),
        env.clone(),
        info,
        endowment_id,
        deposit_amount,
    )
    .map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot mint the {} vault tokens from {}",
                deposit_amount,
                endowment_id.to_string()
            ),
        })
    })?;

    Ok(Response::new().add_attribute("action", "deposit"))
}

/// Redeem: Take in an amount of locked/liquid deposit tokens
/// to redeem from the vault for stablecoins to send back to the the Accounts SC
pub fn redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    endowment_id: u32,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // get accounts contract from registrar
    let registrar_config: RegistrarConfigResponse = deps
        .querier
        .query_wasm_smart(config.registrar_contract.to_string(), &Config {})?;

    // First, burn the vault tokens
    execute_burn(deps.branch(), env.clone(), info, endowment_id, amount).map_err(|_| {
        ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Cannot burn the {} vault tokens from {}",
                amount,
                endowment_id.to_string()
            ),
        })
    })?;

    Ok(Response::new()
        .add_attribute("action", "redeem_from_vault")
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registrar_config.accounts_contract.unwrap(),
            msg: to_binary(&VaultReceipt {
                id: endowment_id,
                acct_type: config.acct_type,
            })
            .unwrap(),
            funds: coins(amount.u128(), config.input_denom),
        })))
}

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

fn execute_burn(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
