use crate::state::{fund_read, fund_store, read_funds, CONFIG, STATE, TCA_DONATIONS};
use angel_core::errors::core::ContractError;
use angel_core::messages::index_fund::*;
use angel_core::structs::{AcceptedTokens, IndexFund, SplitDetails};
use angel_core::utils::deduct_tax;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Balance, Cw20ReceiveMsg};

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update their address in the configs
    if info.sender != config.owner {
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

pub fn update_tca_list(
    deps: DepsMut,
    info: MessageInfo,
    new_list: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // only the owner/admin of the contract can update the TCA Members List
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let mut tca_list = vec![];
    for member in new_list.iter() {
        tca_list.push(deps.api.addr_validate(member)?);
    }

    // update config attributes with newly passed list
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.terra_alliance = tca_list;
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // only the SC owner can update these configs
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.fund_rotation = msg.fund_rotation.unwrap_or(config.fund_rotation);
    config.fund_member_limit = msg.fund_member_limit.unwrap_or(config.fund_member_limit);
    config.funding_goal = msg.funding_goal; // only config set as optional, don't unwrap
    config.split_to_liquid = SplitDetails {
        max: msg.split_max.unwrap_or(config.split_to_liquid.max),
        min: msg.split_min.unwrap_or(config.split_to_liquid.min),
        default: msg.split_default.unwrap_or(config.split_to_liquid.default),
    };
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
    fund: IndexFund,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // check that a fund does not already exists at the provided ID
    let exists = fund_read(deps.storage).may_load(&fund.id.to_be_bytes())?;
    match exists {
        Some(_) => Err(ContractError::IndexFundAlreadyExists {}),
        None => {
            // check if this is the first fund being added in...
            if read_funds(deps.storage)?.is_empty() {
                // increment state funds totals AND set the active fund ID
                STATE.update(deps.storage, |mut state| -> StdResult<_> {
                    state.total_funds += 1;
                    state.active_fund = fund.id;
                    Ok(state)
                })?;
            } else {
                // increment state funds totals
                STATE.update(deps.storage, |mut state| -> StdResult<_> {
                    state.total_funds += 1;
                    Ok(state)
                })?;
            }
            // Add the new Fund to storage
            fund_store(deps.storage).save(&fund.id.to_be_bytes(), &fund)?;

            Ok(Response::default())
        }
    }
}

pub fn remove_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    fund_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // this will fail if fund ID passed is not found
    let _fund = fund_read(deps.storage).load(&fund_id.to_be_bytes())?;
    // remove the fund from FUNDS
    fund_store(deps.storage).remove(&fund_id.to_be_bytes());
    // decrement state funds totals
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.total_funds -= 1;
        Ok(state)
    })?;
    Ok(Response::default())
}

