use angel_core::{
    errors::core::ContractError, messages::accounts::DepositMsg, structs::GenericBalance,
};
use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, Addr, Coin, Decimal, StdError, Uint128,
};
use cw_asset::{AssetBase, AssetInfoBase};

use crate::{
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, Deposit},
    tests::mock_querier::mock_dependencies,
};

const KEEPER: &str = "keeper";
const REGISTRAR_CONTRACT: &str = "registrar-contract";
const OWNER: &str = "contract-owner";
const DEPOSITOR: &str = "depositor";

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
        keeper: KEEPER.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(OWNER, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // Check the config for success
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(config.owner, OWNER);
    assert_eq!(config.registrar_contract, REGISTRAR_CONTRACT);
    assert_eq!(config.keeper, KEEPER);
    assert_eq!(config.next_deposit, 1_u64);
}

#[test]
fn test_deposit() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
        keeper: KEEPER.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Cannot deposit the multiple coins, ONLY 1 coin at a time
    let info = mock_info(
        DEPOSITOR,
        &[
            Coin {
                denom: "ujuno".to_string(),
                amount: Uint128::from(100_u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100_u128),
            },
        ],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { to_address: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Deposited token should be one of "accepted_tokens"
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { to_address: None },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!("Not accepted token: {}", "uusd"),
        })
    );

    // Cannot deposit 0 tokens
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::zero(),
        }],
    );
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { to_address: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // If "to_address" is None, it just saves the deposit info.
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { to_address: None },
    )
    .unwrap();
    assert_eq!(res.attributes.len(), 2);

    // Check the deposit info & ID
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(config.next_deposit, 2_u64);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Deposit {
            deposit_id: config.next_deposit - 1,
        },
    )
    .unwrap();
    let deposit: Deposit = from_binary(&res).unwrap();
    assert_eq!(deposit.claimed, false);
    assert_eq!(deposit.sender, DEPOSITOR);
    assert_eq!(
        deposit.token,
        cw_asset::AssetBase::native("ujuno", 100_u128)
    );

    // If "to_address" is given, BALANCES state is updated.
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit {
            to_address: Some("receiver".to_string()),
        },
    )
    .unwrap();
    assert_eq!(res.attributes.len(), 2);

    // Query the balance
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "receiver".to_string(),
        },
    )
    .unwrap();
    let balance: GenericBalance = from_binary(&res).unwrap();
    assert_eq!(
        balance.native,
        vec![Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128)
        }]
    );
}

#[test]
fn test_deposit_cw20() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
        keeper: KEEPER.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    //  Deposited cw20 token should be one of "accepted_tokens".
    let info = mock_info("any-cw20", &[]);
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&crate::msg::ReceiveMsg::Deposit { to_address: None }).unwrap(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: format!("Not accepted token: {}", "any-cw20"),
        })
    );

    // Cannot deposit 0 tokens.
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&crate::msg::ReceiveMsg::Deposit { to_address: None }).unwrap(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // Succeed to deposit the cw20 tokens
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&crate::msg::ReceiveMsg::Deposit { to_address: None }).unwrap(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap();
    assert_eq!(res.attributes.len(), 2);

    // Check the deposit info & ID
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(config.next_deposit, 2_u64);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Deposit {
            deposit_id: config.next_deposit - 1,
        },
    )
    .unwrap();
    let deposit: Deposit = from_binary(&res).unwrap();
    assert_eq!(deposit.claimed, false);
    assert_eq!(deposit.sender, DEPOSITOR);
    assert_eq!(
        deposit.token,
        cw_asset::AssetBase::cw20(Addr::unchecked("test-cw20"), 100_u128)
    );

    // If "to_address" is given, BALANCES state is updated.
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&crate::msg::ReceiveMsg::Deposit {
            to_address: Some("receiver".to_string()),
        })
        .unwrap(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap();
    assert_eq!(res.attributes.len(), 2);

    // Query the balance
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "receiver".to_string(),
        },
    )
    .unwrap();
    let balance: GenericBalance = from_binary(&res).unwrap();
    assert_eq!(
        balance.cw20,
        vec![cw20::Cw20CoinVerified {
            address: Addr::unchecked("test-cw20"),
            amount: Uint128::from(100_u128)
        }]
    );
}

