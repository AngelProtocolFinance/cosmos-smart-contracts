use crate::executers;
use crate::queriers;
use crate::state::{Account, Config, Endowment, RebalanceDetails, ACCOUNTS, CONFIG, ENDOWMENT};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::{AcceptedTokens, StrategyComponent};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdResult, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};

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
            admin_addr: deps.api.addr_validate(&msg.admin_addr)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            index_fund_contract: deps.api.addr_validate(&msg.index_fund_contract)?,
            accepted_tokens: AcceptedTokens::default(),
            deposit_approved: false,  // bool
            withdraw_approved: false, // bool
        },
    )?;

    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.registrar_contract,
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

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
            strategies: vec![StrategyComponent {
                vault: deps.api.addr_validate(&registrar_config.default_vault)?,
                locked_percentage: Decimal::one(),
                liquid_percentage: Decimal::one(),
            }],
        },
    )?;

    let account = Account {
        ust_balance: Uint256::zero(),
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

#[entry_point]
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
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::Deposit(msg) => executers::deposit(
            deps,
            env,
            info.clone(),
            info.sender,
            info.funds[0].amount,
            msg,
        ),
        ExecuteMsg::VaultReceipt(msg) => {
            executers::vault_receipt(deps, info.clone(), info.sender, info.funds[0].amount, msg)
        }
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateAdmin { new_admin } => {
            executers::update_admin(deps, env, info, new_admin)
        }
        ExecuteMsg::UpdateStrategy { strategies } => {
            executers::update_strategy(deps, env, info, strategies)
        }
        ExecuteMsg::Liquidate { beneficiary } => executers::liquidate(deps, env, info, beneficiary),
        ExecuteMsg::TerminateToFund { fund } => executers::terminate_to_fund(deps, env, info, fund),
        ExecuteMsg::TerminateToAddress { beneficiary } => {
            executers::terminate_to_address(deps, env, info, beneficiary)
        }
        ExecuteMsg::Receive(msg) => executers::receive(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::Endowment {} => to_binary(&queriers::query_endowment_details(deps)?),
        QueryMsg::Account { account_type } => {
            to_binary(&queriers::query_account_details(deps, account_type)?)
        }
        QueryMsg::AccountList {} => to_binary(&queriers::query_account_list(deps)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
