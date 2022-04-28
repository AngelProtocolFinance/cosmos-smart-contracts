use crate::executers;
use crate::queriers;
use crate::state::{endow_type_fees_write, Config, OldConfig, CONFIG};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::structs::{EndowmentEntry, EndowmentStatus, EndowmentType, SplitDetails, Tier};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    entry_point, to_binary, to_vec, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};
use cosmwasm_std::{from_slice, Decimal, StdError};
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
        owner: info.sender,
        index_fund_contract: None,
        accounts_code_id: msg.accounts_code_id.unwrap_or(0u64),
        treasury: deps.api.addr_validate(&msg.treasury)?,
        tax_rate,
        default_vault: msg.default_vault,
        cw3_code: None,
        cw4_code: None,
        subdao_gov_code: None,
        subdao_token_code: None,
        subdao_cw900_code: None,
        subdao_distributor_code: None,
        donation_match_code: None,
        split_to_liquid: splits,
        halo_token: None,
        gov_contract: None,
        collector_addr: None,
        collector_share: Decimal::percent(50_u64),
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
        ExecuteMsg::UpdateEndowTypeFees(msg) => {
            executers::update_endowtype_fees(deps, env, info, msg)
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
            endow_type,
        } => to_binary(&queriers::query_endowment_list(
            deps, name, owner, status, tier, endow_type,
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
        QueryMsg::Fees {} => to_binary(&queriers::query_fees(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    // Update the `Config`
    // NOTE: This block should be removed after the `registrar`
    //       contract is migrated to `v2`.
    const CONFIG_KEY: &[u8] = b"config";
    let old_config_data = deps.storage.get(CONFIG_KEY).ok_or_else(|| {
        ContractError::Std(StdError::GenericErr {
            msg: "Not found Config".to_string(),
        })
    })?;
    let old_config: OldConfig = from_slice(&old_config_data)?;

    let default_collector_addr = deps
        .api
        .addr_validate("terra1uxqjsgnq30lg5lhlhwd2gmct844vwqcdlv93x5")?;
    let collector_addr = msg.collector_addr.map_or(default_collector_addr, |addr| {
        deps.api.addr_validate(&addr).unwrap()
    });

    let config: Config = Config {
        owner: old_config.owner,
        index_fund_contract: old_config.index_fund_contract,
        accounts_code_id: old_config.accounts_code_id,
        treasury: old_config.treasury,
        tax_rate: old_config.tax_rate,
        default_vault: old_config.default_vault,
        cw3_code: old_config.cw3_code,
        cw4_code: old_config.cw4_code,
        subdao_gov_code: old_config.subdao_gov_code,
        subdao_token_code: old_config.subdao_token_code,
        subdao_cw900_code: old_config.subdao_cw900_code,
        subdao_distributor_code: old_config.subdao_distributor_code,
        donation_match_code: old_config.donation_match_code,
        split_to_liquid: old_config.split_to_liquid,
        halo_token: old_config.halo_token,
        gov_contract: old_config.gov_contract,
        collector_addr: Some(collector_addr),
        collector_share: Decimal::percent(50_u64),
    };
    deps.storage.set(CONFIG_KEY, &to_vec(&config)?);

    // Update the `registry`
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
                endow_type: EndowmentType::Charity, // EndowmentType,
            })?,
        );
    }

    // Save the values for "EndowTypeFees" map
    const ENDOWTYPE_FEES_KEY: &[u8] = b"endowment_type_fees";

    let charity_path: Path<String> = Path::new(
        ENDOWTYPE_FEES_KEY,
        &[EndowmentType::Charity.to_string().as_bytes()],
    );
    let normal_path: Path<String> = Path::new(
        ENDOWTYPE_FEES_KEY,
        &[EndowmentType::Normal.to_string().as_bytes()],
    );

    deps.storage.set(
        charity_path.deref(),
        &to_vec(&msg.endowtype_fees.endowtype_charity)?,
    );
    deps.storage.set(
        normal_path.deref(),
        &to_vec(&msg.endowtype_fees.endowtype_normal)?,
    );

    Ok(Response::default())
}
