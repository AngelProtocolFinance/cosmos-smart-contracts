use crate::state::{
    read_registry_entries, read_vaults, registry_read, registry_store, vault_read, vault_store,
    Config, CONFIG,
};
use angel_core::error::ContractError;
use angel_core::registrar_msg::*;
use angel_core::registrar_rsp::*;
use angel_core::structs::{AssetVault, EndowmentEntry, EndowmentStatus, SplitDetails};
use cosmwasm_std::{
    attr, entry_point, to_binary, Binary, ContractResult, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, SubMsgExecutionResponse, WasmMsg,
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
    let index_fund_contract_addr = deps.api.addr_validate(
        &msg.index_fund_contract
            .unwrap_or("XXXXXXXXXXXXXXXXXXXXXXXX".to_string()),
    )?;

    let configs = Config {
        owner: info.sender,
        index_fund_contract: index_fund_contract_addr,
        approved_coins: msg.approved_coins.unwrap_or(vec![]),
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
        ExecuteMsg::VaultAdd {
            vault_addr,
            vault_name,
            vault_description,
        } => vault_add(deps, env, info, vault_addr, vault_name, vault_description),
        ExecuteMsg::VaultUpdateStatus {
            vault_addr,
            approved,
        } => vault_update_status(deps, env, info, vault_addr, approved),
        ExecuteMsg::VaultRemove { vault_addr } => vault_remove(deps, env, info, vault_addr),
    }
}

fn build_account_status_change_msg(
    account: String,
    deposit: bool,
    withdraw: bool,
) -> StdResult<SubMsg> {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account,
        msg: to_binary(&angel_core::accounts_msg::UpdateEndowmentStatusMsg {
            deposit_approved: deposit,
            withdraw_approved: withdraw,
        })?,
        funds: vec![],
    });

    Ok(SubMsg {
        id: 0,
        msg: wasm_msg,
        gas_limit: None,
        reply_on: ReplyOn::Never,
    })
}

