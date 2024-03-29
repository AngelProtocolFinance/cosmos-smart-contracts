use crate::state::{read_funds, CONFIG, FUND, STATE};
use angel_core::errors::core::ContractError;
use angel_core::msgs::index_fund::*;
use angel_core::msgs::registrar::QueryMsg as RegistrarQuerier;
use angel_core::msgs::registrar::{
    ConfigExtensionResponse as RegistrarConfigExtensionResponse,
    ConfigResponse as RegistrarConfigResponse,
};
use angel_core::structs::{IndexFund, SplitDetails};
use angel_core::utils::{percentage_checks, validate_deposit_fund};
use cosmwasm_std::{
    attr, to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg, WasmQuery,
};
use cw_asset::{Asset, AssetInfoBase};

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update their address in the configs
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    let new_owner = deps.api.addr_validate(&new_owner)?;
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_registrar(
    deps: DepsMut,
    info: MessageInfo,
    new_registrar: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the registrar contract can update it's address in the config
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_registrar = deps.api.addr_validate(&new_registrar)?;
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.registrar_contract = new_registrar;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_alliance_member_list(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
    action: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update the Alliance Members List
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // validate the member address
    let member_addr = deps.api.addr_validate(address.as_str())?;
    let pos = config
        .alliance_members
        .iter()
        .position(|m| m == &member_addr);

    if action == *"add" {
        if pos.is_none() {
            config.alliance_members.push(member_addr);
        }
    } else if action == *"remove" {
        if pos.is_some() {
            config.alliance_members.swap_remove(pos.unwrap());
        }
    } else {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Invalid action".to_string(),
        }));
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![
        attr("method", "update_alliance_list"),
        attr("action", action),
    ]))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the SC owner can update these configs
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    config.funding_goal = match msg.funding_goal {
        Some(goal) => {
            // underflow check - goal value cannot be lte round_donations
            let state = STATE.load(deps.storage)?;
            if goal <= state.round_donations {
                return Err(ContractError::InvalidInputs {});
            }
            Some(goal) // config set as optional, don't unwrap
        }
        None => None,
    };
    config.fund_rotation = msg.fund_rotation; // config set as optional, don't unwrap
    config.fund_member_limit = msg.fund_member_limit.unwrap_or(config.fund_member_limit);

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn create_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    description: String,
    members: Vec<u32>,
    rotating_fund: Option<bool>,
    split_to_liquid: Option<Decimal>,
    expiry_time: Option<u64>,
    expiry_height: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    let optional_split = split_to_liquid.map(|split| percentage_checks(split).unwrap());

    // build fund struct from msg params
    let fund = IndexFund {
        id: state.next_fund_id,
        name,
        description,
        members,
        rotating_fund,
        split_to_liquid: optional_split,
        expiry_time,
        expiry_height,
    };

    // check if this is the first fund being added in...
    if read_funds(deps.storage, None, None)?.is_empty() {
        state.active_fund = fund.id;
    }
    state.total_funds += 1;
    state.next_fund_id += 1;
    STATE.save(deps.storage, &state)?;

    // Add the new Fund to storage
    FUND.save(deps.storage, &fund.id.to_be_bytes(), &fund)?;

    Ok(Response::default())
}

pub fn remove_index_fund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fund_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // decrement state funds totals
    let mut state = STATE.load(deps.storage)?;
    // check if this is the active fund, update the active_fund using rotate_fund
    if state.active_fund == fund_id {
        state.active_fund = rotate_fund(
            read_funds(deps.storage, None, None).unwrap(),
            fund_id,
            env.block.height,
            env.block.time,
        );
    }
    state.total_funds -= 1;
    STATE.save(deps.storage, &state)?;

    // remove the fund from storage
    FUND.remove(deps.storage, &fund_id.to_be_bytes());

    Ok(Response::default())
}

pub fn update_fund_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fund_id: u64,
    add: Vec<u32>,
    remove: Vec<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // this will fail if fund ID passed is not found
    let mut fund = FUND.load(deps.storage, &fund_id.to_be_bytes())?;

    if fund.is_expired(env.block.height, env.block.time) {
        return Err(ContractError::IndexFundExpired {});
    }

    // add members to the fund, only if they do not already exist
    for add in add.into_iter() {
        let pos = fund.members.iter().position(|m| *m == add);
        // ignore if that member was found in the list
        if pos.is_none() {
            fund.members.push(add);
        }
    }

    // remove the members from the fund
    for remove in remove.into_iter() {
        // ignore if no member is found
        if let Some(pos) = fund.members.iter().position(|m| *m == remove) {
            fund.members.swap_remove(pos);
        }
    }

    // check that the final number of fund members is still under the upper limit
    if fund.members.len() as u32 > config.fund_member_limit {
        return Err(ContractError::IndexFundMembershipExceeded {});
    }

    // save revised fund to storage
    FUND.save(deps.storage, &fund_id.to_be_bytes(), &fund)?;

    Ok(Response::default())
}

