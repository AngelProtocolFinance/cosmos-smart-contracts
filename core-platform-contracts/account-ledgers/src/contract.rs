use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Api, BankMsg, Binary, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, SubMsg, WasmMsg,
};

use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, CreateAcctMsg, DetailsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg,
    QueryMsg, ReceiveMsg, UpdateConfigMsg,
};
use crate::state::{
    Account, AssetVault, Config, GenericBalance, SplitDetails, Strategy, StrategyComponent,
    ACCOUNTS, CONFIG, VAULTS,
};

// version info for future migration info
const CONTRACT_NAME: &str = "account-ledgers";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn addr_approved_coins(
    approved_coins: &Option<Vec<String>>,
    api: &dyn Api,
) -> StdResult<Vec<Addr>> {
    match approved_coins.as_ref() {
        Some(v) => v.iter().map(|h| api.addr_validate(h)).collect(),
        None => Ok(vec![]),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // convert given approved_coins to addr for storage
    let approved_coins = addr_approved_coins(&msg.cw20_approved_coins, deps.api)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            charity_endowment_sc: None,
            index_fund_sc: None,
            cw20_approved_coins: approved_coins,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateAcct(msg) => execute_create(deps, env, msg, &info.sender.clone()),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::UpdateConfig(msg) => update_config(deps, env, info, msg),
        ExecuteMsg::UpdateOwner { new_owner } => update_owner(deps, env, info, new_owner),
        ExecuteMsg::Terminate { account_id } => execute_terminate(deps, env, info, account_id),
        ExecuteMsg::Deposit { account_id } => execute_deposit(deps, info, account_id),
    }
}

