use crate::executers;
use crate::queriers;
use crate::state::{Config, CONFIG};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::structs::{EndowmentEntry, EndowmentStatus, EndowmentType, SplitDetails, Tier};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    entry_point, to_binary, to_vec, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Path;
use std::ops::Deref;

// version info for future migration info
const CONTRACT_NAME: &str = "registrar";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let tax_rate = percentage_checks(msg.tax_rate).unwrap();
    let splits: SplitDetails = match msg.split_to_liquid {
        Some(splits) => split_checks(splits.max, splits.min, splits.default),
        None => Ok(SplitDetails::default()),
    }
    .unwrap();

    let configs = Config {
        owner: info.sender.clone(),
        guardian_angels: None,
        index_fund_contract: None,
        accounts_code_id: msg.accounts_code_id.unwrap_or(0u64),
        treasury: deps.api.addr_validate(&msg.treasury)?,
        tax_rate,
        default_vault: msg.default_vault,
        guardians_multisig_addr: None,
        endowment_owners_group_addr: None,
        split_to_liquid: splits,
        halo_token: None,
        gov_contract: None,
        charity_shares_contract: None,
    };

    CONFIG.save(deps.storage, &configs)?;

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
        ExecuteMsg::CreateEndowment(msg) => executers::create_endowment(deps, env, info, msg),
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::VaultAdd(msg) => executers::vault_add(deps, env, info, msg),
        ExecuteMsg::VaultRemove { vault_addr } => {
            executers::vault_remove(deps, env, info, vault_addr)
        }
        ExecuteMsg::VaultUpdateStatus {
            vault_addr,
            approved,
        } => executers::vault_update_status(deps, env, info, vault_addr, approved),
        ExecuteMsg::Harvest {
            collector_address,
            collector_share,
        } => executers::harvest(deps, env, info, collector_address, collector_share),
        ExecuteMsg::UpdateEndowmentType(msg) => {
            executers::update_endowment_type(deps, env, info, msg)
        }
    }
}

/// Replies back to the registrar from instantiate calls to Accounts SC (@ some code_id)
/// should be caught and handled to register the Endowment's newly created Accounts SC
/// in the REGISTRY storage
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => executers::new_accounts_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::Endowment { endowment_addr } => {
            to_binary(&queriers::query_endowment_details(deps, endowment_addr)?)
        }
        QueryMsg::EndowmentList {
            name,
            owner,
            status,
            tier,
            un_sdg,
            endow_type,
        } => to_binary(&queriers::query_endowment_list(
            deps, name, owner, status, tier, un_sdg, endow_type,
        )?),
        QueryMsg::ApprovedVaultList { start_after, limit } => to_binary(
            &queriers::query_approved_vault_list(deps, start_after, limit)?,
        ),
        QueryMsg::VaultList { start_after, limit } => {
            to_binary(&queriers::query_vault_list(deps, start_after, limit)?)
        }
        QueryMsg::Vault { vault_addr } => {
            to_binary(&queriers::query_vault_details(deps, vault_addr)?)
        }
        QueryMsg::ApprovedVaultRateList {} => {
            to_binary(&queriers::query_approved_vaults_fx_rate(deps)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    const REGISTRY_KEY: &[u8] = b"registry";
    // msg pass in an { endowments: [ (address, status, name, owner, tier), ... ] }
    for e in msg.endowments {
        // build key for registrar's endowment
        // let key = [REGISTRY_KEY, e.addr.clone().as_bytes()].concat();

        let path: Path<EndowmentEntry> = Path::new(REGISTRY_KEY, &[e.addr.clone().as_bytes()]);
        let key = path.deref();

        // set the new EndowmentEntry at the given key
        deps.storage.set(
            key,
            &to_vec(&EndowmentEntry {
                address: deps.api.addr_validate(&e.addr)?, // Addr,
                name: e.name,                              // String,
                owner: e.owner,                            // String,
                // EndowmentStatus
                status: match e.status {
                    0 => EndowmentStatus::Inactive,
                    1 => EndowmentStatus::Approved,
                    2 => EndowmentStatus::Frozen,
                    3 => EndowmentStatus::Closed,
                    _ => EndowmentStatus::Inactive,
                },
                // Option<Tier>
                tier: match e.tier {
                    1 => Some(Tier::Level1),
                    2 => Some(Tier::Level2),
                    3 => Some(Tier::Level3),
                    _ => None,
                },
                // UN_SDG Option<u64>
                un_sdg: match e.un_sdg {
                    0 => None,
                    _ => Some(e.un_sdg),
                },
                endow_type: EndowmentType::Charity, // EndowmentType,
                logo: "".to_string(),
                image: "".to_string(),
            })?,
        );
    }
    Ok(Response::default())
}
