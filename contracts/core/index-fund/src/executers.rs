use crate::state::{
    fund_read, fund_store, read_funds, ALLIANCE_MEMBERS, CONFIG, STATE, TCA_DONATIONS,
};
use angel_core::errors::core::ContractError;
use angel_core::messages::index_fund::*;
use angel_core::messages::registrar::QueryMsg as RegistrarQuerier;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::{AcceptedTokens, AllianceMember, IndexFund, SplitDetails};
use angel_core::utils::{deduct_tax, percentage_checks};
use cosmwasm_std::{
    attr, to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg, WasmQuery,
};
use cw20::Balance;

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
    if info.sender != config.registrar_contract {
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
    member: AllianceMember,
    action: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update the Alliance Members List
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // validate the member address
    let member_addr = deps.api.addr_validate(address.as_str())?;

    if action == "add".to_string() {
        ALLIANCE_MEMBERS.update(
            deps.storage,
            member_addr.clone(),
            |m: Option<AllianceMember>| -> Result<AllianceMember, ContractError> {
                match m {
                    _ => Ok(AllianceMember {
                        name: member.name,
                        logo: member.logo,
                        website: member.website,
                    }),
                }
            },
        )?;
    } else if action == "remove".to_string() {
        ALLIANCE_MEMBERS.remove(deps.storage, member_addr.clone());
    } else {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Invalid action".to_string(),
        }));
    }

    Ok(Response::new().add_attributes(vec![
        attr("method", "update_alliance_list"),
        attr("action", action),
        attr("address", member_addr.clone()),
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
    config.accepted_tokens = AcceptedTokens {
        native: msg
            .accepted_tokens_native
            .unwrap_or(config.accepted_tokens.native),
        cw20: msg
            .accepted_tokens_cw20
            .unwrap_or(config.accepted_tokens.cw20),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn create_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    description: String,
    members: Vec<String>,
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

    // check all members addresses passed are valid
    let validated_members: Vec<Addr> = members
        .iter()
        .map(|addr| deps.api.addr_validate(&addr).unwrap())
        .collect();

    let optional_split = match split_to_liquid {
        Some(split) => Some(percentage_checks(split).unwrap()),
        None => None,
    };

    // build fund struct from msg params
    let fund = IndexFund {
        id: state.next_fund_id,
        name,
        description,
        members: validated_members,
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
    fund_store(deps.storage).save(&fund.id.to_be_bytes(), &fund)?;

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
    fund_store(deps.storage).remove(&fund_id.to_be_bytes());

    Ok(Response::default())
}

pub fn update_fund_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fund_id: u64,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }
    // this will fail if fund ID passed is not found
    let mut fund = fund_store(deps.storage).load(&fund_id.to_be_bytes())?;

    if fund.is_expired(env.block.height, env.block.time) {
        return Err(ContractError::IndexFundExpired {});
    }

    // add members to the fund, only if they do not already exist
    for add in add.into_iter() {
        let add_addr = deps.api.addr_validate(&add)?;
        let pos = fund.members.iter().position(|m| *m == add_addr);
        // ignore if that member was found in the list
        if pos == None {
            fund.members.push(add_addr);
        }
    }

    // remove the members from the fund
    for remove in remove.into_iter() {
        let remove_addr = deps.api.addr_validate(&remove)?;
        // ignore if no member is found
        if let Some(pos) = fund.members.iter().position(|m| *m == remove_addr) {
            fund.members.swap_remove(pos);
        }
    }

    // check that the final number of fund members is still under the upper limit
    if fund.members.len() as u32 > config.fund_member_limit {
        return Err(ContractError::IndexFundMembershipExceeded {});
    }

    // save revised fund to storage
    fund_store(deps.storage).save(&fund_id.to_be_bytes(), &fund)?;

    Ok(Response::default())
}

pub fn remove_member(
    deps: DepsMut,
    info: MessageInfo,
    member: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // check the string is proper addr
    let member_addr = deps.api.addr_validate(&member)?;

    // Check all Funds for the given member and remove the member if found
    let funds = read_funds(deps.storage, None, None)?;
    for mut fund in funds.into_iter() {
        fund.members.retain(|m| m != &member_addr);
        fund_store(deps.storage).save(&fund.id.to_be_bytes(), &fund)?;
    }
    Ok(Response::default())
}

// pub fn receive(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     cw20_msg: Cw20ReceiveMsg,
// ) -> Result<Response, ContractError> {
//     let config = CONFIG.load(deps.storage)?;
//     // check that the sending token contract is an Approved Token
//     if !config.accepted_tokens.cw20_valid(info.sender.to_string()) {
//         return Err(ContractError::Unauthorized {});
//     }
//     if cw20_msg.amount.is_zero() {
//         return Err(ContractError::EmptyBalance {});
//     }
//     let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
//     let msg = from_binary(&cw20_msg.msg)?;
//     match msg {
//         ReceiveMsg::Deposit(msg) => deposit(deps, env, info, sender_addr, msg),
//     }
// }

pub fn update_alliance_member(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
    member: AllianceMember,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update the Alliance Members
    if info.sender.ne(&config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // check the string is proper addr
    let member_addr = deps.api.addr_validate(address.as_str())?;

    // Update the corresponding Alliance Members.
    ALLIANCE_MEMBERS.update(
        deps.storage,
        member_addr.clone(),
        |m: Option<AllianceMember>| -> Result<AllianceMember, ContractError> {
            match m {
                _ => Ok(AllianceMember {
                    name: member.name,
                    logo: member.logo,
                    website: member.website,
                }),
            }
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("method", "update_alliance_member"),
        attr("member_addr", member_addr.to_string()),
    ]))
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_addr: Addr,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // only accept max of 1 deposit coin/token per donation
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidCoinsDeposited {});
    }

    let mut deposit_amount: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == *"uusd")
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // check each of the currenly allowed Alliance member addr
    let mut tca_member = false;
    let is_sender_alliance_member = ALLIANCE_MEMBERS.has(deps.storage, sender_addr.clone());
    if is_sender_alliance_member {
        tca_member = true;
        // note increased donation amount for the TCA member
        let mut tca_donor = TCA_DONATIONS
            .may_load(deps.storage, sender_addr.to_string())?
            .unwrap_or_default();
        tca_donor.add_tokens(Balance::from(vec![Coin {
            denom: "uusd".to_string(),
            amount: deposit_amount,
        }]));
        TCA_DONATIONS.save(deps.storage, sender_addr.to_string(), &tca_donor)?;
    }

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

    // Get the Registrar SC Split to liquid parameters
    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQuerier::Config {})?,
        }))?;
    let registrar_split_configs: SplitDetails = registrar_config.split_to_liquid;

    let mut donation_messages: Vec<(Addr, (Uint128, Decimal), (Uint128, Decimal))> = vec![];

    // check if active fund donation or if there a provided fund ID
    match msg.fund_id {
        // A Fund ID was provided, simple donation of all to one fund
        Some(id) => {
            let fund = fund_read(deps.storage).load(&id.to_be_bytes())?;
            // check that the fund has members to donate to
            if fund.members.len() == 0 {
                return Err(ContractError::IndexFundEmpty {});
            }
            // double check the given fund is valid & not expired
            if fund.is_expired(env.block.height, env.block.time) {
                return Err(ContractError::IndexFundExpired {});
            }
            let split = calculate_split(
                tca_member,
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
                        let fund =
                            fund_read(deps.storage).load(&state.active_fund.to_be_bytes())?;
                        // check that the fund has members to donate to
                        if fund.members.len() == 0 {
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
                            tca_member,
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
                    let fund = fund_read(deps.storage).load(&state.active_fund.to_be_bytes())?;
                    // check that the fund has members to donate to
                    if fund.members.len() == 0 {
                        return Err(ContractError::IndexFundEmpty {});
                    }
                    // double check the given fund is not expired
                    if fund.is_expired(env.block.height, env.block.time) {
                        return Err(ContractError::IndexFundExpired {});
                    }
                    let split = calculate_split(
                        tca_member,
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
        .add_submessages(build_donation_messages(deps.as_ref(), donation_messages))
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
    deps: Deps,
    donation_messages: Vec<(Addr, (Uint128, Decimal), (Uint128, Decimal))>,
) -> Vec<SubMsg> {
    let mut messages = vec![];
    for member in donation_messages.iter() {
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: member.0.to_string(),
            msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::Deposit(
                angel_core::messages::accounts::DepositMsg {
                    locked_percentage: member.1 .1,
                    liquid_percentage: member.2 .1,
                },
            ))
            .unwrap(),
            funds: vec![deduct_tax(
                deps,
                Coin {
                    denom: "uusd".to_string(),
                    amount: member.1 .0 + member.2 .0,
                },
            )
            .unwrap()],
        })));
    }
    messages
}