pub fn remove_member(
    deps: DepsMut,
    info: MessageInfo,
    member: u32,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigExtensionResponse = deps.querier.query_wasm_smart(
        config.registrar_contract,
        &RegistrarQuerier::ConfigExtension {},
    )?;

    if let Some(accounts_contract) = registrar_config.accounts_contract {
        if info.sender != accounts_contract {
            return Err(ContractError::Unauthorized {});
        }
    } else {
        return Err(ContractError::Std(StdError::generic_err(
            "Accounts contract has not been set in registrar contract config".to_string(),
        )));
    }

    // Check all Funds for the given member and remove the member if found
    let funds = read_funds(deps.storage, None, None)?;
    for mut fund in funds.into_iter() {
        fund.members.retain(|m| m != &member);
        FUND.save(deps.storage, &fund.id.to_be_bytes(), &fund)?;
    }
    Ok(Response::default())
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender_addr: Addr,
    msg: DepositMsg,
    fund: Asset,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    let deposit_fund =
        validate_deposit_fund(deps.as_ref(), config.registrar_contract.as_str(), fund)?;
    let mut deposit_amount = deposit_fund.amount;

    // check each of the currenly allowed Alliance member addr
    let alliance_member = match config
        .alliance_members
        .iter()
        .position(|m| *m == sender_addr)
    {
        None => false,
        _ => true,
    };

    // check if block height limit is exceeded
    if let Some(blocks) = config.fund_rotation {
        match env.block.height >= state.next_rotation_block {
            true => {
                // update STATE with new active fund & reset round donations
                let new_fund_id = rotate_fund(
                    read_funds(deps.storage, None, None).unwrap(),
                    state.active_fund,
                    env.block.height,
                    env.block.time,
                );
                state.active_fund = new_fund_id;
                state.round_donations = Uint128::zero();
                // increment next block rotation point until it exceeds the current block height
                while env.block.height >= state.next_rotation_block {
                    state.next_rotation_block += blocks;
                }
            }
            false => (),
        }
    };

    let registrar_config_ext: RegistrarConfigExtensionResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::ConfigExtension {})?,
        }))?;
    // Get the Registrar SC Split to liquid parameters
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let registrar_split_configs: SplitDetails = registrar_config.split_to_liquid;

    let mut donation_messages: Vec<(u32, (Uint128, Decimal), (Uint128, Decimal))> = vec![];

    // check if active fund donation or if there a provided fund ID
    match msg.fund_id {
        // A Fund ID was provided, simple donation of all to one fund
        Some(id) => {
            let fund = FUND.load(deps.storage, &id.to_be_bytes())?;
            // check that the fund has members to donate to
            if fund.members.is_empty() {
                return Err(ContractError::IndexFundEmpty {});
            }
            // double check the given fund is valid & not expired
            if fund.is_expired(env.block.height, env.block.time) {
                return Err(ContractError::IndexFundExpired {});
            }
            let split = calculate_split(
                alliance_member,
                registrar_split_configs,
                fund.split_to_liquid,
                msg.split,
            );
            donation_messages =
                update_donation_messages(&donation_messages, fund.members, split, deposit_amount);
        }
        // Active Fund donation, check donation limits
        None => {
            match config.funding_goal {
                Some(_goal) => {
                    // loop active fund until the donation amount has been fully distributed
                    let mut loop_donation;
                    while deposit_amount > Uint128::zero() {
                        let fund = FUND.load(deps.storage, &state.active_fund.to_be_bytes())?;
                        // check that the fund has members to donate to
                        if fund.members.is_empty() {
                            return Err(ContractError::IndexFundEmpty {});
                        }
                        // double check the given fund is not expired
                        if fund.is_expired(env.block.height, env.block.time) {
                            return Err(ContractError::IndexFundExpired {});
                        }
                        // donate up to the donation goal limit to this round's active fund
                        let goal_leftover = config.funding_goal.unwrap() - state.round_donations;
                        if deposit_amount >= goal_leftover {
                            state.round_donations = Uint128::zero();
                            // set state active fund to next fund for next loop iteration
                            state.active_fund = rotate_fund(
                                read_funds(deps.storage, None, None).unwrap(),
                                state.active_fund,
                                env.block.height,
                                env.block.time,
                            );
                            loop_donation = goal_leftover;
                        } else {
                            state.round_donations += deposit_amount;
                            loop_donation = deposit_amount;
                        };
                        let split = calculate_split(
                            alliance_member,
                            registrar_split_configs.clone(),
                            fund.split_to_liquid,
                            msg.split,
                        );

                        donation_messages = update_donation_messages(
                            &donation_messages,
                            fund.members,
                            split,
                            loop_donation,
                        );
                        // deduct donated amount in this round from total donation amt
                        deposit_amount -= loop_donation;
                    }
                }
                None => {
                    // no funding goal, dump all donated funds into current active fund
                    let fund = FUND.load(deps.storage, &state.active_fund.to_be_bytes())?;
                    // check that the fund has members to donate to
                    if fund.members.is_empty() {
                        return Err(ContractError::IndexFundEmpty {});
                    }
                    // double check the given fund is not expired
                    if fund.is_expired(env.block.height, env.block.time) {
                        return Err(ContractError::IndexFundExpired {});
                    }
                    let split = calculate_split(
                        alliance_member,
                        registrar_split_configs,
                        fund.split_to_liquid,
                        msg.split,
                    );
                    donation_messages = update_donation_messages(
                        &donation_messages,
                        fund.members,
                        split,
                        deposit_amount,
                    );
                }
            };
        }
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_submessages(build_donation_messages(
            deps.as_ref(),
            registrar_config_ext.accounts_contract.unwrap(),
            donation_messages,
            deposit_fund.info,
        ))
        .add_attribute("action", "deposit"))
}

