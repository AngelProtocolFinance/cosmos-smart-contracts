use crate::executers;
use crate::queriers;
use crate::state::{Config, Endowment, State, CONFIG, ENDOWMENT, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::guardians_group::InstantiateMsg as GuardiansGroupInstantiateMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::{AcceptedTokens, BalanceInfo, RebalanceDetails, StrategyComponent};
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Reply, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw4::Member;

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
            owner: deps.api.addr_validate(&msg.owner_sc)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            accepted_tokens: AcceptedTokens::default(),
            deposit_approved: false,  // bool
            withdraw_approved: false, // bool
            pending_redemptions: None,
            multisig_code: Some(msg.guardians_multisig_code),
            group_code: Some(msg.guardians_group_code),
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
            whitelisted_beneficiaries: msg.whitelisted_beneficiaries, // Vec<String>
            whitelisted_contributors: msg.whitelisted_contributors, // Vec<String>
            name: msg.name.clone(),
            description: msg.description.clone(),
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,                       // Option<u64>
            maturity_height: msg.maturity_height,                   // Option<u64>
            locked_endowment_configs: msg.locked_endowment_configs, // vec<String>
            strategies: vec![StrategyComponent {
                vault: deps.api.addr_validate(&registrar_config.default_vault)?,
                locked_percentage: Decimal::one(),
                liquid_percentage: Decimal::one(),
            }],
            rebalance: RebalanceDetails::default(),
            guardian_set: vec![],
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            donations_received: Uint128::zero(),
            balances: BalanceInfo::default(),
            closing_endowment: false,
            closing_beneficiary: None,
        },
    )?;

    // if there is NO guardians list passed initially, use the
    // original Endowment owner address as the sole member
    let mut first_guardians: Vec<Member> = vec![Member {
        addr: msg.owner.into(),
        weight: 1,
    }];
    if !msg.guardian_members.is_empty() {
        first_guardians = msg.guardian_members;
    }

    Ok(Response::new().add_submessage(SubMsg {
        id: 1,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.guardians_group_code,
            admin: None,
            label: "new endowment guardians group".to_string(),
            msg: to_binary(&GuardiansGroupInstantiateMsg {
                admin: Some(info.sender.to_string()),
                members: first_guardians,
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }))
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
        ExecuteMsg::Withdraw {
            sources,
            beneficiary,
        } => executers::withdraw(deps, env, info, sources, beneficiary),
        ExecuteMsg::VaultReceipt(msg) => {
            executers::vault_receipt(deps, env, info.clone(), info.sender, msg)
        }
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::UpdateStrategies { strategies } => {
            executers::update_strategies(deps, env, info, strategies)
        }
        ExecuteMsg::CloseEndowment { beneficiary } => {
            executers::close_endowment(deps, env, info, beneficiary)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
    }
}

/// Replies back to the Endowment Account from various multisig contract calls (@ some passed code_id)
/// should be caught and handled to fire subsequent setup calls and ultimately the storing of the multisig
/// as the Accounts endowment owner
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => executers::new_guardians_group_reply(deps, env, msg.result),
        2 => executers::new_guardians_multisig_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
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
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
