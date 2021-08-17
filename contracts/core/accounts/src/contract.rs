use crate::executers::executers as AccountExecuters;
use crate::queriers::accounts as AccountQueriers;
use crate::state::{Account, Config, Endowment, RebalanceDetails, ACCOUNTS, CONFIG, ENDOWMENT};
use angel_core::accounts_msg::*;
use angel_core::error::ContractError;
use angel_core::structs::{AcceptedTokens, GenericBalance};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Balance;

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
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
            admin_addr: deps.api.addr_validate(&msg.admin_addr)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            index_fund_contract: deps.api.addr_validate(&msg.index_fund_contract)?,
            accepted_tokens: AcceptedTokens::default(),
            deposit_approved: false,  // bool
            withdraw_approved: false, // bool
        },
    )?;

    ENDOWMENT.save(
        deps.storage,
        &Endowment {
            owner: deps.api.addr_validate(&msg.owner)?, // Addr
            beneficiary: deps.api.addr_validate(&msg.beneficiary)?, // Addr
            name: msg.name.clone(),
            description: msg.description.clone(),
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,                       // Option<u64>
            maturity_height: msg.maturity_height,                   // Option<u64>
            split_to_liquid: msg.split_to_liquid,                   // SplitDetails
        },
    )?;

    let account = Account {
        balance: GenericBalance {
            native: vec![],
            cw20: vec![],
        },
        strategy: vec![],
        rebalance: RebalanceDetails::default(),
    };

    // try to create both prefixed accounts
    for prefix in ["locked", "liquid"].iter() {
        // try to store it, fail if the account ID was already in use
        ACCOUNTS.update(
            deps.storage,
            prefix.to_string(),
            |existing| match existing {
                None => Ok(account.clone()),
                Some(_) => Err(ContractError::AlreadyInUse {}),
            },
        )?;
    }

    Ok(Response::new()
        .add_attribute("name", msg.name)
        .add_attribute("description", msg.description))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // let balance = Balance::from(info.funds.clone());
    if info.funds.len() > 1 {
        return Err(ContractError::InvalidCoinsDeposited {});
    }
    match msg {
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            AccountExecuters::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            AccountExecuters::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::Deposit(msg) => {
            AccountExecuters::deposit(deps, env, info.sender, info.funds[0].amount, msg)
        }
        ExecuteMsg::VaultReceipt(msg) => {
            AccountExecuters::vault_receipt(deps, env, info.sender, info.funds[0].amount)
        }
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            AccountExecuters::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateAdmin { new_admin } => {
            AccountExecuters::update_admin(deps, env, info, new_admin)
        }
        ExecuteMsg::UpdateStrategy {
            account_type,
            strategy,
        } => AccountExecuters::update_strategy(deps, env, info, account_type, strategy),
        ExecuteMsg::Liquidate { beneficiary } => {
            AccountExecuters::liquidate(deps, env, info, beneficiary)
        }
        ExecuteMsg::TerminateToFund { fund } => {
            AccountExecuters::terminate_to_fund(deps, env, info, fund)
        }
        ExecuteMsg::TerminateToAddress { beneficiary } => {
            AccountExecuters::terminate_to_address(deps, env, info, beneficiary)
        }
        ExecuteMsg::Receive(msg) => AccountExecuters::receive(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&AccountQueriers::query_config(deps)?),
        QueryMsg::Endowment {} => to_binary(&AccountQueriers::query_endowment_details(deps)?),
        QueryMsg::Account { account_type } => {
            to_binary(&AccountQueriers::query_account_details(deps, account_type)?)
        }
        QueryMsg::AccountList {} => to_binary(&AccountQueriers::query_account_list(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
