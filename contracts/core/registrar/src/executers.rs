use crate::state::{
    read_registry_entries, read_vaults, registry_read, registry_store, vault_read, vault_store,
    CONFIG,
};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::QueryMsg as AccountsQueryMsg;
use angel_core::messages::registrar::*;
use angel_core::responses::accounts::EndowmentDetailsResponse;
use angel_core::responses::registrar::*;
use angel_core::structs::{EndowmentEntry, EndowmentStatus, SplitDetails, YieldVault};
use cosmwasm_std::{
    to_binary, ContractResult, CosmosMsg, DepsMut, Env, MessageInfo, QueryRequest, ReplyOn,
    Response, StdResult, SubMsg, SubMsgExecutionResponse, WasmMsg, WasmQuery,
};
use cw4::Member;
use cw4_group::msg::ExecuteMsg::UpdateMembers;

fn build_account_status_change_msg(account: String, deposit: bool, withdraw: bool) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account,
        msg: to_binary(
            &angel_core::messages::accounts::ExecuteMsg::UpdateEndowmentStatus(
                angel_core::messages::accounts::UpdateEndowmentStatusMsg {
                    deposit_approved: deposit,
                    withdraw_approved: withdraw,
                },
            ),
        )
        .unwrap(),
        funds: vec![],
    });

    SubMsg {
        id: 0,
        msg: wasm_msg,
        gas_limit: None,
        reply_on: ReplyOn::Never,
    }
}

pub fn update_endowment_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentStatusMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) || msg.status > 3 {
        return Err(ContractError::Unauthorized {});
    }

    // look up the endowment in the Registry. Will fail if doesn't exist
    let endowment_addr = msg.endowment_addr.as_bytes();
    let mut endowment_entry = registry_read(deps.storage)
        .may_load(endowment_addr)?
        .unwrap();

    // get config details about the Endowment of interest
    let endowment_info: EndowmentDetailsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: endowment_entry.address.to_string(),
            msg: to_binary(&AccountsQueryMsg::Endowment {})?,
        }))?;

    let msg_endowment_status = match msg.status {
        0 => EndowmentStatus::Inactive,
        1 => EndowmentStatus::Approved,
        2 => EndowmentStatus::Frozen,
        3 => EndowmentStatus::Closed,
        _ => EndowmentStatus::Inactive, // should never be reached due to status check earlier
    };

    // check first that the current status is different from the new status sent
    if endowment_entry.status.to_string() == msg_endowment_status.to_string() {
        return Ok(Response::default());
    }

    // check that the endowment has not been closed (liquidated or terminated) as this is not reversable
    if endowment_entry.status == EndowmentStatus::Closed {
        return Err(ContractError::AccountClosed {});
    }

    // update entry status & save to the Registry
    endowment_entry.status = msg_endowment_status.clone();
    registry_store(deps.storage).save(endowment_addr, &endowment_entry)?;

    // Take different actions on the affected Accounts SC, based on the status passed
    // Build out list of SubMsgs to send to the Account SC and/or Index Fund SC
    // 1. INDEX FUND - Update fund members list removing a member if the member can no longer accept deposits
    // 2. ACCOUNTS - Update the Endowment deposit/withdraw approval config settings based on the new status

    let sub_messages: Vec<SubMsg> = match msg_endowment_status {
        // Allowed to receive donations and process withdrawals
        EndowmentStatus::Approved => {
            vec![
                build_account_status_change_msg(endowment_entry.address.to_string(), true, true),
                // send msg to C4 Endowment Owners group SC to add new member
                SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.endowment_owners_group_addr.unwrap(),
                    msg: to_binary(&UpdateMembers {
                        add: vec![Member {
                            addr: endowment_info.owner.to_string(),
                            weight: 1,
                        }],
                        remove: vec![],
                    })?,
                    funds: vec![],
                })),
            ]
        }
        // Can accept inbound deposits, but cannot withdraw funds out
        EndowmentStatus::Frozen => {
            vec![build_account_status_change_msg(
                endowment_entry.address.to_string(),
                true,
                false,
            )]
        }
        // Has been liquidated or terminated. Remove from Funds and lockdown money flows
        EndowmentStatus::Closed => vec![
            build_account_status_change_msg(endowment_entry.address.to_string(), false, false),
            // trigger the removal of this endowment from all Index Funds
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.index_fund_contract.to_string(),
                msg: to_binary(&angel_core::messages::index_fund::ExecuteMsg::RemoveMember(
                    angel_core::messages::index_fund::RemoveMemberMsg {
                        member: endowment_entry.address.to_string(),
                    },
                ))
                .unwrap(),
                funds: vec![],
            })),
            // send msg to C4 Endowment Owners group SC to remove a member
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.endowment_owners_group_addr.unwrap(),
                msg: to_binary(&UpdateMembers {
                    add: vec![],
                    remove: vec![endowment_info.owner.to_string()],
                })?,
                funds: vec![],
            })),
            // start redemption of Account SC's Vault holdings to final beneficiary/index fund
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: endowment_entry.address.to_string(),
                msg: to_binary(
                    &angel_core::messages::accounts::ExecuteMsg::CloseEndowment {
                        beneficiary: msg.beneficiary,
                    },
                )
                .unwrap(),
                funds: vec![],
            })),
        ],
        _ => vec![],
    };

    Ok(Response::new()
        .add_submessages(sub_messages)
        .add_attribute("action", "update_endowment_status"))
}