#[test]
fn test_claim() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
        keeper: KEEPER.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Deposit the un-claimed tokens
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit { to_address: None },
    )
    .unwrap();

    let unclaimed_deposit_id = res.attributes[1].value.parse::<u64>().unwrap();

    // Deposit the claimed tokens
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit {
            to_address: Some("receiver".to_string()),
        },
    )
    .unwrap();
    let claimed_deposit_id = res.attributes[1].value.parse::<u64>().unwrap();

    // Only keeper address can call this "claim" entry
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Claim {
            deposit: unclaimed_deposit_id,
            recipient: "recipient".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Requested deposit should exist.
    let info = mock_info(KEEPER, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Claim {
            deposit: claimed_deposit_id + 100,
            recipient: "recipient".to_string(),
        },
    )
    .unwrap_err();

    // Requested deposit should be unclaimed
    let info = mock_info(KEEPER, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Claim {
            deposit: claimed_deposit_id,
            recipient: "recipient".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Deposit has already been claimed".to_string(),
        })
    );

    // Succeed to claim the deposit for "recipient" address.
    let info = mock_info(KEEPER, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Claim {
            deposit: unclaimed_deposit_id,
            recipient: "recipient".to_string(),
        },
    )
    .unwrap();
    assert_eq!(res.attributes.len(), 1);

    // Check the claim action result.
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Deposit {
            deposit_id: unclaimed_deposit_id,
        },
    )
    .unwrap();
    let deposit: Deposit = from_binary(&res).unwrap();
    assert_eq!(deposit.claimed, true);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "recipient".to_string(),
        },
    )
    .unwrap();
    let balance: GenericBalance = from_binary(&res).unwrap();
    assert_eq!(
        balance.native,
        vec![Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128)
        }]
    );
}

#[test]
fn test_spend() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
        keeper: KEEPER.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Will output error if no fund available for sender
    let info = mock_info("anyone", &[]);
    let _err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Spend {
            asset: AssetBase {
                info: AssetInfoBase::native("ujuno"),
                amount: Uint128::from(30_u128),
            },
            deposit_msg: DepositMsg {
                id: 1_u32,
                locked_percentage: Decimal::default(),
                liquid_percentage: Decimal::default(),
            },
        },
    )
    .unwrap_err();

    // Deposit the claimed tokens
    let info = mock_info(
        DEPOSITOR,
        &[Coin {
            denom: "ujuno".to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Deposit {
            to_address: Some("receiver".to_string()),
        },
    )
    .unwrap();

    // Will output error if trying to spend zero or insufficient balance.
    let info = mock_info("receiver", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Spend {
            asset: AssetBase {
                info: AssetInfoBase::native("uusd"),
                amount: Uint128::from(30_u128),
            },
            deposit_msg: DepositMsg {
                id: 1_u32,
                locked_percentage: Decimal::default(),
                liquid_percentage: Decimal::default(),
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    let info = mock_info("receiver", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Spend {
            asset: AssetBase {
                info: AssetInfoBase::native("ujuno"),
                amount: Uint128::from(300_u128),
            },
            deposit_msg: DepositMsg {
                id: 1_u32,
                locked_percentage: Decimal::default(),
                liquid_percentage: Decimal::default(),
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to spend the funds
    let info = mock_info("receiver", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Spend {
            asset: AssetBase {
                info: AssetInfoBase::native("ujuno"),
                amount: Uint128::from(30_u128),
            },
            deposit_msg: DepositMsg {
                id: 1_u32,
                locked_percentage: Decimal::default(),
                liquid_percentage: Decimal::default(),
            },
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    // Check the deducted balance
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "receiver".to_string(),
        },
    )
    .unwrap();
    let balance: GenericBalance = from_binary(&res).unwrap();
    assert_eq!(balance.native.len(), 1);
    assert_eq!(balance.native[0].denom, "ujuno".to_string());
    assert_eq!(balance.native[0].amount, Uint128::from(100_u128 - 30_u128));

    // Same logic applies to cw20 token balances
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = cw20::Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&crate::msg::ReceiveMsg::Deposit {
            to_address: Some("receiver".to_string()),
        })
        .unwrap(),
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(deposit_msg),
    )
    .unwrap();

    let info = mock_info("receiver", &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Spend {
            asset: AssetBase {
                info: AssetInfoBase::cw20(Addr::unchecked("test-cw20")),
                amount: Uint128::from(30_u128),
            },
            deposit_msg: DepositMsg {
                id: 1_u32,
                locked_percentage: Decimal::default(),
                liquid_percentage: Decimal::default(),
            },
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_udpate_config() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
        keeper: KEEPER.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };
    let info = mock_info(OWNER, &[]);
    instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Only owner can execute this entry
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig {
            owner: Some("new-owner".to_string()),
            keeper: Some("new-keeper".to_string()),
            registrar_contract: Some("new-registrar".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to update the config
    let info = mock_info(OWNER, &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig {
            owner: Some("new-owner".to_string()),
            keeper: Some("new-keeper".to_string()),
            registrar_contract: Some("new-registrar-contract".to_string()),
        },
    )
    .unwrap();

    // Check the updated config
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(config.owner, Addr::unchecked("new-owner"));
    assert_eq!(config.keeper, Addr::unchecked("new-keeper"));
    assert_eq!(
        config.registrar_contract,
        Addr::unchecked("new-registrar-contract")
    );
}
