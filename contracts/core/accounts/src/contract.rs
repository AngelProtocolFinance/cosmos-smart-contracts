use crate::state::{Account, Config, GenericBalance, RebalanceDetails, ACCOUNTS, CONFIG};
use angel_core::accounts_msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg, UpdateEndowmentSettingsMsg,
};
use angel_core::accounts_rsp::{AccountDetailsResponse, AccountListResponse, ConfigResponse};
use angel_core::error::ContractError;
use angel_core::structs::Strategy;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, SubMsg, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            admin_addr: info.sender.clone(),
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            index_fund_contract: deps.api.addr_validate(&msg.index_fund_contract)?,
            endowment_owner: deps.api.addr_validate(&msg.endowment_owner)?, // Addr
            endowment_beneficiary: deps.api.addr_validate(&msg.endowment_beneficiary)?, // Addr
            deposit_approved: false,                                        // bool
            withdraw_approved: false,                                       // bool
            withdraw_before_maturity: msg.withdraw_before_maturity,         // bool
            maturity_time: msg.maturity_time,                               // Option<u64>
            maturity_height: msg.maturity_height,                           // Option<u64>
            split_to_liquid: msg.split_to_liquid,                           // SplitDetails
        },
    )?;

    let account = Account {
        balance: GenericBalance {
            native: vec![],
            cw20: vec![],
        },
        strategy: Strategy::default(),
        rebalance: RebalanceDetails::default(),
    };

    // try to create both prefixed accounts
    for prefix in ["locked", "liquid"].iter() {
        // try to store it, fail if the account ID was already in use
        ACCOUNTS.update(
            deps.storage,
            prefix.to_string(),
            |existing| match existing {
                None => Ok(account.clone()),
                Some(_) => Err(ContractError::AlreadyInUse {}),
            },
        )?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let balance = Balance::from(info.funds.clone());
    match msg {
        ExecuteMsg::UpdateEndowmentSettings(msg) => update_endowment_settings(deps, env, info, msg),
        ExecuteMsg::Deposit(msg) => {
            execute_deposit(deps, info.sender, balance.clone(), msg.account_type)
        }
        ExecuteMsg::VaultReceipt(msg) => {
            execute_vault_receipt(deps, info.sender, balance.clone(), msg.account_type)
        }
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateStrategy {
            account_type,
            strategy,
        } => update_strategy(deps, env, info, account_type, strategy),
        ExecuteMsg::Liquidate { beneficiary } => execute_liquidate(deps, env, info, beneficiary),
        ExecuteMsg::TerminateToFund { fund } => execute_terminate_to_fund(deps, env, info, fund),
        ExecuteMsg::TerminateToAddress { beneficiary } => {
            execute_terminate_to_address(deps, env, info, beneficiary)
        }
        ExecuteMsg::Receive(msg) => execute_receive(deps, info, msg),
    }
}

pub fn update_registrar(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_registrar: String,
) -> Result<Response, ContractError> {
    let new_registrar = deps.api.addr_validate(&new_registrar)?;
    let config = CONFIG.load(deps.storage)?;
    // only the owner of the contract can update the configs...for now
    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.registrar_contract = new_registrar;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_endowment_settings(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateEndowmentSettingsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the endowment owner can update these configs
    if info.sender != config.endowment_owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate SC address strings passed
    let endowment_beneficiary = deps.api.addr_validate(&msg.beneficiary)?;
    let endowment_owner = deps.api.addr_validate(&msg.owner)?;

    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.endowment_owner = endowment_owner;
        config.endowment_beneficiary = endowment_beneficiary;
        config.split_to_liquid = msg.split_to_liquid;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    account_type: String,
    strategy: Strategy,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.endowment_owner {
        return Err(ContractError::Unauthorized {});
    }

    // this fails if no account is there
    let mut account = ACCOUNTS.load(deps.storage, account_type.clone())?;

    // update account strategy attribute with the newly passed strategy
    account.strategy = strategy;

    // and save
    ACCOUNTS.save(deps.storage, account_type, &account)?;

    Ok(Response::default())
}

pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender.clone(),
        amount: cw20_msg.amount,
    });
    let sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
    let msg = from_binary(&cw20_msg.msg)?;
    match msg {
        ReceiveMsg::Deposit(msg) => execute_deposit(deps, sender_addr, balance, msg.account_type),
        ReceiveMsg::VaultReceipt(msg) => {
            execute_vault_receipt(deps, sender_addr, balance, msg.account_type)
        }
    }
}

