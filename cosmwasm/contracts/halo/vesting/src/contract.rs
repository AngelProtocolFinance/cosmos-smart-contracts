use crate::state::{
    read_config, read_vesting_info, read_vesting_infos, store_config, store_vesting_info, Config,
};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use halo_token::common::OrderBy;
use halo_token::vesting::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, VestingAccount,
    VestingAccountResponse, VestingAccountsResponse, VestingInfo,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            halo_token: deps.api.addr_validate(&msg.halo_token)?,
            genesis_time: env.block.height,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Claim {} => claim(deps, env, info),
        _ => {
            let config = read_config(deps.storage)?;
            if info.sender.ne(&config.owner) {
                return Err(StdError::generic_err("unauthorized"));
            }
            match msg {
                ExecuteMsg::UpdateConfig { owner, halo_token } => {
                    update_config(deps, info, owner, halo_token)
                }
                ExecuteMsg::RegisterVestingAccounts { vesting_accounts } => {
                    register_vesting_accounts(deps, info, vesting_accounts)
                }
                ExecuteMsg::AddSchedulesToVestingAccount {
                    address,
                    new_schedules,
                } => add_schedules_to_vesting_account(deps, address, new_schedules),
                ExecuteMsg::UpdateVestingAccount { vesting_account } => {
                    update_vesting_account(deps, info, vesting_account)
                }
                _ => panic!("DO NOT ENTER HERE"),
            }
        }
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    halo_token: Option<String>,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;
    if info.sender.ne(&config.owner) {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }
    if let Some(halo_token) = halo_token {
        config.halo_token = deps.api.addr_validate(&halo_token)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

fn assert_vesting_schedules(vesting_schedules: &[(u64, u64, Uint128)]) -> StdResult<()> {
    for vesting_schedule in vesting_schedules.iter() {
        if vesting_schedule.0 >= vesting_schedule.1 {
            return Err(StdError::generic_err(
                "end_time must greater than start_time",
            ));
        }
    }

    Ok(())
}

pub fn register_vesting_accounts(
    deps: DepsMut,
    info: MessageInfo,
    vesting_accounts: Vec<VestingAccount>,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    if info.sender.ne(&config.owner) {
        return Err(StdError::generic_err("unauthorized"));
    }

    for vesting_account in vesting_accounts.iter() {
        assert_vesting_schedules(&vesting_account.schedules)?;

        let vesting_address = deps.api.addr_validate(&vesting_account.address)?;
        match read_vesting_info(deps.storage, &vesting_address) {
            Ok(mut vesting_info) => {
                vesting_info.schedules = vesting_account.schedules.clone();
                store_vesting_info(deps.storage, &vesting_address, &vesting_info)?;
            }
            _ => {
                store_vesting_info(
                    deps.storage,
                    &vesting_address,
                    &VestingInfo {
                        last_claim_time: config.genesis_time,
                        schedules: vesting_account.schedules.clone(),
                    },
                )?;
            }
        }
    }

    Ok(Response::new().add_attributes(vec![("action", "register_vesting_accounts")]))
}

pub fn add_schedules_to_vesting_account(
    deps: DepsMut,
    address: String,
    new_schedules: Vec<(u64, u64, Uint128)>,
) -> StdResult<Response> {
    let addr = deps.api.addr_validate(&address).unwrap();

    if let Ok(mut vesting_info) = read_vesting_info(deps.storage, &addr) {
        vesting_info.schedules.append(&mut new_schedules.clone());
        store_vesting_info(deps.storage, &addr, &vesting_info)?;
    }

    Ok(Response::new().add_attributes(vec![("action", "add_schedules_to_vesting_account")]))
}

pub fn update_vesting_account(
    deps: DepsMut,
    info: MessageInfo,
    vesting_account: VestingAccount,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    if info.sender.ne(&config.owner) {
        return Err(StdError::generic_err("unauthorized"));
    }

    let addr = deps.api.addr_validate(&vesting_account.address).unwrap();
    match read_vesting_info(deps.storage, &addr) {
        Ok(mut vesting_info) => {
            vesting_info.schedules = vesting_account.schedules;
            store_vesting_info(deps.storage, &addr, &vesting_info)?;
        }
        _ => {
            let config: Config = read_config(deps.storage)?;
            store_vesting_info(
                deps.storage,
                &addr,
                &VestingInfo {
                    last_claim_time: config.genesis_time,
                    schedules: vesting_account.schedules,
                },
            )?;
        }
    }

    Ok(Response::new().add_attributes(vec![("action", "update_vesting_account")]))
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let current_time = env.block.time.nanos() / 1_000_000_000;
    let address = info.sender;

    let config: Config = read_config(deps.storage)?;
    let mut vesting_info: VestingInfo = read_vesting_info(deps.storage, &address)?;

    let claim_amount = compute_claim_amount(current_time, &vesting_info);
    let messages: Vec<CosmosMsg> = if claim_amount.is_zero() {
        vec![]
    } else {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.halo_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: address.to_string(),
                amount: claim_amount,
            })?,
        })]
    };

    vesting_info.last_claim_time = current_time;
    store_vesting_info(deps.storage, &address, &vesting_info)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "claim"),
        ("address", address.as_str()),
        ("claim_amount", claim_amount.to_string().as_str()),
        ("last_claim_time", current_time.to_string().as_str()),
    ]))
}

