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
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            ibc_controller: info.sender, // set as orig sender initially
            owner: deps.api.addr_validate(&msg.owner_sc)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            next_account_id: 1_u32,
            max_general_category_id: 1_u8,
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
                info: AssetInfo::Native(info.funds[0].denom.to_string()),
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
            executers::vault_receipt(deps, env, id, acct_type, info.sender, native_fund)
        }
        ExecuteMsg::CreateEndowment(msg) => executers::create_endowment(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentDetails(msg) => {
            executers::update_endowment_details(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::DistributeToBeneficiary { id } => {
            executers::distribute_to_beneficiary(deps, env, info, id)
        }
        ExecuteMsg::ReinvestToLocked {
            id,
            amount,
            vault_addr,
        } => executers::reinvest_to_locked(deps, env, info, id, amount, vault_addr),
        ExecuteMsg::Withdraw {
            id,
            acct_type,
            beneficiary_wallet,
            beneficiary_endow,
            assets,
        } => executers::withdraw(
            deps,
            env,
            info,
            id,
            acct_type,
            beneficiary_wallet,
            beneficiary_endow,
            assets,
        ),
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
        ExecuteMsg::UpdateConfig {
            new_owner,
            new_registrar,
            max_general_category_id,
            ibc_controller,
        } => executers::update_config(
            deps,
            env,
            info,
            new_owner,
            new_registrar,
            max_general_category_id,
            ibc_controller,
        ),
        ExecuteMsg::UpdateStrategies {
            id,
            acct_type,
            strategies,
        } => executers::update_strategies(deps, env, info, id, acct_type, strategies),
        ExecuteMsg::CloseEndowment { id, beneficiary } => {
            executers::close_endowment(deps, env, info, id, beneficiary)
        }
        // Manage the allowances for the 3rd_party wallet to withdraw
        // the endowment TOH liquid balances without the proposal
        ExecuteMsg::Allowance {
            endowment_id,
            action,
            spender,
            asset,
            expires,
        } => executers::manage_allowances(
            deps,
            env,
            info,
            endowment_id,
            action,
            spender,
            asset,
            expires,
        ),
        ExecuteMsg::SpendAllowance {
            endowment_id,
            asset,
        } => executers::spend_allowance(deps, env, info, endowment_id, asset),
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
            env,
            id,
            acct_type,
            api.addr_validate(&cw20_msg.sender)?,
            cw20_fund,
        ),
        Ok(ReceiveMsg::Deposit(msg)) => executers::deposit(
            deps,
            env,
            info,
            api.addr_validate(&cw20_msg.sender)?,
            msg,
            cw20_fund,
        ),
        Ok(ReceiveMsg::SwapReceipt {
            id,
            final_asset,
            acct_type,
        }) => {
            let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            executers::swap_receipt(deps, id, sender_addr, final_asset, acct_type)
        }
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
        // 1 => executers::dao_reply(deps, env, msg.result),
        // 2 => executers::donation_match_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::Balance { id } => to_binary(&queriers::query_endowment_balance(deps, id)?),
        QueryMsg::State { id } => to_binary(&queriers::query_state(deps, id)?),
        QueryMsg::EndowmentByProposalLink { proposal_link } => to_binary(
            &queriers::query_endowment_by_proposal_link(deps, proposal_link)?,
        ),
        QueryMsg::Endowment { id } => to_binary(&queriers::query_endowment_details(deps, id)?),
        QueryMsg::Allowances { id, spender } => {
            to_binary(&queriers::query_allowances(deps, id, spender)?)
        }
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