pub fn execute_vault_receipt(
    deps: DepsMut,
    _sender_addr: Addr,
    balance: Balance,
    account_type: String,
) -> Result<Response, ContractError> {
    // this fails if no account is there
    let mut account = ACCOUNTS.load(deps.storage, account_type.clone())?;

    // this lookup fails if the token deposit was not coming from an Asset Vault SC
    // let _vaults = VAULTS.load(deps.storage, sender_addr.to_string())?;

    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    account.balance.add_tokens(balance);

    // and save
    ACCOUNTS.save(deps.storage, account_type.clone(), &account)?;

    let res = Response {
        attributes: vec![
            attr("action", "vault_receipt"),
            attr("account_type", account_type),
        ],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_deposit(
    deps: DepsMut,
    _sender_addr: Addr,
    balance: Balance,
    account_type: String,
) -> Result<Response, ContractError> {
    // this fails if no account is there
    let mut account = ACCOUNTS.load(deps.storage, account_type.clone())?;

    let _config = CONFIG.load(deps.storage)?;

    // this lookup fails if the token deposit was not coming from:
    // an Asset Vault SC, the Charity Endownment SC, or the Index Fund SC
    // if sender_addr != config.index_fund_contract {
    //     return Err(ContractError::Unauthorized {});
    // }

    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    // if let Balance::Cw20(token) = &balance {
    //     // ensure the token is on the approved_coins
    //     if !config.approved_coins.iter().any(|t| t == &token.address) {
    //         return Err(ContractError::NotInApprovedCoins {});
    //     }
    // };

    account.balance.add_tokens(balance);

    // and save
    ACCOUNTS.save(deps.storage, account_type, &account)?;

    // let res = Response {
    //     messages: vec![SubMsg::new(WasmMsg::Execute {
    //         contract_addr: config.index_fund_contract.to_string(),
    //         msg: Binary::from_base64(&msg_data)?,
    //         funds: vec![],
    //     })],
    //     attributes: vec![
    //         attr("action", "deposit"),
    //         attr("account_type", account_type.clone()),
    // ],
    // ..Response::default()
    // };
    // Ok(res)
    Ok(Response::default())
}

pub fn execute_liquidate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // send all tokens out to the index fund sc
        let _messages = send_tokens(&config.index_fund_contract, &account.balance)?;
    }

    let attributes = vec![attr("action", "liquidate"), attr("to", beneficiary_addr)];
    let res = Response {
        attributes: attributes,
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_terminate_to_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }

    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    let mut messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // send all tokens out to the index fund sc
        messages.append(&mut send_tokens(&beneficiary_addr, &account.balance)?);
    }

    let attributes = vec![attr("action", "terminate"), attr("to", beneficiary_addr)];
    let res = Response {
        messages: messages,
        attributes: attributes,
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_terminate_to_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fund: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.registrar_contract {
        return Err(ContractError::Unauthorized {});
    }
    let mut messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, prefix.to_string())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, prefix.to_string());
        // send all tokens out to the index fund sc
        messages.append(&mut send_tokens(
            &config.index_fund_contract,
            &account.balance,
        )?);
    }

    let attributes = vec![
        attr("action", "terminate"),
        attr("fund_id", fund),
        attr("to", config.index_fund_contract),
    ];
    let res = Response {
        messages: messages,
        attributes: attributes,
        ..Response::default()
    };
    Ok(res)
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
        QueryMsg::Account { account_type } => {
            to_binary(&query_account_details(deps, account_type)?)
        }
        QueryMsg::AccountList {} => to_binary(&query_account_list(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        admin_addr: config.registrar_contract.to_string(),
    };
    Ok(res)
}

fn query_account_details(deps: Deps, account_type: String) -> StdResult<AccountDetailsResponse> {
    // this fails if no account is found
    let account = ACCOUNTS.load(deps.storage, account_type.clone())?;

    let balance: StdResult<Vec<_>> = account
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

    let details = AccountDetailsResponse {
        account_type: account_type,
        strategy: account.strategy,
        balance: balance?,
    };
    Ok(details)
}

