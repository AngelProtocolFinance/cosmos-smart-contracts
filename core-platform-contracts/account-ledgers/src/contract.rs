use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, SubMsg, WasmMsg,
};

use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{
    AccountDetailsResponse, AccountListResponse, ConfigResponse, CreateAcctMsg, ExecuteMsg,
    InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg, UpdateConfigMsg, VaultDetailsResponse,
    VaultListResponse,
};
use crate::state::{
    Account, AssetVault, Config, GenericBalance, RebalanceDetails, Strategy, ACCOUNTS, CONFIG,
    VAULTS,
};

// version info for future migration info
const CONTRACT_NAME: &str = "account-ledgers";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner_addr: info.sender,
            charity_endowment_contract: deps.api.addr_validate("XXXXXXXXXXXXXXXXXXXXXX")?,
            index_fund_contract: deps.api.addr_validate("XXXXXXXXXXXXXXXXXXXXXX")?,
            approved_coins: vec![],
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
    let funds = Balance::from(info.funds.clone());
    match msg {
        ExecuteMsg::CreateAcct(msg) => execute_create(deps, env, msg, &info.sender.clone()),
        ExecuteMsg::UpdateConfig(msg) => update_config(deps, env, info, msg),
        ExecuteMsg::Deposit(msg) => {
            execute_deposit(deps, info, funds.clone(), msg.eid, msg.account_type)
        }
        ExecuteMsg::VaultReceipt(msg) => {
            execute_vault_receipt(deps, info, funds.clone(), msg.eid, msg.account_type)
        }
        ExecuteMsg::UpdateOwner { new_owner } => update_owner(deps, env, info, new_owner),
        ExecuteMsg::UpdateStrategy {
            eid,
            account_type,
            strategy,
        } => update_strategy(deps, env, info, eid, account_type, strategy),
        ExecuteMsg::VaultAdd { vault_addr, vault } => vault_add(deps, env, info, vault_addr, vault),
        ExecuteMsg::VaultUpdateStatus {
            vault_addr,
            approved,
        } => vault_update_status(deps, env, info, vault_addr, approved),
        ExecuteMsg::VaultRemove { vault_addr } => vault_remove(deps, env, info, vault_addr),
        ExecuteMsg::Liquidate { eid, beneficiary } => {
            execute_liquidate(deps, env, info, eid, beneficiary)
        }
        ExecuteMsg::TerminateToFund { eid, fund } => {
            execute_terminate_to_fund(deps, env, info, eid, fund)
        }
        ExecuteMsg::TerminateToAddress { eid, beneficiary } => {
            execute_terminate_to_address(deps, env, info, eid, beneficiary)
        }
        ExecuteMsg::Receive(msg) => execute_receive(deps, info, msg),
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
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }
    // update config attributes with newly passed args
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.owner_addr = new_owner;
        Ok(config)
    })?;

    // TO DO: Need to send out updateOwner messages to all other AP SC:
    // Charity Endowment SC & Index Fund SC in CONFIG
    // all Asset Vault SCs in VAULTS

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // only the owner of the contract can update the configs
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    // validate SC address strings passed
    let charity_endowment_contract = deps.api.addr_validate(&msg.charity_endowment_contract)?;
    let index_fund_contract = deps.api.addr_validate(&msg.index_fund_contract)?;

    // update config attributes with newly passed args
    let approved_coins_list = msg.addr_approved_list(deps.api)?;
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.charity_endowment_contract = charity_endowment_contract;
        config.index_fund_contract = index_fund_contract;
        config.approved_coins = approved_coins_list;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn update_strategy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    eid: String,
    account_type: String,
    strategy: Strategy,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.charity_endowment_contract {
        return Err(ContractError::Unauthorized {});
    }

    // this fails if no account is there
    let account_id = format!("{}_{}", account_type, eid);
    let mut account = ACCOUNTS.load(deps.storage, account_id.clone())?;

    // update account strategy attribute with the newly passed strategy
    account.strategy = strategy;

    // and save
    ACCOUNTS.save(deps.storage, account_id.clone(), &account)?;

    Ok(Response::default())
}

pub fn vault_add(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
    vault: AssetVault,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }
    // save the new vault to storage
    VAULTS.save(deps.storage, vault_addr.clone(), &vault)?;
    Ok(Response::default())
}

