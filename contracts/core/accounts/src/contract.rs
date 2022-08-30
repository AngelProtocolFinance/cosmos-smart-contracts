use crate::executers;
use crate::queriers;
use crate::state::{Config, CONFIG};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfoBase};

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner_sc)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            next_account_id: 1 as u32,
        },
    )?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Deposit(msg) => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::deposit(deps, env, info.clone(), info.sender, msg, native_fund)
        }
        ExecuteMsg::SwapToken {
            id,
            acct_type,
            amount,
            operations,
        } => executers::swap_token(deps, info, id, acct_type, amount, operations),
        ExecuteMsg::SwapReceipt {
            id,
            final_asset,
            acct_type,
        } => executers::swap_receipt(deps, id, info.sender, final_asset, acct_type),
        ExecuteMsg::VaultReceipt { id, acct_type } => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::vault_receipt(deps, id, acct_type, info.sender, native_fund)
        }
        ExecuteMsg::CreateEndowment(msg) => executers::create_endowment(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::ReinvestToLocked {
            id,
            amount,
            vault_addr,
        } => executers::reinvest_to_locked(deps, env, info, id, amount, vault_addr),
        ExecuteMsg::Withdraw {
            id,
            acct_type,
            beneficiary,
            assets,
        } => executers::withdraw(deps, info, id, acct_type, beneficiary, assets),
        ExecuteMsg::VaultsInvest {
            id,
            acct_type,
            vaults,
        } => executers::vaults_invest(deps, info, id, acct_type, vaults),
        ExecuteMsg::VaultsRedeem {
            id,
            acct_type,
            vaults,
        } => executers::vaults_redeem(deps, env, info, id, acct_type, vaults),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::UpdateStrategies {
            id,
            acct_type,
            strategies,
        } => executers::update_strategies(deps, env, info, id, acct_type, strategies),
        ExecuteMsg::CopycatStrategies {
            id,
            acct_type,
            id_to_copy,
        } => executers::copycat_strategies(deps, info, id, acct_type, id_to_copy),
        ExecuteMsg::CloseEndowment { id, beneficiary } => {
            executers::close_endowment(deps, env, info, id, beneficiary)
        }
        ExecuteMsg::UpdateProfile(msg) => executers::update_profile(deps, env, info, msg),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    let cw20_fund = Asset {
        info: AssetInfoBase::Cw20(deps.api.addr_validate(info.sender.as_str())?),
        amount: cw20_msg.amount,
    };
    match from_binary(&cw20_msg.msg) {
        Ok(ReceiveMsg::VaultReceipt { id, acct_type }) => executers::vault_receipt(
            deps,
            id,
            acct_type,
            api.addr_validate(&cw20_msg.sender)?,
            cw20_fund,
        ),
        Ok(ReceiveMsg::Deposit(msg)) => executers::deposit(
            deps,
            env,
            info.clone(),
            api.addr_validate(&cw20_msg.sender)?,
            msg,
            cw20_fund,
        ),
        _ => Err(ContractError::InvalidInputs {}),
    }
}

/// Replies back to the Endowment Account from various multisig contract calls (@ some passed code_id)
/// should be caught and handled to fire subsequent setup calls and ultimately the storing of the multisig
/// as the Accounts endowment owner
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => executers::cw3_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::Balance { id } => to_binary(&queriers::query_account_balance(deps, id)?),
        QueryMsg::State { id } => to_binary(&queriers::query_state(deps, id)?),
        QueryMsg::Endowment { id } => to_binary(&queriers::query_endowment_details(deps, id)?),
        QueryMsg::GetProfile { id } => to_binary(&queriers::query_profile(deps, id)?),
        QueryMsg::TokenAmount {
            id,
            asset_info,
            acct_type,
        } => to_binary(&queriers::query_token_amount(
            deps, id, asset_info, acct_type,
        )?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        }));
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        }));
    }

    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
