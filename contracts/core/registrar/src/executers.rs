use crate::state::{read_vaults, CONFIG, NETWORK_CONNECTIONS, REGISTRY, VAULTS};
use angel_core::errors::core::ContractError;
use angel_core::messages::registrar::*;
use angel_core::responses::registrar::*;
use angel_core::structs::{
    AcceptedTokens, EndowmentEntry, EndowmentStatus, EndowmentType, NetworkInfo, Tier, YieldVault,
};
use angel_core::utils::{percentage_checks, split_checks};
use cosmwasm_std::{
    attr, to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, ReplyOn, Response, StdResult,
    SubMsg, SubMsgResult, WasmMsg,
};
use cw_utils::Duration;

fn build_account_status_change_msg(
    accounts: String,
    id: u32,
    deposit: bool,
    withdraw: bool,
) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: accounts,
        msg: to_binary(
            &angel_core::messages::accounts::ExecuteMsg::UpdateEndowmentStatus(
                angel_core::messages::accounts::UpdateEndowmentStatusMsg {
                    id,
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
    // look up the endowment in the Registry. Will fail if doesn't exist
    let endowment_id = msg.endowment_id;
    let mut endowment_entry = REGISTRY.load(deps.storage, endowment_id)?;
    let msg_endowment_status = match msg.status {
        0 => EndowmentStatus::Inactive,
        1 => EndowmentStatus::Approved,
        2 => EndowmentStatus::Frozen,
        3 => EndowmentStatus::Closed,
        _ => EndowmentStatus::Inactive, // should never be reached due to status check earlier
    };

    // AP Applications Review: Can only handle Endowments that are still INACTIVE.
    // Can only set a status of either Approved OR Closed.
    // AP Team CW3 (owner): Can only handle Endowments that are NOT Inactive. Can set any status.
    if !((info.sender == config.applications_review
        && msg_endowment_status == EndowmentStatus::Inactive
        && (msg.status == 1 || msg.status == 3))
        || (info.sender == config.owner && msg_endowment_status != EndowmentStatus::Inactive))
        || msg.status > 3
    {
        return Err(ContractError::Unauthorized {});
    }

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
    REGISTRY.save(deps.storage, endowment_id, &endowment_entry)?;

    // Take different actions on the affected Accounts SC, based on the status passed
    // Build out list of SubMsgs to send to the Account SC and/or Index Fund SC
    // 1. INDEX FUND - Update fund members list removing a member if the member can no longer accept deposits
    // 2. ACCOUNTS - Update the Endowment deposit/withdraw approval config settings based on the new status

    let index_fund_contract = match config.index_fund_contract {
        Some(addr) => addr,
        None => return Err(ContractError::ContractNotConfigured {}),
    };

    let accounts_contract = config.accounts_contract.unwrap().to_string();
    let sub_messages: Vec<SubMsg> = match msg_endowment_status {
        // Allowed to receive donations and process withdrawals
        EndowmentStatus::Approved => {
            vec![build_account_status_change_msg(
                accounts_contract,
                endowment_id,
                true,
                true,
            )]
        }
        // Can accept inbound deposits, but cannot withdraw funds out
        EndowmentStatus::Frozen => {
            vec![build_account_status_change_msg(
                accounts_contract,
                endowment_id,
                true,
                false,
            )]
        }
        // Has been liquidated or terminated. Remove from Funds and lockdown money flows
        EndowmentStatus::Closed => vec![
            build_account_status_change_msg(
                accounts_contract.clone(),
                endowment_id.clone(),
                false,
                false,
            ),
            // trigger the removal of this endowment from all Index Funds
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: index_fund_contract.to_string(),
                msg: to_binary(&angel_core::messages::index_fund::ExecuteMsg::RemoveMember(
                    angel_core::messages::index_fund::RemoveMemberMsg {
                        member: msg.endowment_id,
                    },
                ))
                .unwrap(),
                funds: vec![],
            })),
            // start redemption of Account SC's Vault holdings to final beneficiary/index fund
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: accounts_contract,
                msg: to_binary(
                    &angel_core::messages::accounts::ExecuteMsg::CloseEndowment {
                        id: endowment_id,
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
    config.applications_review = match msg.applications_review {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => config.applications_review,
    };
    config.accounts_contract = match msg.accounts_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => config.accounts_contract,
    };
    config.cw3_code = match msg.cw3_code {
        Some(v) => Some(v),
        None => config.cw3_code,
    };
    config.cw4_code = match msg.cw4_code {
        Some(v) => Some(v),
        None => config.cw4_code,
    };
    config.charity_shares_contract = match msg.charity_shares_contract {
        Some(contract_addr) => Some(deps.api.addr_validate(&contract_addr)?),
        None => config.charity_shares_contract,
    };
    config.default_vault = match msg.default_vault {
        Some(addr) => match addr {
            Some(a) => Some(deps.api.addr_validate(&a)?),
            None => None,
        },
        None => config.default_vault,
    };
    config.index_fund_contract = match msg.index_fund_contract {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => config.index_fund_contract,
    };
    config.treasury = deps
        .api
        .addr_validate(&msg.treasury.unwrap_or_else(|| config.treasury.to_string()))?;
    config.tax_rate = match msg.tax_rate {
        Some(tax_rate) => percentage_checks(tax_rate),
        None => Ok(config.tax_rate),
    }
    .unwrap();
    let max = match msg.split_max {
        Some(max) => percentage_checks(max),
        None => Ok(config.split_to_liquid.max),
    };
    let min = match msg.split_min {
        Some(min) => percentage_checks(min),
        None => Ok(config.split_to_liquid.min),
    };
    let default = match msg.split_default {
        Some(default) => percentage_checks(default),
        None => Ok(config.split_to_liquid.default),
    };
    config.split_to_liquid = split_checks(max.unwrap(), min.unwrap(), default.unwrap()).unwrap();
    config.accepted_tokens = AcceptedTokens {
        native: msg
            .accepted_tokens_native
            .unwrap_or(config.accepted_tokens.native),
        cw20: msg
            .accepted_tokens_cw20
            .unwrap_or(config.accepted_tokens.cw20),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn create_endowment(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: CreateEndowmentMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match config.accounts_contract {
        None => Err(ContractError::ContractNotConfigured {}),
        Some(accounts_contract) => {
            let wasm_msg = WasmMsg::Execute {
                contract_addr: accounts_contract.to_string(),
                msg: to_binary(
                    &angel_core::messages::accounts::ExecuteMsg::CreateEndowment(
                        angel_core::messages::accounts::CreateEndowmentMsg {
                            owner: msg.owner,
                            beneficiary: msg.beneficiary,
                            withdraw_before_maturity: msg.withdraw_before_maturity,
                            maturity_time: msg.maturity_time,
                            maturity_height: msg.maturity_height,
                            profile: msg.profile,
                            cw4_members: msg.cw4_members,
                            kyc_donors_only: msg.kyc_donors_only,
                            cw3_threshold: msg.cw3_threshold,
                            cw3_max_voting_period: Duration::Time(msg.cw3_max_voting_period),
                        },
                    ),
                )?,
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
    }
}

pub fn vault_add(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: VaultAddMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // validate the passed address
    let addr = deps.api.addr_validate(&msg.vault_addr)?;

    // check that the vault does not already exist for a given address in storage
    if VAULTS.load(deps.storage, addr.as_bytes()).is_ok() {
        return Err(ContractError::VaultAlreadyExists {});
    }

    // save the new vault to storage
    VAULTS.save(
        deps.storage,
        addr.as_bytes(),
        &YieldVault {
            network: msg.network.unwrap_or(env.block.chain_id),
            address: addr.to_string(),
            input_denom: msg.input_denom,
            yield_token: deps.api.addr_validate(&msg.yield_token)?.to_string(),
            approved: false,
            restricted_from: msg.restricted_from,
        },
    )?;
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
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // validate the passed address
    let _addr = deps.api.addr_validate(&vault_addr)?;

    // remove the vault from storage
    VAULTS.remove(deps.storage, vault_addr.as_bytes());
    Ok(Response::default())
}

pub fn vault_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
    approved: bool,
    restricted_from: Vec<EndowmentType>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given vault in Storage
    let addr = deps.api.addr_validate(&vault_addr)?;
    let mut vault = VAULTS.load(deps.storage, addr.as_bytes())?;

    // update new vault approval status attribute from passed arg
    vault.approved = approved;
    // set any restricted endowment types
    vault.restricted_from = restricted_from;
    VAULTS.save(deps.storage, addr.as_bytes(), &vault)?;

    Ok(Response::default())
}

pub fn new_accounts_reply(
    deps: DepsMut,
    _env: Env,
    msg: SubMsgResult,
) -> Result<Response, ContractError> {
    match msg {
        SubMsgResult::Ok(subcall) => {
            let mut endowment_id: u32 = 0;
            let mut endowment_name = String::from("");
            let mut endowment_owner = String::from("");
            let mut endowment_type = String::from("");
            let mut endowment_logo = String::from("");
            let mut endowment_image = String::from("");
            let mut endowment_tier: u8 = 0;
            let mut endowment_un_sdg: u8 = 0;
            for event in subcall.events {
                if event.ty == *"wasm" {
                    for attrb in event.attributes {
                        match attrb.key.as_str() {
                            "endow_id" => endowment_id = attrb.value.parse().unwrap(),
                            "endow_name" => endowment_name = attrb.value,
                            "endow_owner" => endowment_owner = attrb.value,
                            "endow_type" => endowment_type = attrb.value,
                            "endow_logo" => endowment_logo = attrb.value,
                            "endow_image" => endowment_image = attrb.value,
                            "endow_tier" => endowment_tier = attrb.value.parse().unwrap(),
                            "endow_un_sdg" => endowment_un_sdg = attrb.value.parse().unwrap(),
                            &_ => (),
                        }
                    }
                }
            }
            // Register the new Endowment on success Reply
            REGISTRY.save(
                deps.storage,
                endowment_id,
                &EndowmentEntry {
                    id: endowment_id.clone(),
                    owner: endowment_owner.clone(),
                    status: EndowmentStatus::Inactive,
                    endow_type: match endowment_type.as_str() {
                        "charity" => EndowmentType::Charity,
                        "normal" => EndowmentType::Normal,
                        _ => unimplemented!(),
                    },
                    name: Some(endowment_name),
                    tier: match endowment_tier {
                        1 => Some(Tier::Level1),
                        2 => Some(Tier::Level2),
                        3 => Some(Tier::Level3),
                        _ => None,
                    },
                    un_sdg: Some(endowment_un_sdg),
                    logo: Some(endowment_logo),
                    image: Some(endowment_image),
                },
            )?;
            Ok(Response::default().add_attributes(vec![
                attr("reply", "create_endowment"),
                attr("endowment_id", endowment_id.to_string()),
                attr("owner", endowment_owner),
            ]))
        }
        SubMsgResult::Err(_) => Err(ContractError::AccountNotCreated {}),
    }
}

pub fn harvest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collector_address: String,
    collector_share: Decimal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // harvest can only be valid if it comes from the  (AP Team/DANO) SC Owner
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // gets a list of approved Vaults
    let vaults = read_vaults(deps.storage, None, None, Some(true), None, None)?;
    let list = VaultListResponse { vaults };

    let mut sub_messages: Vec<SubMsg> = vec![];
    for vault in list.vaults.iter() {
        sub_messages.push(harvest_msg(
            vault.address.to_string(),
            collector_address.clone(),
            collector_share,
        ));
    }

    Ok(Response::new()
        .add_submessages(sub_messages)
        .add_attribute("action", "harvest"))
}

fn harvest_msg(account: String, collector_address: String, collector_share: Decimal) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: account,
        msg: to_binary(&angel_core::messages::vault::ExecuteMsg::Harvest {
            collector_address,
            collector_share,
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

pub fn update_endowment_entry(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentEntryMsg,
) -> Result<Response, ContractError> {
    // look up the endowment in the Registry. Will fail if doesn't exist
    let mut endowment_entry = REGISTRY.load(deps.storage, msg.endowment_id)?;

    let config = CONFIG.load(deps.storage)?;
    if !(info.sender == config.owner || info.sender == endowment_entry.owner) {
        return Err(ContractError::Unauthorized {});
    }

    endowment_entry.name = msg.name;
    endowment_entry.owner = msg.owner.unwrap_or(endowment_entry.owner);
    endowment_entry.endow_type = msg.endow_type.unwrap_or(endowment_entry.endow_type);
    endowment_entry.logo = msg.logo;
    endowment_entry.image = msg.image;

    if let Some(tier) = msg.tier {
        endowment_entry.tier = tier;
    }

    REGISTRY.save(deps.storage, msg.endowment_id, &endowment_entry)?;

    Ok(Response::new().add_attribute("action", "update_endowment_entry"))
}

pub fn update_network_connections(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    network_info: NetworkInfo,
    action: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    if action == *"add" {
        // Add the network_info to NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.save(deps.storage, &network_info.chain_id, &network_info)?;
    } else if action == *"remove" {
        // Remove the network_info from NETWORK_CONNECTIONS
        NETWORK_CONNECTIONS.remove(deps.storage, &network_info.chain_id);
    } else {
        return Err(ContractError::InvalidInputs {});
    }

    Ok(Response::default().add_attribute("action", "update_network_connections"))
}