pub fn calculate_split(
    tca: bool,
    registrar_split: SplitDetails,
    fund_split: Option<Decimal>,
    user_split: Option<Decimal>,
) -> Decimal {
    // calculate the split to use
    let mut split = Decimal::zero(); // start with TCA member split (0% to liquid)

    // if the fund has a specific split amount set this overrides all other splits
    match fund_split {
        Some(s) => split = s,
        None => {
            if !tca {
                // if the user has provided a split, check it against the SC level configs
                match user_split {
                    Some(us) => {
                        if us > registrar_split.min && us < registrar_split.max {
                            split = us;
                        }
                    }
                    None => {
                        // use the SC default split
                        split = registrar_split.default;
                    }
                }
            }
        }
    }
    split
}

pub fn build_donation_messages(
    _deps: Deps,
    accounts_contract: String,
    donation_messages: Vec<(u32, (Uint128, Decimal), (Uint128, Decimal))>,
    deposit_fund_info: AssetInfoBase<Addr>,
) -> Vec<SubMsg> {
    let mut messages = vec![];
    for member in donation_messages.iter() {
        match deposit_fund_info {
            AssetInfoBase::Native(ref denom) => {
                messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: accounts_contract.clone(),
                    msg: to_binary(&angel_core::msgs::accounts::ExecuteMsg::Deposit(
                        angel_core::msgs::accounts::DepositMsg {
                            id: member.0,
                            locked_percentage: member.1 .1,
                            liquid_percentage: member.2 .1,
                        },
                    ))
                    .unwrap(),
                    funds: vec![Coin {
                        denom: denom.to_string(),
                        amount: member.1 .0 + member.2 .0,
                    }],
                })));
            }
            AssetInfoBase::Cw20(ref contract_addr) => {
                messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                        contract: accounts_contract.clone(),
                        amount: member.1 .0 + member.2 .0,
                        msg: to_binary(&angel_core::msgs::accounts::DepositMsg {
                            id: member.0,
                            locked_percentage: member.1 .1,
                            liquid_percentage: member.2 .1,
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                })));
            }
            _ => unreachable!(),
        }
    }
    messages
}

pub fn update_donation_messages(
    donation_messages: &[(u32, (Uint128, Decimal), (Uint128, Decimal))],
    members: Vec<u32>,
    split: Decimal,
    balance: Uint128,
) -> Vec<(u32, (Uint128, Decimal), (Uint128, Decimal))> {
    // set split percentages between locked & liquid accounts
    let member_portion = balance
        .checked_div(Uint128::from(members.len() as u128))
        .unwrap();
    let lock_split = Decimal::one() - split;
    let mut donation_messages = donation_messages.to_owned();

    for member in members.iter() {
        let pos = donation_messages
            .clone()
            .into_iter()
            .position(|msg| &msg.0 == member);

        if let Some(pos) = pos {
            // member addr already exists in the messages vec. Update values.
            let mut msg_data = donation_messages[pos];
            msg_data.1 .0 += member_portion * lock_split;
            msg_data.1 .1 = lock_split;
            msg_data.2 .0 += member_portion * split;
            msg_data.2 .1 = split;
            donation_messages[pos] = msg_data;
        } else {
            // add new entry for the member
            donation_messages.push((
                *member, // Addr
                (member_portion * lock_split, lock_split),
                (member_portion * split, split),
            ));
        }
    }
    donation_messages
}

pub fn rotate_fund(
    funds: Vec<IndexFund>,
    curr_fund: u64,
    env_height: u64,
    env_time: Timestamp,
) -> u64 {
    let active_funds: Vec<IndexFund> = funds
        .into_iter()
        .filter(|fund| !fund.is_expired(env_height, env_time) && fund.rotating_fund == Some(true))
        .collect();
    let curr_fund_index = active_funds.iter().position(|fund| fund.id == curr_fund);

    match curr_fund_index {
        Some(fund_index) => {
            if fund_index == (active_funds.len() - 1) {
                // go back to the start of the funds list
                active_funds[0].id
            } else {
                // get the next fund in the index
                active_funds[fund_index + 1].id
            }
        }
        None => {
            let filter_funds: Vec<IndexFund> = active_funds
                .clone()
                .into_iter()
                .filter(|fund| fund.id > curr_fund)
                .collect();
            if !filter_funds.is_empty() {
                filter_funds[0].id
            } else {
                active_funds[0].id
            }
        }
    }
}