pub fn update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner_addr = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed owner
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner_addr;
        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_owner"))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // update config attributes with newly passed configs
    config.approved_charities = msg.charities_list(deps.api)?;
    config.accounts_code_id = msg.accounts_code_id.unwrap_or(config.accounts_code_id);
    config.guardians_multisig_addr = match msg.guardians_multisig_addr {
        Some(v) => Some(deps.api.addr_validate(&v)?.to_string()),
        None => {
            if config.guardians_multisig_addr != None {
                config.guardians_multisig_addr.clone()
            } else {
                None
            }
        }
    };
    config.endowment_owners_group_addr = match msg.endowment_owners_group_addr {
        Some(v) => Some(deps.api.addr_validate(&v)?.to_string()),
        None => {
            if config.endowment_owners_group_addr != None {
                config.endowment_owners_group_addr.clone()
            } else {
                None
            }
        }
    };
    config.default_vault = deps.api.addr_validate(
        &msg.default_vault
            .unwrap_or_else(|| config.default_vault.to_string()),
    )?;
    config.index_fund_contract = deps.api.addr_validate(
        &msg.index_fund_contract
            .unwrap_or_else(|| config.index_fund_contract.to_string()),
    )?;
    config.treasury = deps
        .api
        .addr_validate(&msg.treasury.unwrap_or_else(|| config.treasury.to_string()))?;
    config.tax_rate = msg.tax_rate.unwrap_or(config.tax_rate);
    config.split_to_liquid = SplitDetails {
        max: msg.split_max.unwrap_or(config.split_to_liquid.max),
        min: msg.split_min.unwrap_or(config.split_to_liquid.min),
        default: msg.split_default.unwrap_or(config.split_to_liquid.default),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn create_endowment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check that the sender is an approved charity address
    let pos = config
        .approved_charities
        .iter()
        .position(|a| *a == info.sender);
    // ignore if that member was found in the list
    if pos == None && info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if config.accounts_code_id == 0 {
        return Err(ContractError::ContractNotConfigured {});
    }

    let wasm_msg = WasmMsg::Instantiate {
        code_id: config.accounts_code_id,
        admin: Some(config.owner.to_string()),
        label: "new endowment accounts".to_string(),
        msg: to_binary(&angel_core::messages::accounts::InstantiateMsg {
            owner_sc: config.owner.to_string(),
            registrar_contract: env.contract.address.to_string(),
            owner: msg.owner,
            beneficiary: msg.beneficiary,
            name: msg.name,
            description: msg.description,
            withdraw_before_maturity: msg.withdraw_before_maturity,
            maturity_time: msg.maturity_time,
            maturity_height: msg.maturity_height,
        })?,
        funds: vec![],
    };

    let sub_message = SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(wasm_msg),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(sub_message)
        .add_attribute("action", "create_endowment"))
}

