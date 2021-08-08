use crate::state::{fund_read, fund_store, read_funds, CONFIG, STATE};
use angel_core::error::ContractError;
use angel_core::index_fund_msg::*;
use angel_core::structs::{IndexFund, SplitDetails};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, DepsMut, MessageInfo, ReplyOn,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ReceiveMsg;

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
        tca_list.push(deps.api.addr_validate(&member)?);
    }

    // update config attributes with newly passed list
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.terra_alliance = tca_list;
        Ok(state)
    })?;

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
        Some(_) => return Err(ContractError::IndexFundAlreadyExists {}),
        None => {
            // Add the new Fund
            fund_store(deps.storage).save(&fund.id.to_be_bytes(), &fund)?;
            // increment state funds totals
            STATE.update(deps.storage, |mut state| -> StdResult<_> {
                state.total_funds += 1;
                Ok(state)
            })?;
            return Ok(Response::default());
        }
    };
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
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // check that the sending token contract is an Approved Token
    if config.accepted_tokens.cw20_valid(info.sender.to_string()) != true {
        return Err(ContractError::Unauthorized {});
    }
    if cw20_msg.amount.is_zero() {
        return Err(ContractError::EmptyBalance {});
    }
    let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
    let msg = from_binary(&cw20_msg.msg)?;
    match msg {
        ReceiveMsg::Deposit(msg) => deposit(deps, sender_addr, cw20_msg.amount, msg),
    }
}

pub fn deposit(
    deps: DepsMut,
    sender_addr: Addr,
    balance: Uint128,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    // check each of the currenly allowed TCA member addr
    let mut tca_member = false;
    for tca in state.terra_alliance.iter() {
        if tca == &sender_addr {
            tca_member = true;
        }
    }
    // if the sender address is not among them raise err
    if tca_member != true {
        return Err(ContractError::Unauthorized {});
    }

    // set target fund tp either the active fund or provided fund ID
    let fund_id: u64 = match msg.fund_id {
        Some(fund_id) => fund_id,
        None => state.active_fund.unwrap(),
    };
    let fund = fund_read(deps.storage).load(&fund_id.to_be_bytes())?;
    let split = calculate_split(
        tca_member,
        config.split_to_liquid,
        fund.split_to_liquid,
        msg.split,
    );
    let msgs = build_donation_messages(fund.members, split, "uust".to_string(), balance);

    Ok(Response {
        messages: msgs,
        attributes: vec![attr("action", "deposit")],
        ..Response::default()
    })
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
            if tca != true {
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
    members: Vec<Addr>,
    split: Decimal,
    token_denom: String,
    balance: Uint128,
) -> Vec<SubMsg> {
    // set split percentages between locked & liquid accounts
    let locked_percentage = Decimal::one() - split;
    let liquid_percentage = split;
    let member_portion = balance.multiply_ratio(1 as u128, members.len() as u128);
    let mut messages = vec![];
    for member in members.iter() {
        // locked msg
        messages.push(donation_submsg(
            member.to_string(),
            "locked".to_string(),
            token_denom.clone(),
            member_portion * locked_percentage,
        ));
        // liquid msg needed if split is greater than zero
        if split > Decimal::zero() {
            messages.push(donation_submsg(
                member.to_string(),
                "liquid".to_string(),
                token_denom.clone(),
                member_portion * liquid_percentage,
            ));
        }
    }
    return messages;
}

pub fn donation_submsg(
    member_addr: String,
    acct_type: String,
    send_denom: String,
    send_amount: Uint128,
) -> SubMsg {
    let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: member_addr,
        msg: to_binary(&angel_core::accounts_msg::DepositMsg {
            account_type: acct_type,
        })
        .unwrap(),

        funds: vec![Coin {
            amount: send_amount,
            denom: send_denom,
        }],
    });

    SubMsg {
        id: 0,
        msg: wasm_msg,
        gas_limit: None,
        reply_on: ReplyOn::Never,
    }
}
