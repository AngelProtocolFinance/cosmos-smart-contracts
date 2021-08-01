use crate::state::{Config, CONFIG, REGISTRY, VAULTS};
use angel_core::error::ContractError;
use angel_core::registrar_msg::{
    CreateEndowmentMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, UpdateConfigMsg,
    UpdateEndowmentStatusMsg,
};
use angel_core::registrar_rsp::{ConfigResponse, VaultDetailsResponse, VaultListResponse};
use angel_core::structs::SplitDetails;
use cosmwasm_std::{
    attr, entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};

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
    let configs = Config {
        owner: info.sender, // msg.endowment.owner,
        index_fund_contract: deps
            .api
            .addr_validate(&"XXXXXXXXXXXXXXXXXXXXXXXX".to_string())?,
        approved_coins: vec![],
        accounts_code_id: msg.accounts_code_id.unwrap_or(0 as u64),
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
        ExecuteMsg::CreateEndowment(msg) => execute_create_endowment(deps, env, info, msg),
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            execute_update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::UpdateOwner { new_owner } => execute_update_owner(deps, env, info, new_owner),
        ExecuteMsg::VaultAdd { vault_addr } => vault_add(deps, env, info, vault_addr),
        ExecuteMsg::VaultUpdateStatus {
            vault_addr,
            approved,
        } => vault_update_status(deps, env, info, vault_addr, approved),
        ExecuteMsg::VaultRemove { vault_addr } => vault_remove(deps, env, info, vault_addr),
    }
}

pub fn execute_update_endowment_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentStatusMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // look up the endowment in the Registry. Will fail if doesn't exist
    let _endowment_status = REGISTRY.load(deps.storage, msg.address.to_string())?;
    // save the new endowment status to the Registry
    REGISTRY.save(deps.storage, msg.address, &msg.status)?;

    // TO DO: Take different actions based on the status passed
    // if msg.status == EndowmentStatus::Approved {
    //     // Allowed to receive donations and process withdrawals
    // }

    let res = Response {
        messages: vec![
            // TO DO: Send msg to the Accounts SC being updated to inform them of the new status changes
            // TO DO: Send msg to the Index Fund SC to inform them of the new status changes for the given endowment
        ],
        attributes: vec![attr("action", "update_endowment_status")],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner_addr = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed owner
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner_addr;
        Ok(config)
    })?;

    let res = Response {
        attributes: vec![attr("action", "update_owner")],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let index_fund_contract_addr = deps.api.addr_validate(&msg.index_fund_contract)?;
    let coins_addr_list = msg.addr_approved_list(deps.api)?;

    // update config attributes with newly passed configs
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.index_fund_contract = index_fund_contract_addr;
        config.accounts_code_id = msg.accounts_code_id.unwrap_or(config.accounts_code_id);
        config.approved_coins = coins_addr_list;
        Ok(config)
    })?;
    let res = Response {
        attributes: vec![attr("action", "update_owner")],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_create_endowment(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.accounts_code_id == 0 {
        return Err(ContractError::ContractNotConfigured {});
    }

    // /// Register the new Endowment on success Reply
    // REGISTRY.save(
    //     deps.storage,
    //     info.sender.to_string(),
    //     &EndowmentStatus::Inactive,
    // );

    let res = Response {
        messages: vec![SubMsg::new(WasmMsg::Instantiate {
            code_id: config.accounts_code_id,
            admin: Some(config.owner.to_string()),
            label: "new endowment".to_string(),
            msg: to_binary(&angel_core::accounts_msg::InstantiateMsg {
                admin_addr: config.owner.to_string(),
                index_fund_contract: config.index_fund_contract.to_string(),
                endowment_owner: msg.endowment_owner,
                endowment_beneficiary: msg.endowment_beneficiary,
                deposit_approved: msg.deposit_approved,
                withdraw_approved: msg.withdraw_approved,
                withdraw_before_maturity: msg.withdraw_before_maturity,
                maturity_time: msg.maturity_time,
                maturity_height: msg.maturity_height,
                split_to_liquid: SplitDetails::default(),
            })?,
            funds: vec![],
        })],
        attributes: vec![attr("action", "create_endowment")],
        ..Response::default()
    };
    Ok(res)
}

pub fn vault_add(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // save the new vault to storage (defaults to true)
    VAULTS.save(deps.storage, vault_addr.clone(), &true)?;
    Ok(Response::default())
}

pub fn vault_update_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
    approved: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given vault in Storage
    let _vault = VAULTS.load(deps.storage, vault_addr.clone())?;

    // update new vault approval status attribute from passed arg
    VAULTS.save(deps.storage, vault_addr, &approved)?;
    Ok(Response::default())
}

pub fn vault_remove(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given vault in Storage
    let _vault = VAULTS.load(deps.storage, vault_addr.clone())?;
    // delete the vault
    VAULTS.remove(deps.storage, vault_addr);
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Vault { address } => to_binary(&query_vault_details(deps, address)?),
        QueryMsg::VaultList { non_approved } => to_binary(&query_vault_list(deps, non_approved)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        owner: config.owner.to_string(),
        approved_coins: config.human_approved_coins(),
        accounts_code_id: config.accounts_code_id,
    };
    Ok(res)
}

fn query_vault_details(deps: Deps, address: String) -> StdResult<VaultDetailsResponse> {
    // this fails if no vault is found
    let details = VaultDetailsResponse {
        address: address.clone(),
        approved: VAULTS.load(deps.storage, address)?,
    };
    Ok(details)
}

fn query_vault_list(_deps: Deps, _non_approved: Option<bool>) -> StdResult<VaultListResponse> {
    let list = VaultListResponse { vaults: vec![] };
    Ok(list)
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            accounts_code_id: Some(0u64),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