pub fn update_fund_members(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateMembersMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // this will fail if fund ID passed is not found
    let mut fund = fund_store(deps.storage).load(&msg.fund_id.to_be_bytes())?;

    // add members to the fund, only if they do not already exist
    for add in msg.add.into_iter() {
        let add_addr = deps.api.addr_validate(&add)?;
        let pos = fund.members.iter().position(|m| *m == add_addr);
        // ignore if that member was found in the list
        if pos == None {
            fund.members.push(add_addr);
        }
    }

    // remove the members from the fund
    for remove in msg.remove.into_iter() {
        let remove_addr = deps.api.addr_validate(&remove)?;
        // ignore if no member is found
        if let Some(pos) = fund.members.iter().position(|m| *m == remove_addr) {
            fund.members.swap_remove(pos);
        }
    }

    // save revised fund to storage
    fund_store(deps.storage).save(&msg.fund_id.to_be_bytes(), &fund)?;

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
    let funds = read_funds(deps.storage)?;
    for mut fund in funds.into_iter() {
        // ignore if no member is found
        if let Some(pos) = fund.members.iter().position(|m| *m == member_addr) {
            fund.members.swap_remove(pos);
            fund_store(deps.storage).save(&fund.id.to_be_bytes(), &fund)?;
        }
    }
    Ok(Response::default())
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // check that the sending token contract is an Approved Token
    if !config.accepted_tokens.cw20_valid(info.sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }
    if cw20_msg.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }
    let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
    let msg = from_binary(&cw20_msg.msg)?;
    match msg {
        ReceiveMsg::Deposit(msg) => deposit(deps, env, info, sender_addr, msg),
    }
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

    // check each of the currenly allowed TCA member addr
    let mut tca_member = false;
    for tca in state.terra_alliance.iter() {
        if tca == &sender_addr {
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
    }
    // FOR MVP ONLY:
    // if the sender address is not among them raise err
    if !tca_member {
        return Err(ContractError::Unauthorized {});
    }

    // check if block height limit is exceeded
    let curr_active_fund = match env.block.height >= state.next_rotation_block {
        true => {
            // update STATE with new active fund & reset round donations
            let new_fund_id = rotate_fund(read_funds(deps.storage).unwrap(), state.active_fund);
            state.active_fund = new_fund_id;
            state.round_donations = Uint128::zero();
            // increment next block rotation point until it exceeds the current block height
            while env.block.height >= state.next_rotation_block {
                state.next_rotation_block += config.fund_rotation;
            }
            new_fund_id
        }
        false => state.active_fund,
    };

    let mut donation_messages = vec![];

    // check if active fund donation or if there a provided fund ID
    match msg.fund_id {
        // A Fund ID was provided, simple donation of all to one fund
        Some(fund_id) => {
            let fund = fund_read(deps.storage).load(&fund_id.to_be_bytes())?;
            let split = calculate_split(
                tca_member,
                config.split_to_liquid,
                fund.split_to_liquid,
                msg.split,
            );
            donation_messages.append(&mut build_donation_messages(
                deps.as_ref(),
                fund.members,
                split,
                "uusd".to_string(),
                deposit_amount,
            ));
        }
        // Active Fund donation, check donation limits
        None => {
            let new_active_fund = match config.funding_goal {
                Some(goal) => {
                    // check if donations limit would be exceeded by current donation amt
                    match state.round_donations + deposit_amount > goal {
                        true => {
                            let new_fund_id =
                                rotate_fund(read_funds(deps.storage).unwrap(), state.active_fund);
                            state.active_fund = new_fund_id;
                            new_fund_id
                        }
                        false => curr_active_fund,
                    }
                }
                None => curr_active_fund,
            };

            if new_active_fund != curr_active_fund && config.funding_goal != None {
                // donate up to the donation limit on old active fund
                let goal_leftover = config.funding_goal.unwrap() - state.round_donations;
                deposit_amount -= goal_leftover;
                let fund = fund_read(deps.storage).load(&curr_active_fund.to_be_bytes())?;
                let split = calculate_split(
                    tca_member,
                    config.split_to_liquid.clone(),
                    fund.split_to_liquid,
                    msg.split,
                );
                donation_messages.append(&mut build_donation_messages(
                    deps.as_ref(),
                    fund.members,
                    split,
                    "uusd".to_string(),
                    goal_leftover,
                ));
            }
            // donate the left over deposit_amount to the new active fund
            let fund = fund_read(deps.storage).load(&new_active_fund.to_be_bytes())?;
            let split = calculate_split(
                tca_member,
                config.split_to_liquid,
                fund.split_to_liquid,
                msg.split,
            );
            donation_messages.append(&mut build_donation_messages(
                deps.as_ref(),
                fund.members,
                split,
                "uusd".to_string(),
                deposit_amount,
            ));
        }
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_submessages(donation_messages)
        .add_attribute("action", "deposit"))
}

pub fn calculate_split(
    tca: bool,
    sc_split: SplitDetails,
    fund_split: Option<Decimal>,
    user_split: Option<Decimal>,
) -> Decimal {
    // calculate the split to use
    let mut split = Decimal::zero(); // start with TCA member split (100% to locked)

    // if the fund has a specific split amount set this overrides all other splits
    match fund_split {
        Some(s) => split = s,
        None => {
            if !tca {
                // if the user has provided a split, check it against the SC level configs
                match user_split {
                    Some(us) => {
                        if us > sc_split.min && us < sc_split.max {
                            split = us;
                        }
                    }
                    None => {
                        // use the SC default split
                        split = sc_split.default;
                    }
                }
            }
        }
    }
    split
}

pub fn build_donation_messages(
    deps: Deps,
    members: Vec<Addr>,
    split: Decimal,
    token_denom: String,
    balance: Uint128,
) -> Vec<SubMsg> {
    // set split percentages between locked & liquid accounts
    let member_portion = balance
        .checked_div(Uint128::from(members.len() as u128))
        .unwrap();
    let mut messages = vec![];
    for member in members.iter() {
        messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: member.to_string(),
            msg: to_binary(&angel_core::messages::accounts::ExecuteMsg::Deposit(
                angel_core::messages::accounts::DepositMsg {
                    locked_percentage: Decimal::one() - split.clone(),
                    liquid_percentage: split,
                },
            ))
            .unwrap(),
            funds: vec![deduct_tax(
                deps,
                Coin {
                    denom: token_denom.clone(),
                    amount: member_portion.clone(),
                },
            )
            .unwrap()],
        })));
    }
    messages
}

pub fn rotate_fund(funds: Vec<IndexFund>, curr_fund: u64) -> u64 {
    let new_fund;
    let curr_fund_index = funds.iter().position(|fund| fund.id == curr_fund).unwrap();
    if funds.len() < curr_fund_index + 1usize {
        // get the next fund in the index
        new_fund = funds[curr_fund_index + 1usize].id;
    } else {
        // go back to the start of the funds list
        new_fund = funds[0usize].id;
    }
    // return the fund ID
    new_fund
}