fn query_account_list(_deps: Deps) -> StdResult<AccountListResponse> {
    let list = AccountListResponse { accounts: vec![] };
    Ok(list)
}

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
    use angel_core::structs::SplitDetails;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Uint128};
    use cw20::Cw20CoinVerified;

    #[test]
    fn test_proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = "angelprotocolteamdano".to_string();
        let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
        let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
        let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();

        let instantiate_msg = InstantiateMsg {
            admin_addr: ap_team.clone(),
            registrar_contract: registrar_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            endowment_owner: charity_addr.clone(),
            endowment_beneficiary: charity_addr.clone(),
            withdraw_before_maturity: false,
            maturity_time: None,
            maturity_height: None,
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info("creator", &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len()); // no news is good news! :)
    }

    #[test]
    fn test_get_config() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = "angelprotocolteamdano".to_string();
        let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
        let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
        let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();

        let instantiate_msg = InstantiateMsg {
            admin_addr: ap_team.clone(),
            registrar_contract: registrar_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            endowment_owner: charity_addr.clone(),
            endowment_beneficiary: charity_addr.clone(),
            withdraw_before_maturity: false,
            maturity_time: None,
            maturity_height: None,
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = "angelprotocolteamdano".to_string();
        let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
        let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
        let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
        let pleb = "plebAccount".to_string();

        let instantiate_msg = InstantiateMsg {
            admin_addr: ap_team.clone(),
            registrar_contract: registrar_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            endowment_owner: charity_addr.clone(),
            endowment_beneficiary: charity_addr.clone(),
            withdraw_before_maturity: false,
            maturity_time: None,
            maturity_height: None,
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the endowment owner and beneficiary
        let msg = UpdateEndowmentSettingsMsg {
            owner: charity_addr.clone(),
            beneficiary: pleb.clone(),
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateEndowmentSettings(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // Not just anyone can update the Endowment's settings! Only Endowment owner can.
        let msg = UpdateEndowmentSettingsMsg {
            owner: charity_addr.clone(),
            beneficiary: pleb.clone(),
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(pleb.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        // This should fail with an error!
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateEndowmentSettings(msg),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_change_contract_owner() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = "angelprotocolteamdano".to_string();
        let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
        let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
        let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
        let pleb = "plebAccount".to_string();

        let instantiate_msg = InstantiateMsg {
            admin_addr: ap_team.clone(),
            registrar_contract: registrar_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            endowment_owner: charity_addr.clone(),
            endowment_beneficiary: charity_addr.clone(),
            withdraw_before_maturity: false,
            maturity_time: None,
            maturity_height: None,
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // change the owner to some pleb
        let info = mock_info(registrar_contract.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateRegistrar {
                new_registrar: pleb.clone(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // check changes saved and can be recalled
        let res = query_config(deps.as_ref()).unwrap();
        assert_eq!(pleb.clone(), res.admin_addr);

        // Original charity owner should not be able to update the configs now
        let msg = UpdateEndowmentSettingsMsg {
            owner: charity_addr.clone(),
            beneficiary: pleb.clone(),
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        // This should fail with an error!
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateEndowmentSettings(msg),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_terminate_account() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = "angelprotocolteamdano".to_string();
        let charity_addr = "XCEMQTWTETGSGSRHJTUIQADG".to_string();
        let index_fund_contract = "INDEXTHADFARHSRTHADGG".to_string();
        let registrar_contract = "REGISTRARGSDRGSDRGSDRGFG".to_string();
        let pleb = "plebAccount".to_string();

        let instantiate_msg = InstantiateMsg {
            admin_addr: ap_team.clone(),
            registrar_contract: registrar_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            endowment_owner: charity_addr.clone(),
            endowment_beneficiary: charity_addr.clone(),
            withdraw_before_maturity: false,
            maturity_time: None,
            maturity_height: None,
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the endowment owner and beneficiary
        let msg = UpdateEndowmentSettingsMsg {
            owner: charity_addr.clone(),
            beneficiary: pleb.clone(),
            split_to_liquid: SplitDetails::default(),
        };
        let info = mock_info(charity_addr.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateEndowmentSettings(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // only Registrar SC addr can send msg to terminate the account
        let info = mock_info(&pleb.clone(), &coins(100000, "earth"));
        let env = mock_env();
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::TerminateToAddress {
                beneficiary: ap_team.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // trigger account termination for real
        let info = mock_info(registrar_contract.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::TerminateToAddress {
                beneficiary: ap_team.clone(),
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
                    amount: Uint128::from(35745 as u128)
                },
                Cw20CoinVerified {
                    address: bar_token,
                    amount: Uint128::from(777 as u128)
                }
            ]
        );
    }
}