pub fn vault_update_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vault_addr: String,
    approved: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // message can only be valid if it comes from the (AP Team/DANO address) SC Owner
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given vault in Storage
    let mut vault = VAULTS.load(deps.storage, vault_addr.clone())?;

    // update new vault approval status attribute from passed arg
    vault.approved = approved;
    VAULTS.save(deps.storage, vault_addr, &vault)?;
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
    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }
    // try to look up the given vault in Storage
    let _vault = VAULTS.load(deps.storage, vault_addr.clone())?;
    // delete the vault
    VAULTS.remove(deps.storage, vault_addr);
    Ok(Response::default())
}

pub fn execute_create(
    deps: DepsMut,
    _env: Env,
    msg: CreateAcctMsg,
    sender: &Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if sender != &config.charity_endowment_contract {
        return Err(ContractError::Unauthorized {});
    }

    let account = Account {
        balance: GenericBalance {
            native: vec![],
            cw20: vec![],
        },
        strategy: Strategy::default(),
        rebalance: RebalanceDetails::default(),
    };

    // try to create both prefixed accounts based on EID passed
    for prefix in ["locked", "liquid"].iter() {
        // try to store it, fail if the account ID was already in use
        ACCOUNTS.update(
            deps.storage,
            format!("{}_{}", prefix, msg.eid.clone()),
            |existing| match existing {
                None => Ok(account.clone()),
                Some(_) => Err(ContractError::AlreadyInUse {}),
            },
        )?;
    }
    let res = Response {
        attributes: vec![attr("action", "create"), attr("id", sender.clone())],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender.clone(),
        amount: wrapper.amount,
    });
    let msg = from_binary(&wrapper.msg)?;
    match msg {
        ReceiveMsg::Deposit(msg) => execute_deposit(deps, info, balance, msg.eid, msg.account_type),
        ReceiveMsg::VaultReceipt(msg) => {
            execute_vault_receipt(deps, info, balance, msg.eid, msg.account_type)
        }
    }
}

pub fn execute_vault_receipt(
    deps: DepsMut,
    info: MessageInfo,
    balance: Balance,
    eid: String,
    account_type: String,
) -> Result<Response, ContractError> {
    // this lookup fails if the token deposit was not coming from an Asset Vault SC
    let vault = VAULTS.load(deps.storage, info.sender.to_string())?;
    if !vault.approved {
        return Err(ContractError::Unauthorized {});
    }

    // this fails if no account is there
    let account_id = format!("{}_{}", account_type, eid);
    let mut account = ACCOUNTS.load(deps.storage, account_id.clone())?;

    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    account.balance.add_tokens(balance);

    // and save
    ACCOUNTS.save(deps.storage, account_id.clone(), &account)?;

    let res = Response {
        attributes: vec![
            attr("action", "vault_receipt"),
            attr("eid", eid),
            attr("account_type", account_type),
        ],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_deposit(
    deps: DepsMut,
    info: MessageInfo,
    balance: Balance,
    eid: String,
    account_type: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // this fails if no account is there
    let account_id = format!("{}_{}", account_type, eid);
    let mut account = ACCOUNTS.load(deps.storage, account_id.clone())?;

    // this lookup fails if the token deposit was not coming from:
    // an Asset Vault SC, the Charity Endownment SC, or the Index Fund SC
    if info.sender != config.charity_endowment_contract || info.sender != config.index_fund_contract
    {
        return Err(ContractError::Unauthorized {});
    }

    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the approved_coins
        if !config.approved_coins.iter().any(|t| t == &token.address) {
            return Err(ContractError::NotInApprovedCoins {});
        }
    };

    account.balance.add_tokens(balance);

    // and save
    ACCOUNTS.save(deps.storage, account_id.clone(), &account)?;

    let res = Response {
        attributes: vec![
            attr("action", "deposit"),
            attr("eid", eid),
            attr("account_type", account_type),
        ],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_liquidate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    eid: String,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.charity_endowment_contract {
        return Err(ContractError::Unauthorized {});
    }
    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    for prefix in ["locked", "liquid"].iter() {
        let account_id = format!("{}_{}", prefix, eid);
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, account_id.clone())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, account_id.clone());
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
    eid: String,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    // validate the beneficiary address passed
    let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;

    let mut messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        let account_id = format!("{}_{}", prefix, eid);
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, account_id.clone())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, account_id.clone());
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
    eid: String,
    fund: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner_addr {
        return Err(ContractError::Unauthorized {});
    }
    let mut messages = vec![];
    for prefix in ["locked", "liquid"].iter() {
        let account_id = format!("{}_{}", prefix, eid);
        // this fails if no account is found
        let account = ACCOUNTS.load(deps.storage, account_id.clone())?;
        // we delete the account
        ACCOUNTS.remove(deps.storage, account_id.clone());
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
        QueryMsg::Vault { address } => to_binary(&query_vault_details(deps, address)?),
        QueryMsg::VaultList { non_approved } => to_binary(&query_vault_list(deps, non_approved)?),
        QueryMsg::Account { eid, account_type } => {
            to_binary(&query_account_details(deps, eid, account_type)?)
        }
        QueryMsg::AccountList { eid } => to_binary(&query_account_list(deps, eid)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        owner_addr: config.owner_addr.to_string(),
        charity_endowment_contract: config.charity_endowment_contract.to_string(),
        index_fund_contract: config.index_fund_contract.to_string(),
        approved_coins: config.human_approved_coins(),
    };
    Ok(res)
}