fn build_index_fund_member_removal_msg(account: String) -> StdResult<SubMsg> {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account.clone(),
        msg: to_binary(&angel_core::index_fund_msg::RemoveMemberMsg { member: account })?,
        funds: vec![],
    });

    Ok(SubMsg {
        id: 0,
        msg: wasm_msg,
        gas_limit: None,
        reply_on: ReplyOn::Never,
    })
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
    let endowment_addr = msg.endowment_addr.as_bytes();
    let mut endowment_entry = registry_read(deps.storage)
        .may_load(endowment_addr)?
        .unwrap();

    // check first that the current status is different from the new status sent
    if endowment_entry.status == msg.status {
        return Ok(Response::default());
    }

    // check that the endowment has not been closed (liquidated or terminated) as this is not reversable
    if endowment_entry.status == EndowmentStatus::Closed {
        return Err(ContractError::AccountClosed {});
    }

    // update entry status & save to the Registry
    endowment_entry.status = msg.status.clone();
    registry_store(deps.storage).save(endowment_addr, &endowment_entry)?;

    // Take different actions on the affected Accounts SC, based on the status passed
    // Build out list of SubMsgs to send to the Account SC and/or Index Fund SC
    // 1. INDEX FUND - Update fund members list removing a member if the member can no longer accept deposits
    // 2. ACCOUNTS - Update the Endowment deposit/withdraw approval config settings based on the new status

    let sub_messages: Vec<SubMsg> = match msg.status {
        // Allowed to receive donations and process withdrawals
        EndowmentStatus::Approved => {
            vec![
                build_account_status_change_msg(endowment_entry.address.to_string(), true, true)
                    .unwrap(),
            ]
        }
        // Can accept inbound deposits, but cannot withdraw funds out
        EndowmentStatus::Frozen => {
            vec![
                build_account_status_change_msg(endowment_entry.address.to_string(), true, false)
                    .unwrap(),
            ]
        }
        // Has been liquidated or terminated. Remove from Funds and lockdown money flows
        EndowmentStatus::Closed => vec![
            build_account_status_change_msg(endowment_entry.address.to_string(), true, true)
                .unwrap(),
            build_index_fund_member_removal_msg(endowment_entry.address.to_string()).unwrap(),
        ],
        _ => vec![],
    };

    let res = Response {
        messages: sub_messages,
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
    env: Env,
    _info: MessageInfo,
    msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.accounts_code_id == 0 {
        return Err(ContractError::ContractNotConfigured {});
    }

    let wasm_msg = WasmMsg::Instantiate {
        code_id: config.accounts_code_id,
        admin: Some(env.contract.address.to_string()),
        label: "new endowment accounts".to_string(),
        msg: to_binary(&angel_core::accounts_msg::InstantiateMsg {
            admin_addr: config.owner.to_string(),
            registrar_contract: env.contract.address.to_string(),
            index_fund_contract: config.index_fund_contract.to_string(),
            endowment_owner: msg.endowment_owner,
            endowment_beneficiary: msg.endowment_beneficiary,
            name: msg.name,
            description: msg.description,
            withdraw_before_maturity: msg.withdraw_before_maturity,
            maturity_time: msg.maturity_time,
            maturity_height: msg.maturity_height,
            split_to_liquid: msg.split_to_liquid.unwrap_or(SplitDetails::default()),
        })?,
        funds: vec![],
    };

    let sub_message = SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(wasm_msg),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    let res = Response {
        messages: vec![sub_message],
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
    vault_name: String,
    vault_description: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // save the new vault to storage (defaults to true)
    let addr = deps.api.addr_validate(&vault_addr)?;
    let new_vault = AssetVault {
        address: addr.clone(),
        name: vault_name,
        description: vault_description,
        approved: true,
    };
    vault_store(deps.storage).save(&addr.as_bytes(), &new_vault)?;
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
    let addr = deps.api.addr_validate(&vault_addr.clone())?;
    let mut vault = vault_read(deps.storage).load(&addr.as_bytes())?;

    // update new vault approval status attribute from passed arg
    vault.approved = approved;
    vault_store(deps.storage).save(&addr.as_bytes(), &vault)?;

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
    let addr = deps.api.addr_validate(&vault_addr.clone())?;
    let _vault = vault_read(deps.storage).load(&addr.as_bytes())?;

    // delete the vault
    vault_store(deps.storage).remove(&addr.as_bytes());

    Ok(Response::default())
}

/// Replies back to the registrar from instantiate calls to Accounts SC (@ some code_id)
/// should be cuaght and handled to register the Endowment's newly created Accounts SC
/// in the REGISTRY storage
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => new_accounts_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

pub fn new_accounts_reply(
    deps: DepsMut,
    env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    let addr = env.contract.address.clone();
    match msg {
        ContractResult::Ok(_subcall) => {
            // Register the new Endowment on success Reply
            registry_store(deps.storage).save(
                &addr.as_bytes(),
                &EndowmentEntry {
                    address: env.contract.address,
                    name: "".to_string(),
                    description: "".to_string(),
                    status: EndowmentStatus::Inactive,
                },
            )?;
            Ok(Response::default())
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::EndowmentList {} => to_binary(&query_endowment_list(deps)?),
        QueryMsg::Vault { vault_addr } => to_binary(&query_vault_details(deps, vault_addr)?),
        QueryMsg::VaultList {} => to_binary(&query_vault_list(deps)?),
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

fn query_vault_details(deps: Deps, vault_addr: String) -> StdResult<VaultDetailResponse> {
    // this fails if no vault is found
    let addr = deps.api.addr_validate(&vault_addr)?;
    let vault = vault_read(deps.storage).load(&addr.as_bytes())?;
    let details = VaultDetailResponse { vault: vault };
    Ok(details)
}

fn query_vault_list(deps: Deps) -> StdResult<VaultListResponse> {
    let vaults = read_vaults(deps.storage)?;
    let list = VaultListResponse { vaults: vaults };
    Ok(list)
}

fn query_endowment_list(deps: Deps) -> StdResult<EndowmentListResponse> {
    let endowments = read_registry_entries(deps.storage)?;
    let list = EndowmentListResponse {
        endowments: endowments,
    };
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
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    const MOCK_ACCOUNTS_CODE_ID: u64 = 17;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            index_fund_contract: Some("INDEXTHADFARHSRTHADGG".to_string()),
            approved_coins: Some(vec![]),
            accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config_response: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(MOCK_ACCOUNTS_CODE_ID, config_response.accounts_code_id);
        assert_eq!("creator", config_response.owner);
    }

    #[test]
    fn update_owner() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            index_fund_contract: Some("INDEXTHADFARHSRTHADGG".to_string()),
            approved_coins: Some(vec![]),
            accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("ill-wisher", &coins(1000, "earth"));
        let msg = ExecuteMsg::UpdateOwner {
            new_owner: String::from("alice"),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        assert_eq!(ContractError::Unauthorized {}, _res.unwrap_err());

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::UpdateOwner {
            new_owner: String::from("alice"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
