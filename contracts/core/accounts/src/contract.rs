use crate::executers;
use crate::queriers;
use crate::state::{Config, Endowment, State, CONFIG, ENDOWMENT, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::{AcceptedTokens, BalanceInfo, RebalanceDetails, StrategyComponent};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdResult, Uint128, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Balance;

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
            accepted_tokens: AcceptedTokens::default(),
            deposit_approved: false,  // bool
            withdraw_approved: false, // bool
            pending_redemptions: None,
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
            rebalance: RebalanceDetails::default(),
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            donations_received: Uint128::zero(),
            balances: BalanceInfo::default(),
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
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::Deposit(msg) => executers::deposit(deps, env, info.clone(), info.sender, msg),
        ExecuteMsg::Withdraw { sources } => executers::withdraw(deps, env, info, sources),
        ExecuteMsg::VaultReceipt(msg) => executers::vault_receipt(
            deps,
            env,
            info.clone(),
            info.sender,
            msg,
            Balance::from(info.funds),
        ),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateAdmin { new_admin } => {
            executers::update_admin(deps, env, info, new_admin)
        }
        ExecuteMsg::UpdateStrategies { strategies } => {
            executers::update_strategies(deps, env, info, strategies)
        }
        ExecuteMsg::Liquidate { beneficiary } => executers::liquidate(deps, env, info, beneficiary),
        ExecuteMsg::TerminateToFund { fund } => executers::terminate_to_fund(deps, env, info, fund),
        ExecuteMsg::TerminateToAddress { beneficiary } => {
            executers::terminate_to_address(deps, env, info, beneficiary)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance {} => to_binary(&queriers::query_account_balance(deps, env)?),
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queriers::query_state(deps)?),
        QueryMsg::Endowment {} => to_binary(&queriers::query_endowment_details(deps)?),
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
