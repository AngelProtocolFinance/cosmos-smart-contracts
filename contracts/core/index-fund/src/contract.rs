use crate::state::{fund_read, fund_store, read_funds, Config, CONFIG, CURRENT_DONATIONS};
use angel_core::error::ContractError;
use angel_core::index_fund_msg::*;
use angel_core::index_fund_rsp::*;
use angel_core::structs::{IndexFund, SplitDetails};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw20::{Balance, Cw20Coin};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Default placeholders used in config to check compiling. Should take from InistantiateMsg.
    let configs = Config {
        owner: info.sender.clone(),
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        terra_alliance: msg.terra_alliance.unwrap_or(vec![]),
        active_fund: deps
            .api
            .addr_validate(&msg.active_fund.unwrap_or(info.sender.to_string()))?,
        fund_rotation_limit: msg
            .fund_rotation_limit
            .unwrap_or(Uint128::from(500000 as u128)), // blocks
        fund_member_limit: msg.fund_member_limit.unwrap_or(10),
        funding_goal: msg.funding_goal.unwrap_or(Some(Balance::default())),
        split_to_liquid: msg.split_to_liquid.unwrap_or(SplitDetails::default()),
    };
    CONFIG.save(deps.storage, &configs)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { new_owner } => execute_update_owner(deps, info, new_owner),
        ExecuteMsg::CreateFund { fund } => execute_create_index_fund(deps, info, fund),
        ExecuteMsg::RemoveFund(msg) => execute_remove_index_fund(deps, info, msg.fund_addr),
        ExecuteMsg::RemoveMember(msg) => execute_remove_member(deps, info, msg.member),
        ExecuteMsg::UpdateMembers(msg) => execute_update_fund_members(deps, info, msg),
    }
}

fn execute_update_owner(
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

fn execute_create_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    fund: IndexFund,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // Add the new Fund
    fund_store(deps.storage).save(&fund.address.as_bytes(), &fund)?;

    Ok(Response::default())
}

fn execute_remove_index_fund(
    deps: DepsMut,
    info: MessageInfo,
    fund_addr: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let addr = deps.api.addr_validate(&fund_addr)?;
    // this will fail if fund ID passed is not found
    let _fund = fund_read(deps.storage).load(&addr.as_bytes())?;
    // remove the fund from FUNDS
    fund_store(deps.storage).remove(&addr.as_bytes());

    Ok(Response::default())
}

fn execute_update_fund_members(
    deps: DepsMut,
    info: MessageInfo,
    msg: UpdateMembersMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let addr = deps.api.addr_validate(&msg.fund_addr)?;
    // this will fail if fund ID passed is not found
    let mut fund = fund_read(deps.storage).load(&addr.as_bytes())?;

    // add members to the fund, only if they do not already exist
    for add in msg.add.into_iter() {
        let add_addr = deps.api.addr_validate(&add)?;
        let pos = fund.members.iter().position(|m| *m == add_addr);
        // ignore if that member was found in the list
        if pos != None {
            fund.members.push(add_addr);
        }
    }

    // remove the members to the fund
    for remove in msg.remove.into_iter() {
        let remove_addr = deps.api.addr_validate(&remove)?;
        // ignore if no member is found
        if let Some(pos) = fund.members.iter().position(|m| *m == remove_addr) {
            fund.members.swap_remove(pos);
        }
    }

    // save revised fund to storage
    fund_store(deps.storage).save(&addr.as_bytes(), &fund)?;

    Ok(Response::default())
}

fn execute_remove_member(
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
            fund_store(deps.storage).save(&fund.address.as_bytes(), &fund)?;
        }
    }
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ConfigDetails {} => to_binary(&query_config(deps)?),
        QueryMsg::FundsList {} => to_binary(&query_funds_list(deps)?),
        QueryMsg::FundDetails { fund_addr } => to_binary(&query_fund_details(deps, fund_addr)?),
        QueryMsg::ActiveFundDetails {} => to_binary(&query_active_fund_details(deps)?),
        QueryMsg::ActiveFundDonations {} => to_binary(&query_active_fund_donations(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        active_fund: config.active_fund.to_string(),
        fund_rotation_limit: config.fund_rotation_limit,
        fund_member_limit: config.fund_member_limit,
        funding_goal: config.funding_goal.unwrap(),
        split_to_liquid: config.split_to_liquid,
    })
}

fn query_funds_list(deps: Deps) -> StdResult<FundListResponse> {
    // Return a list of Index Funds
    let funds = read_funds(deps.storage)?;
    Ok(FundListResponse { funds: funds })
}

fn query_fund_details(deps: Deps, fund_addr: String) -> StdResult<FundDetailsResponse> {
    let addr = deps.api.addr_validate(&fund_addr)?;
    Ok(FundDetailsResponse {
        fund: fund_read(deps.storage).may_load(&addr.as_bytes())?,
    })
}

fn query_active_fund_details(deps: Deps) -> StdResult<FundDetailsResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(FundDetailsResponse {
        fund: fund_read(deps.storage).may_load(&config.active_fund.as_bytes())?,
    })
}

fn query_active_fund_donations(deps: Deps) -> StdResult<DonationListResponse> {
    let config = CONFIG.load(deps.storage)?;
    let mut donors = vec![];
    for tca in config.terra_alliance.into_iter() {
        let donations = CURRENT_DONATIONS.may_load(deps.storage, tca.to_string())?;
        if donations != None {
            let cw20_bal: StdResult<Vec<_>> = donations
                .unwrap()
                .cw20
                .into_iter()
                .map(|token| {
                    Ok(Cw20Coin {
                        address: token.address.into(),
                        amount: token.amount,
                    })
                })
                .collect();
            // add to response vector
            donors.push(DonationDetailResponse {
                address: tca.to_string(),
                tokens: cw20_bal?,
            });
        }
    }
    Ok(DonationListResponse { donors: donors })
}

#[entry_point]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
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
            registrar_contract: String::from("some-registrar-sc"),
            terra_alliance: Some(vec![]),
            active_fund: Some("active_index_fund_addr".to_string()),
            fund_rotation_limit: Some(Uint128::from(500000u128)),
            fund_member_limit: Some(10),
            funding_goal: None,
            split_to_liquid: None,
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
