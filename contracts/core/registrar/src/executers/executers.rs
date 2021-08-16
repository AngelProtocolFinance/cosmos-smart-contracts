use crate::state::{portal_read, portal_store, registry_read, registry_store, CONFIG};
use angel_core::error::ContractError;
use angel_core::registrar_msg::*;
use angel_core::structs::{EndowmentEntry, EndowmentStatus, SplitDetails, YieldPortal};
use cosmwasm_std::{
    to_binary, ContractResult, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdResult,
    SubMsg, SubMsgExecutionResponse, WasmMsg,
};

fn build_account_status_change_msg(account: String, deposit: bool, withdraw: bool) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account,
        msg: to_binary(&angel_core::accounts_msg::UpdateEndowmentStatusMsg {
            deposit_approved: deposit,
            withdraw_approved: withdraw,
        })
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

fn build_index_fund_member_removal_msg(account: String) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account.clone(),
        msg: to_binary(&angel_core::index_fund_msg::RemoveMemberMsg { member: account }).unwrap(),
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

    if info.sender.ne(&config.owner) {
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
            vec![build_account_status_change_msg(
                endowment_entry.address.to_string(),
                true,
                true,
            )]
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
            build_account_status_change_msg(endowment_entry.address.to_string(), true, true),
            build_index_fund_member_removal_msg(endowment_entry.address.to_string()),
        ],
        _ => vec![],
    };

    let mut res = Response::new().add_attribute("action", "update_endowment_status");
    res.messages = sub_messages;

    Ok(res)
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
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    let index_fund_contract_addr = deps.api.addr_validate(&msg.index_fund_contract)?;
    let charities_addr_list = msg.charities_list(deps.api)?;

    // update config attributes with newly passed configs
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.index_fund_contract = index_fund_contract_addr;
        config.accounts_code_id = msg.accounts_code_id.unwrap_or(config.accounts_code_id);
        config.approved_charities = charities_addr_list;
        Ok(config)
    })?;

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
    if pos == None {
        return Err(ContractError::Unauthorized {});
    }

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
            owner: msg.owner,
            beneficiary: msg.beneficiary,
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

    Ok(Response::new()
        .add_submessage(sub_message)
        .add_attribute("action", "create_endowment"))
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

pub fn portal_add(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: PortalAddMsg,
) -> Result<Response, ContractError> {
    // save the new portal to storage (defaults to false)
    let addr = deps.api.addr_validate(&msg.portal_addr)?;
    let new_portal = YieldPortal {
        address: addr.clone(),
        input_denom: msg.input_denom,
        yield_token: deps.api.addr_validate(&msg.yield_token)?,
        deposit_token: deps.api.addr_validate(&msg.deposit_token)?,
        approved: false,
    };
    portal_store(deps.storage).save(&addr.as_bytes(), &new_portal)?;
    Ok(Response::default())
}

pub fn portal_update_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    portal_addr: String,
    approved: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given portal in Storage
    let addr = deps.api.addr_validate(&portal_addr.clone())?;
    let mut portal = portal_read(deps.storage).load(&addr.as_bytes())?;

    // update new portal approval status attribute from passed arg
    portal.approved = approved;
    portal_store(deps.storage).save(&addr.as_bytes(), &portal)?;

    Ok(Response::default())
}

pub fn portal_remove(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    portal_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given portal
    let addr = deps.api.addr_validate(&portal_addr.clone())?;
    // remove the portal
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
            let mut endowment_name = String::from("");
            let mut endowment_desc = String::from("");
            for event in subcall.events {
                if event.ty == "message".to_string() {
                    for attrb in event.attributes {
                        if attrb.key == "name" {
                            endowment_name = attrb.value;
                        } else if attrb.key == "name" {
                            endowment_desc = attrb.value;
                        }
                    }
                } else if event.ty == "instantiate_contract".to_string() {
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
                &addr.clone().as_bytes(),
                &EndowmentEntry {
                    address: addr,
                    name: endowment_name,
                    description: endowment_desc,
                    status: EndowmentStatus::Inactive,
                },
            )?;
            Ok(Response::default())
        }
        ContractResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}
