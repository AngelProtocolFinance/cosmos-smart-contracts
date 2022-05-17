use crate::executers;
use crate::queriers;
use crate::state::{Config, OldConfig, CONFIG};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::structs::{EndowmentEntry, EndowmentStatus, EndowmentType, SplitDetails, Tier};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    entry_point, from_slice, to_binary, to_vec, Binary, Decimal, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
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
        donation_match_charites_contract: None,
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
        ExecuteMsg::UpdateEndowmentEntry(msg) => {
            executers::update_endowment_entry(deps, env, info, msg)
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
        QueryMsg::Fees {} => to_binary(&queriers::query_fees(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
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
        donation_match_charites_contract: None,
        collector_addr: Some(collector_addr),
        collector_share: Decimal::percent(50_u64),
    };
    deps.storage.set(CONFIG_KEY, &to_vec(&config)?);

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
