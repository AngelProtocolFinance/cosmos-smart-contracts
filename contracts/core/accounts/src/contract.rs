use crate::executers;
use crate::queriers;
use crate::state::PROFILE;
use crate::state::{Config, Endowment, OldState, State, CONFIG, ENDOWMENT, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw4_group::InstantiateMsg as Cw4GroupInstantiateMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::{
    AcceptedTokens, BalanceInfo, Profile, RebalanceDetails, StrategyComponent,
};
use cosmwasm_std::ReplyOn;
use cosmwasm_std::StdError;
use cosmwasm_std::{
    attr, entry_point, from_slice, to_binary, to_vec, Binary, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, QueryRequest, Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
    WasmQuery,
};
use cw2::set_contract_version;

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
        },
    )?;

    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.registrar_contract.clone(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    let default_vault = match registrar_config.default_vault {
        Some(addr) => addr,
        None => return Err(ContractError::ContractNotConfigured {}),
    };
    ENDOWMENT.save(
        deps.storage,
        &Endowment {
            owner: deps.api.addr_validate(&msg.owner)?, // Addr
            beneficiary: deps.api.addr_validate(&msg.beneficiary)?, // Addr
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,           // Option<u64>
            maturity_height: msg.maturity_height,       // Option<u64>
            strategies: vec![StrategyComponent {
                vault: deps.api.addr_validate(&default_vault)?,
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
            transactions: vec![],
        },
    )?;

    PROFILE.save(deps.storage, &msg.profile)?;

    // initial default Response to add submessages to
    let mut res = Response::new().add_attributes(vec![
        attr("endow_name", msg.profile.name),
        attr("endow_owner", msg.owner),
        attr("endow_type", msg.profile.endow_type.to_string()),
        attr(
            "endow_logo",
            msg.profile.logo.unwrap_or_else(|| "".to_string()),
        ),
        attr(
            "endow_image",
            msg.profile.image.unwrap_or_else(|| "".to_string()),
        ),
    ]);

    // check if CW3/CW4 codes were passed to setup a multisig/group
    if msg.cw4_members.ne(&vec![])
        && (registrar_config.cw3_code.ne(&None) && registrar_config.cw4_code.ne(&None))
    {
        res = res.add_submessage(SubMsg {
            id: 1,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: registrar_config.cw4_code.unwrap(),
                admin: None,
                label: "new endowment cw4 group".to_string(),
                msg: to_binary(&Cw4GroupInstantiateMsg {
                    admin: Some(info.sender.to_string()),
                    members: msg.cw4_members,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        })
    }

    Ok(res)
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
        ExecuteMsg::UpdateGuardians { add, remove } => {
            executers::update_guardians(deps, env, info, add, remove)
        }
        ExecuteMsg::UpdateProfile(msg) => executers::update_profile(deps, env, info, msg),
    }
}

/// Replies back to the Endowment Account from various multisig contract calls (@ some passed code_id)
/// should be caught and handled to fire subsequent setup calls and ultimately the storing of the multisig
/// as the Accounts endowment owner
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => executers::new_cw4_group_reply(deps, env, msg.result),
        2 => executers::new_cw3_multisig_reply(deps, env, msg.result),
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
        QueryMsg::GetProfile {} => to_binary(&queriers::query_profile(deps)?),
        QueryMsg::GetTxRecords {
            sender,
            recipient,
            denom,
        } => to_binary(&queriers::query_transactions(
            deps, sender, recipient, denom,
        )?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    // Documentation on performing updates during migration
    // https://docs.cosmwasm.com/docs/1.0/smart-contracts/migration/#using-migrate-to-update-otherwise-immutable-state
    const STATE_KEY: &[u8] = b"state";
    let data = deps.storage.get(STATE_KEY).ok_or_else(|| {
        ContractError::Std(StdError::NotFound {
            kind: "State".to_string(),
        })
    })?;
    let state: OldState = from_slice(&data)?;

    deps.storage.set(
        STATE_KEY,
        &to_vec(&State {
            donations_received: state.donations_received,
            balances: state.balances,
            closing_endowment: state.closing_endowment,
            closing_beneficiary: state.closing_beneficiary,
            transactions: vec![],
        })?,
    );

    const PROFILE_KEY: &[u8] = b"profile";

    let mut profile = Profile::default();
    profile.name = msg.name;
    profile.overview = msg.overview;
    profile.logo = msg.logo;
    profile.image = msg.image;

    deps.storage.set(PROFILE_KEY, &to_vec(&profile)?);
    Ok(Response::default())
}