fn compute_claim_amount(current_time: u64, vesting_info: &VestingInfo) -> Uint128 {
    let mut claimable_amount: Uint128 = Uint128::zero();
    for s in vesting_info.schedules.iter() {
        if s.0 > current_time || s.1 < vesting_info.last_claim_time {
            continue;
        }

        // min(s.1, current_time) - max(s.0, last_claim_time)
        let passed_time =
            std::cmp::min(s.1, current_time) - std::cmp::max(s.0, vesting_info.last_claim_time);

        // prevent zero time_period case
        let time_period = s.1 - s.0;
        let release_amount_per_time: Decimal = Decimal::from_ratio(s.2, time_period);

        claimable_amount += Uint128::from(passed_time as u128) * release_amount_per_time;
    }

    claimable_amount
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::VestingAccount { address } => {
            Ok(to_binary(&query_vesting_account(deps, address)?)?)
        }
        QueryMsg::VestingAccounts {
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_vesting_accounts(
            deps,
            start_after,
            limit,
            order_by,
        )?)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: state.owner.to_string(),
        halo_token: state.halo_token.to_string(),
        genesis_time: state.genesis_time,
    };

    Ok(resp)
}

pub fn query_vesting_account(deps: Deps, address: String) -> StdResult<VestingAccountResponse> {
    let info = read_vesting_info(deps.storage, &deps.api.addr_validate(&address)?)?;
    let resp = VestingAccountResponse { address, info };

    Ok(resp)
}

pub fn query_vesting_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<VestingAccountsResponse> {
    let vesting_infos = if let Some(start_after) = start_after {
        read_vesting_infos(
            deps,
            deps.storage,
            Some(deps.api.addr_validate(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_vesting_infos(deps, deps.storage, None, limit, order_by)?
    };

    let vesting_account_responses: StdResult<Vec<VestingAccountResponse>> = vesting_infos
        .iter()
        .map(|vesting_account| {
            Ok(VestingAccountResponse {
                address: vesting_account.0.to_string(),
                info: vesting_account.1.clone(),
            })
        })
        .collect();

    Ok(VestingAccountsResponse {
        vesting_accounts: vesting_account_responses?,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[test]
fn test_assert_vesting_schedules() {
    // valid
    assert_vesting_schedules(&[
        (100u64, 101u64, Uint128::from(100u128)),
        (100u64, 110u64, Uint128::from(100u128)),
        (100u64, 200u64, Uint128::from(100u128)),
    ])
    .unwrap();

    // invalid
    let res = assert_vesting_schedules(&[
        (100u64, 100u64, Uint128::from(100u128)),
        (100u64, 110u64, Uint128::from(100u128)),
        (100u64, 200u64, Uint128::from(100u128)),
    ]);
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "end_time must greater than start_time")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }
}