pub fn migrate_accounts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    let mut messages = vec![];
    for endowment in read_registry_entries(deps.storage)?.into_iter() {
        let wasm_msg = WasmMsg::Migrate {
            contract_addr: endowment.address.to_string(),
            new_code_id: config.accounts_code_id,
            msg: to_binary(&angel_core::messages::accounts::MigrateMsg {})?,
        };
        messages.push(CosmosMsg::Wasm(wasm_msg));
    }
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "migrate_accounts"))
}

pub fn charity_add(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    charity: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // save the new charity to the list if it does not already exist
    let addr = deps.api.addr_validate(&charity)?;
    let pos = config.approved_charities.iter().position(|a| *a == addr);
    // ignore if that member was found in the list
    if pos == None {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.approved_charities.push(addr);
            Ok(config)
        })?;
    }

    Ok(Response::default())
}

pub fn charity_remove(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    charity: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // remove the charity from the list if it exists
    let addr = deps.api.addr_validate(&charity)?;
    let pos = config.approved_charities.iter().position(|a| *a == addr);
    // ignore if that member was found in the list
    if pos != None {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.approved_charities.swap_remove(pos.unwrap());
            Ok(config)
        })?;
    }

    Ok(Response::default())
}

pub fn vault_add(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: VaultAddMsg,
) -> Result<Response, ContractError> {
    // save the new vault to storage (defaults to false)
    let addr = deps.api.addr_validate(&msg.vault_addr)?;
    let new_vault = YieldVault {
        address: addr.clone(),
        input_denom: msg.input_denom,
        yield_token: deps.api.addr_validate(&msg.yield_token)?,
        approved: false,
    };
    vault_store(deps.storage).save(addr.as_bytes(), &new_vault)?;
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
    let addr = deps.api.addr_validate(&vault_addr)?;
    let mut vault = vault_read(deps.storage).load(addr.as_bytes())?;

    // update new vault approval status attribute from passed arg
    vault.approved = approved;
    vault_store(deps.storage).save(addr.as_bytes(), &vault)?;

    Ok(Response::default())
}

pub fn new_accounts_reply(
    deps: DepsMut,
    _env: Env,
    msg: ContractResult<SubMsgExecutionResponse>,
) -> Result<Response, ContractError> {
    match msg {
        ContractResult::Ok(subcall) => {
            let mut endowment_addr = String::from("");
            for event in subcall.events {
                if event.ty == *"instantiate_contract" {
                    for attrb in event.attributes {
                        if attrb.key == "contract_address" {
                            endowment_addr = attrb.value;
                        }
                    }
                }
            }
            // Register the new Endowment on success Reply
            let addr = deps.api.addr_validate(&endowment_addr)?;
            registry_store(deps.storage).save(
                addr.clone().as_bytes(),
                &EndowmentEntry {
                    address: addr,
                    status: EndowmentStatus::Inactive,
                },
            )?;
            Ok(Response::default())
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn harvest(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // harvest can only be valid if it comes from the  (AP Team/DANO) SC Owner
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // gets a list of approved Vaults
    let vaults = read_vaults(deps.storage)?;
    let list = VaultListResponse {
        vaults: vaults.into_iter().filter(|p| p.approved).collect(),
    };

    let mut sub_messages: Vec<SubMsg> = vec![];
    for vault in list.vaults.iter() {
        sub_messages.push(harvest_msg(vault.address.to_string()));
    }

    Ok(Response::new()
        .add_submessages(sub_messages)
        .add_attribute("action", "harvest"))
}

fn harvest_msg(account: String) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account,
        msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Harvest {}).unwrap(),
        funds: vec![],
    });

    SubMsg {
        id: 0,
        msg: wasm_msg,
        gas_limit: None,
        reply_on: ReplyOn::Never,
    }
}