pub fn update_donation_messages(
    donation_messages: &[(Addr, (Uint128, Decimal), (Uint128, Decimal))],
    members: Vec<Addr>,
    split: Decimal,
    balance: Uint128,
) -> Vec<(Addr, (Uint128, Decimal), (Uint128, Decimal))> {
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

        if pos != None {
            // member addr already exists in the messages vec. Update values.
            let mut msg_data = donation_messages[pos.unwrap()].clone();
            msg_data.1 .0 += member_portion * lock_split;
            msg_data.1 .1 = lock_split;
            msg_data.2 .0 += member_portion * split;
            msg_data.2 .1 = split;
            donation_messages[pos.unwrap()] = msg_data;
        } else {
            // add new entry for the member
            donation_messages.push((
                member.clone(), // Addr
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate_funds() {
        let index_fund_1 = IndexFund {
            id: 1,
            name: "Fund #1".to_string(),
            description: "Fund number 1 test rotation".to_string(),
            members: vec![],
            split_to_liquid: None,
            expiry_time: None,
            expiry_height: None,
            rotating_fund: Some(true),
        };
        let index_fund_2 = IndexFund {
            id: 2,
            name: "Fund #2".to_string(),
            description: "Fund number 2 test rotation".to_string(),
            members: vec![],
            split_to_liquid: None,
            expiry_time: None,
            expiry_height: None,
            rotating_fund: Some(true),
        };

        let new_fund_1 = rotate_fund(
            vec![index_fund_1.clone()],
            1,
            10,
            Timestamp::from_seconds(100),
        );
        assert_eq!(new_fund_1, 1);
        let new_fund_2 = rotate_fund(
            vec![index_fund_1.clone(), index_fund_2.clone()],
            1,
            10,
            Timestamp::from_seconds(100),
        );
        assert_eq!(new_fund_2, 2);
        let new_fund_3 = rotate_fund(
            vec![index_fund_1, index_fund_2],
            2,
            10,
            Timestamp::from_seconds(100),
        );
        assert_eq!(new_fund_3, 1);
    }

    #[test]
    fn rotate_funds_with_expired_funds() {
        let index_fund_1 = IndexFund {
            id: 1,
            name: "Fund #1".to_string(),
            description: "Fund number 1 test rotation".to_string(),
            members: vec![],
            split_to_liquid: None,
            expiry_time: None,
            expiry_height: None,
            rotating_fund: Some(true),
        };
        let index_fund_2 = IndexFund {
            id: 2,
            name: "Fund #2".to_string(),
            description: "Fund number 2 test rotation".to_string(),
            members: vec![],
            split_to_liquid: None,
            expiry_time: None,
            expiry_height: Some(10),
            rotating_fund: Some(false),
        };
        let index_fund_3 = IndexFund {
            id: 3,
            name: "Fund #3".to_string(),
            description: "Fund number 3 test rotation".to_string(),
            members: vec![],
            split_to_liquid: None,
            expiry_time: Some(1000),
            expiry_height: Some(1000),
            rotating_fund: Some(true),
        };

        let new_fund_1 = rotate_fund(
            vec![index_fund_1.clone()],
            1,
            100,
            Timestamp::from_seconds(10000),
        );
        assert_eq!(new_fund_1, 1);

        let new_fund_2 = rotate_fund(
            vec![index_fund_2.clone(), index_fund_1.clone()],
            1,
            100,
            Timestamp::from_seconds(10000),
        );
        assert_eq!(new_fund_2, 1);

        let new_fund_3 = rotate_fund(
            vec![index_fund_3, index_fund_1, index_fund_2],
            1,
            100,
            Timestamp::from_seconds(10000),
        );
        assert_eq!(new_fund_3, 1);
    }

    #[test]
    fn test_tca_without_split() {
        let sc_split = SplitDetails::default();
        assert_eq!(calculate_split(true, sc_split, None, None), Decimal::zero());
    }
    #[test]
    fn test_tca_with_split() {
        let sc_split = SplitDetails::default();
        assert_eq!(
            calculate_split(true, sc_split, None, Some(Decimal::percent(42))),
            Decimal::zero()
        );
    }
    #[test]
    fn test_non_tca_with_split() {
        let sc_split = SplitDetails::default();
        assert_eq!(
            calculate_split(false, sc_split, None, Some(Decimal::percent(23))),
            Decimal::percent(23)
        );
    }
    #[test]
    fn test_non_tca_without_split() {
        let sc_split = SplitDetails::default();
        assert_eq!(
            calculate_split(false, sc_split.clone(), None, None),
            sc_split.default
        );
    }
}