pub fn update_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let new_owner = deps.api.addr_validate(&new_owner)?;
    let config = CONFIG.load(deps.storage)?;
    // only the owner of the contract can update the configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner = new_owner;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the owner of the contract can update the configs...for now
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // convert given approved_coins to addr for storage
    let approved_coins = addr_approved_coins(&msg.cw20_approved_coins, deps.api)?;

    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.cw20_approved_coins = approved_coins;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn execute_create(
    deps: DepsMut,
    _env: Env,
    msg: CreateAcctMsg,
    sender: &Addr,
) -> Result<Response, ContractError> {
    let account = Account {
        balance: GenericBalance {
            native: vec![],
            cw20: vec![],
        },
        strategy: Strategy::default(),
        split_deposit: SplitDetails::default(),
        split_interest: SplitDetails::default(),
    };

    // try to store it, fail if the account ID was already in use
    ACCOUNTS.update(deps.storage, msg.account_id, |existing| match existing {
        None => Ok(account),
        Some(_) => Err(ContractError::AlreadyInUse {}),
    })?;

    let res = Response {
        attributes: vec![attr("action", "create"), attr("id", sender.clone())],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let msg_sender = &deps.api.addr_validate(&wrapper.sender)?;
    match msg {
        ReceiveMsg::CreateAcct(msg) => execute_create(deps, env, msg, msg_sender),
        ReceiveMsg::Deposit { account_id } => execute_deposit(deps, info, account_id),
    }
}

pub fn execute_deposit(
    deps: DepsMut,
    info: MessageInfo,
    account_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // this fails if no account is there
    let mut account = ACCOUNTS.load(deps.storage, account_id.clone())?;

    // this lookup fails if the native token deposit was not coming from one of the vaults
    let vault = VAULTS.load(deps.storage, info.sender.to_string())?;

    if !vault.approved {
        return Err(ContractError::Unauthorized {});
    }

    let balance = Balance::from(info.funds);
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the approved_coins
        if !config
            .cw20_approved_coins
            .iter()
            .any(|t| t == &token.address)
        {
            return Err(ContractError::NotInApprovedCoins {});
        }
    };

    account.balance.add_tokens(balance);

    // and save
    ACCOUNTS.save(deps.storage, account_id.clone(), &account)?;

    let res = Response {
        attributes: vec![attr("action", "deposit"), attr("id", account_id)],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_terminate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    account_id: String,
) -> Result<Response, ContractError> {
    // this fails if no account is found
    let account = ACCOUNTS.load(deps.storage, account_id.clone())?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.charity_endowment_sc.unwrap() {
        Err(ContractError::Unauthorized {})
    } else {
        // we delete the account
        ACCOUNTS.remove(deps.storage, account_id.clone());

        // send all tokens out to the beneficiary
        let _messages = send_tokens(&config.owner, &account.balance)?;

        let attributes = vec![
            attr("action", "terminate"),
            attr("account_id", account_id.clone()),
            attr("to", config.owner),
        ];
        let res = Response {
            attributes: attributes,
            ..Response::default()
        };
        Ok(res)
    }
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<SubMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        // QueryMsg::List { address } => to_binary(&query_list(deps, address)?),
        QueryMsg::Details { account_id } => to_binary(&query_details(deps, account_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        charity_endowment_sc: config.clone().charity_endowment_sc.unwrap().to_string(),
        index_fund_sc: config.clone().index_fund_sc.unwrap().to_string(),
        cw20_approved_coins: config.human_approved_coins(),
    };
    Ok(res)
}

fn query_details(deps: Deps, address: String) -> StdResult<DetailsResponse> {
    // this fails if no account is found
    let account = ACCOUNTS.load(deps.storage, address)?;

    // transform tokens
    let native_balance = account.balance.native;

    let cw20_balance: StdResult<Vec<_>> = account
        .balance
        .cw20
        .into_iter()
        .map(|token| {
            Ok(Cw20Coin {
                address: token.address.into(),
                amount: token.amount,
            })
        })
        .collect();

    let details = DetailsResponse {
        native_balance,
        cw20_balance: cw20_balance?,
    };
    Ok(details)
}

// fn query_list(deps: Deps, address: Option<String>) -> StdResult<ListResponse> {
//     // TO DO: Return a list of Accounts
//     // Returns the list of addresses for all registered accounts in storage
//     // let acct_list = ACCOUNTS.keys(&deps.storage, None, None).collect()?;
//     Ok(ListResponse {
//         accounts: vec![],
//     })
// }

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
    use cosmwasm_std::{coin, coins, Uint128};

    #[test]
    fn test_proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info("creator", &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len()); // no news is good news! :)
    }

    #[test]
    fn test_get_config() {
        let mut deps = mock_dependencies(&[]);
        let agent1 = String::from("agent007");

        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies(&[]);
        let agent1 = String::from("agent007");
        let agent2 = String::from("agent008");

        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // change the expirary to true and shorten payout to 15 days
        let msg = UpdateConfigMsg {
            charity_endowment_sc: Some(String::from("charity-endowment-sc")),
            index_fund_sc: Some(String::from("index-fund-sc")),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateConfig(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // check that new configs were set
        let res = query_config(deps.as_ref()).unwrap();
        assert_eq!(1, res.cw20_approved_coins.len());

        // Not just anyone can update the configs! Only owner can.
        let msg = UpdateConfigMsg {
            charity_endowment_sc: Some(String::from("charity-endowment-sc")),
            index_fund_sc: Some(String::from("index-fund-sc")),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent2.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        // This should fail with an error!
        let err = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateConfig(msg)).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_change_contract_owner() {
        let mut deps = mock_dependencies(&[]);
        let agent1 = String::from("agent007");
        let agent2 = String::from("agent008");

        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // change the owner to Agent2
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateOwner {
                new_owner: agent2.clone(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // Agent1 should not be able to update the configs now
        let msg = UpdateConfigMsg {
            charity_endowment_sc: Some(String::from("charity-endowment-sc")),
            index_fund_sc: None,
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        // This should fail with an error!
        let err = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateConfig(msg)).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_create_account() {
        let mut deps = mock_dependencies(&[]);
        let agent1 = String::from("agent007");

        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = CreateAcctMsg {
            account_id: String::from("locked_XXCETWTETGSGSRHJTUIQAADFAG"),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_terminate_account() {
        let mut deps = mock_dependencies(&[]);
        let agent1 = String::from("agent007");
        let agent2 = String::from("agent008");

        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = CreateAcctMsg {
            account_id: String::from("locked_XXCETWTETGSGSRHJTUIQAADFAG"),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // only arbiter can terminate the account
        let info = mock_info(&agent2.clone(), &coins(100000, "earth"));
        let env = mock_env();
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Terminate {
                account_id: agent1.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // trigger account termination for real
        let info = mock_info(agent1.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Terminate {
                account_id: agent1.clone(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_balance_add_tokens_proper() {
        let mut tokens = GenericBalance::default();
        tokens.add_tokens(Balance::from(vec![coin(123, "atom"), coin(789, "eth")]));
        tokens.add_tokens(Balance::from(vec![coin(456, "atom"), coin(12, "btc")]));
        assert_eq!(
            tokens.native,
            vec![coin(579, "atom"), coin(789, "eth"), coin(12, "btc")]
        );
    }

    #[test]
    fn test_balance_add_cw_tokens_proper() {
        let mut tokens = GenericBalance::default();
        let bar_token = Addr::unchecked("bar_token");
        let foo_token = Addr::unchecked("foo_token");
        tokens.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: foo_token.clone(),
            amount: Uint128::from(12345 as u128),
        }));
        tokens.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: bar_token.clone(),
            amount: Uint128::from(777 as u128),
        }));
        tokens.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: foo_token.clone(),
            amount: Uint128::from(23400 as u128),
        }));
        assert_eq!(
            tokens.cw20,
            vec![
                Cw20CoinVerified {
                    address: foo_token,
                    amount: Uint128::from(35745 as u128),
                },
                Cw20CoinVerified {
                    address: bar_token,
                    amount: Uint128::from(777 as u128),
                }
            ]
        );
    }

    #[test]
    fn test_account_receives_native_tokens() {
        let mut deps = mock_dependencies(&[]);
        let agent1 = String::from("agent007");
        let agent3 = String::from("agent006");

        let instantiate_msg = InstantiateMsg {
            charity_endowment_sc: String::from("charity-endowment-sc"),
            index_fund_sc: String::from("index-fund-sc"),
            cw20_approved_coins: Some(vec![String::from("bar_token"), String::from("foo_token")]),
        };
        let info = mock_info(agent1.as_ref(), &coins(100000, "bar_token"));
        let env = mock_env();

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = CreateAcctMsg {
            account_id: String::from("locked_XXCETWTETGSGSRHJTUIQAADFAG"),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // try to top account up with 2 tokens
        // should NOT be able to recieve any tokens before Account is approved
        let extra_native = vec![coin(250, "bar_token"), coin(300, "foo_token")];
        let info = mock_info(&agent3.clone(), &extra_native);
        let deposit = ExecuteMsg::Deposit {
            account_id: agent1.clone(),
        };
        let err = execute(deps.as_mut(), mock_env(), info, deposit).unwrap_err();
        assert_eq!(err, ContractError::AccountNotApproved {});

        // try to top account up with 2 approved tokens
        let extra_native = vec![coin(250, "bar_token"), coin(300, "foo_token")];
        let info = mock_info(&agent3.clone(), &extra_native);
        let deposit = ExecuteMsg::Deposit {
            account_id: agent1.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, deposit).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "deposit"), res.attributes[0]);

        // try to top account up with a non-approved tokens
        let bad_coins = vec![coin(250, "rat_poison"), coin(300, "squared")];
        let info = mock_info(&agent3.clone(), &bad_coins);
        let deposit = ExecuteMsg::Deposit {
            account_id: agent1.clone(),
        };
        let err = execute(deps.as_mut(), mock_env(), info, deposit).unwrap_err();
        assert_eq!(err, ContractError::NotInApprovedCoins {});
    }
}