fn query_vault_details(deps: Deps, address: String) -> StdResult<VaultDetailsResponse> {
    // this fails if no vault is found
    let vault = VAULTS.load(deps.storage, address.clone())?;

    let details = VaultDetailsResponse {
        address: address,
        name: vault.name,
        description: vault.description,
        approved: vault.approved,
    };
    Ok(details)
}

fn query_vault_list(_deps: Deps, _non_approved: Option<bool>) -> StdResult<VaultListResponse> {
    let list = VaultListResponse { vaults: vec![] };
    Ok(list)
}

fn query_account_details(
    deps: Deps,
    eid: String,
    account_type: String,
) -> StdResult<AccountDetailsResponse> {
    // this fails if no account is found
    let account_id = format!("{}_{}", account_type, eid);
    let account = ACCOUNTS.load(deps.storage, account_id.clone())?;

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
        eid: eid,
        account_type: account_type,
        strategy: account.strategy,
        balance: balance?,
    };
    Ok(details)
}

fn query_account_list(_deps: Deps, _eid: Option<String>) -> StdResult<AccountListResponse> {
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
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Uint128};
    use cw20::Cw20CoinVerified;

    #[test]
    fn test_proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len()); // no news is good news! :)
    }

    #[test]
    fn test_get_config() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");
        let charity_endowment_contract = String::from("XCEMQTWTETGSGSRHJTUIQADG");
        let index_fund_contract = String::from("SDFGRHAETHADFARHSRTHADGG");
        let pleb = String::from("plebAccount");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the approved coins list and trusted SC addresses
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth"), String::from("mars")]),
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
        assert_eq!(2, res.approved_coins.len());
        assert_eq!(
            charity_endowment_contract.clone(),
            res.charity_endowment_contract
        );

        // Not just anyone can update the configs! Only owner can.
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(pleb.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        // This should fail with an error!
        let err = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateConfig(msg)).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_change_contract_owner() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");
        let charity_endowment_contract = String::from("XCEMQTWTETGSGSRHJTUIQADG");
        let index_fund_contract = String::from("SDFGRHAETHADFARHSRTHADGG");
        let pleb = String::from("plebAccount");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // change the owner to Agent2
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateOwner {
                new_owner: pleb.clone(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // check changes saved and can be recalled
        let res = query_config(deps.as_ref()).unwrap();
        assert_eq!(pleb.clone(), res.owner_addr);

        // Original AP_Team address should not be able to update the configs now
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth")]),
        };
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth "));
        let env = mock_env();
        // This should fail with an error!
        let err = execute(deps.as_mut(), env, info, ExecuteMsg::UpdateConfig(msg)).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_create_account() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");
        let charity_endowment_contract = String::from("XCEMQTWTETGSGSRHJTUIQADG");
        let index_fund_contract = String::from("SDFGRHAETHADFARHSRTHADGG");
        let pleb = String::from("plebAccount");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the approved coins list and trusted SC addresses
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth"), String::from("mars")]),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateConfig(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        let msg = CreateAcctMsg {
            eid: String::from("GWRGDRGERGRGRGDRGDRGSGSDFS"),
        };

        // only the Charity Endowment SC can create accounts (not plebs!)
        let info = mock_info(&pleb.clone(), &coins(100000, "earth"));
        let env = mock_env();
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg.clone()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // Create the account for real from the trusted SC address
        let info = mock_info(&charity_endowment_contract.clone(), &coins(100000, "earth"));
        let env = mock_env();
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
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");
        let charity_endowment_contract = String::from("XCEMQTWTETGSGSRHJTUIQADG");
        let index_fund_contract = String::from("SDFGRHAETHADFARHSRTHADGG");
        let pleb = String::from("plebAccount");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the approved coins list and trusted SC addresses
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth"), String::from("mars")]),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateConfig(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // create a new account
        let msg_locked = CreateAcctMsg {
            eid: String::from("XCEMQTWTETGSGSRHJTUIQADG"),
        };
        let info = mock_info(charity_endowment_contract.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg_locked),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // only AP team/DANO (SC owner) can terminate the account
        let info = mock_info(&pleb.clone(), &coins(100000, "earth"));
        let env = mock_env();
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::TerminateToAddress {
                eid: String::from("XCEMQTWTETGSGSRHJTUIQADG"),
                beneficiary: ap_team.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // trigger account termination for real
        let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::TerminateToAddress {
                eid: String::from("XCEMQTWTETGSGSRHJTUIQADG"),
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

    #[test]
    fn test_create_new_endowment_accounts() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");
        let charity_endowment_contract = String::from("XCEMQTWTETGSGSRHJTUIQADG");
        let index_fund_contract = String::from("SDFGRHAETHADFARHSRTHADGG");
        let pleb = String::from("plebAccount");
        // create an account id for a fictional Endowment (EID)
        let eid = String::from("GWRGDRGERGRGRGDRGDRGSGSDFS");
        let account_type = String::from("locked");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "bar_token"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the approved coins list and trusted SC addresses
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth"), String::from("mars")]),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateConfig(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // test a non-owner account can't create accounts
        let msg = CreateAcctMsg { eid: eid.clone() };
        let info = mock_info(pleb.as_ref(), &coins(100000, "bar_token"));
        let env = mock_env();
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // create a set of new accounts
        let msg = CreateAcctMsg { eid: eid.clone() };
        let info = mock_info(
            charity_endowment_contract.as_ref(),
            &coins(100000, "bar_token"),
        );
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateAcct(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // should be able to get a created account now (ex. Locked Acct)
        let res = query_account_details(deps.as_ref(), eid.clone(), account_type.clone()).unwrap();
        assert_eq!(eid.clone(), res.eid);
        assert_eq!(account_type.clone(), res.account_type);
    }

    #[test]
    fn test_create_asset_vault() {
        let mut deps = mock_dependencies(&[]);
        // meet the cast of characters
        let ap_team = String::from("angelprotocolteamdano");
        let charity_endowment_contract = String::from("XCEMQTWTETGSGSRHJTUIQADG");
        let index_fund_contract = String::from("SDFGRHAETHADFARHSRTHADGG");
        let pleb = String::from("plebAccount");
        let asset_vault = String::from("greatestAssetVaultEver");

        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(ap_team.as_ref(), &coins(100000, "bar_token"));
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // update the approved coins list and trusted SC addresses
        let msg = UpdateConfigMsg {
            charity_endowment_contract: charity_endowment_contract.clone(),
            index_fund_contract: index_fund_contract.clone(),
            approved_coins: Some(vec![String::from("earth"), String::from("mars")]),
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateConfig(msg),
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // test a non-owner account can't add new vaults
        let info = mock_info(pleb.as_ref(), &coins(100000, "bar_token"));
        let env = mock_env();
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::VaultAdd {
                vault_addr: asset_vault.clone(),
                vault: AssetVault {
                    name: String::from("Greatest Asset Vault Ever"),
                    description: String::from(
                        "We give investors a 1000% APY return on their assets.",
                    ),
                    approved: true,
                },
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // create a new AssetVault
        let info = mock_info(ap_team.as_ref(), &coins(100000, "bar_token"));
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::VaultAdd {
                vault_addr: asset_vault.clone(),
                vault: AssetVault {
                    name: String::from("Greatest Asset Vault Ever"),
                    description: String::from(
                        "We give investors a 1000% APY return on their assets.",
                    ),
                    approved: true,
                },
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());

        // should be able to get a created vault now
        let res = query_vault_details(deps.as_ref(), asset_vault.clone()).unwrap();
        assert_eq!(String::from("Greatest Asset Vault Ever"), res.name);
    }
}
