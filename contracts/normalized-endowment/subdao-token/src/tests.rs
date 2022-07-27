use angel_core::errors::core::{ContractError, PaymentError};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, to_binary, Decimal, OverflowError, StdError, Uint128};
use cw20::{BalanceResponse, Cw20ReceiveMsg};

use crate::contract::{execute, instantiate, query};
use angel_core::messages::subdao_token::{
    CurveInfoResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let instantiate_msg = InstantiateMsg {
        name: "Dao-Token".to_string(),
        symbol: "DT".to_string(),
        decimals: 6,
        reserve_denom: "reserve-token-address".to_string(),
        reserve_decimals: 6,
        curve_type: angel_core::messages::subdao_token::CurveType::Constant {
            value: Uint128::from(10_u128),
            scale: 1,
        },
        unbonding_period: 100_u64,
    };

    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    assert_eq!(res.messages.len(), 0);

    // query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::CurveInfo {}).unwrap();
    let curve: CurveInfoResponse = from_binary(&res).unwrap();
    assert_eq!(curve.reserve_denom, "reserve-token-address".to_string());
    assert_eq!(curve.supply, Uint128::zero());
    assert_eq!(curve.reserve, Uint128::zero());
    assert_eq!(
        curve.spot_price,
        Decimal::new(Uint128::from(1000000000000000000_u128))
    );
}

#[test]
fn test_claim_tokens() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let instantiate_msg = InstantiateMsg {
        name: "Dao-Token".to_string(),
        symbol: "DT".to_string(),
        decimals: 6,
        reserve_denom: "reserve-token-address".to_string(),
        reserve_decimals: 6,
        curve_type: angel_core::messages::subdao_token::CurveType::Constant {
            value: Uint128::from(10_u128),
            scale: 1,
        },
        unbonding_period: 100_u64,
    };

    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Fails to "claim_tokens" because of the period
    let info = mock_info("claimer", &[]);
    let claim_token_msg = ExecuteMsg::ClaimTokens {};
    let err = execute(deps.as_mut(), mock_env(), info, claim_token_msg).unwrap_err();

    assert_eq!(err, ContractError::NothingToClaim {});

    // Success to claim tokens
    // TODO: How to test the "claim_tokens" logic

    // // subaction 1: Buy the dao-token
    // let info = mock_info("reserve-token-address", &[]);
    // let cw20_receive_msg = Cw20ReceiveMsg {
    //     sender: "buyer".to_string(),
    //     amount: Uint128::from(10_u128),
    //     msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    // };
    // let _res = execute(
    //     deps.as_mut(),
    //     mock_env(),
    //     info,
    //     ExecuteMsg::Receive(cw20_receive_msg),
    // )
    // .unwrap();

    // // subaction 2. Succeed to "sell"
    // let info = mock_info("buyer", &[]);
    // let sell_msg = ExecuteMsg::Burn {
    //     amount: Uint128::from(5_u128),
    // };
    // let _res = execute(deps.as_mut(), mock_env(), info, sell_msg).unwrap();

    // // subaction 3: Claim the tokens
    // let info = mock_info("reserve-token-address", &[]);
    // let cw20_receive_msg = Cw20ReceiveMsg {
    //     sender: "buyer".to_string(),
    //     amount: Uint128::from(10_u128),
    //     msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    // };
    // let _res = execute(
    //     deps.as_mut(),
    //     mock_env(),
    //     info,
    //     ExecuteMsg::Receive(cw20_receive_msg),
    // )
    // .unwrap();

    // let mut env = mock_env();
    // env.block.height += 100;
    // let info = mock_info("claimer", &[]);
    // let claim_token_msg = ExecuteMsg::ClaimTokens {};
    // let res = execute(deps.as_mut(), env, info, claim_token_msg).unwrap();

    // assert_eq!(res.messages.len(), 1);
    // assert_eq!(res.attributes.len(), 2);
}

#[test]
fn test_buy() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let instantiate_msg = InstantiateMsg {
        name: "Dao-Token".to_string(),
        symbol: "DT".to_string(),
        decimals: 6,
        reserve_denom: "reserve-token-address".to_string(),
        reserve_decimals: 6,
        curve_type: angel_core::messages::subdao_token::CurveType::Constant {
            value: Uint128::from(10_u128),
            scale: 1,
        },
        unbonding_period: 100_u64,
    };

    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Fails to "buy" because of sending wrong token
    let info = mock_info("non-reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "buyer".to_string(),
        amount: Uint128::from(10_u128),
        msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fails to "buy" because of sending 0 token
    let info = mock_info("reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "buyer".to_string(),
        amount: Uint128::from(0_u128),
        msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // Succeed to "buy"
    let info = mock_info("reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "buyer".to_string(),
        amount: Uint128::from(10_u128),
        msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);

    // query the state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "buyer".to_string(),
        },
    )
    .unwrap();
    let dao_token_bal: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(dao_token_bal.balance, Uint128::from(10_u128));
}

#[test]
fn test_sell() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let instantiate_msg = InstantiateMsg {
        name: "Dao-Token".to_string(),
        symbol: "DT".to_string(),
        decimals: 6,
        reserve_denom: "reserve-token-address".to_string(),
        reserve_decimals: 6,
        curve_type: angel_core::messages::subdao_token::CurveType::Constant {
            value: Uint128::from(10_u128),
            scale: 1,
        },
        unbonding_period: 100_u64,
    };

    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // "Buy" some dao-tokens
    let info = mock_info("reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "buyer".to_string(),
        amount: Uint128::from(10_u128),
        msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    };
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap();

    // Fails to "sell" because of sending some tokens
    let info = mock_info("buyer", &coins(100, "earth"));
    let sell_msg = ExecuteMsg::Burn {
        amount: Uint128::from(5_u128),
    };
    let err = execute(deps.as_mut(), mock_env(), info, sell_msg).unwrap_err();
    assert_eq!(err, ContractError::Payment(PaymentError::NonPayable {}));

    // Fails to "sell" because of asking more than owned
    let info = mock_info("buyer", &[]);
    let sell_msg = ExecuteMsg::Burn {
        amount: Uint128::from(15_u128),
    };
    let err = execute(deps.as_mut(), mock_env(), info, sell_msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::overflow(OverflowError {
            operation: cosmwasm_std::OverflowOperation::Sub,
            operand1: "10".to_string(),
            operand2: "15".to_string(),
        }))
    );

    // Succeed to "sell"
    let info = mock_info("buyer", &[]);
    let sell_msg = ExecuteMsg::Burn {
        amount: Uint128::from(5_u128),
    };
    let res = execute(deps.as_mut(), mock_env(), info, sell_msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes.len(), 4);

    // query the state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "buyer".to_string(),
        },
    )
    .unwrap();
    let dao_token_bal: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(dao_token_bal.balance, Uint128::from(5_u128));
}

#[test]
fn test_transfer() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let instantiate_msg = InstantiateMsg {
        name: "Dao-Token".to_string(),
        symbol: "DT".to_string(),
        decimals: 6,
        reserve_denom: "reserve-token-address".to_string(),
        reserve_decimals: 6,
        curve_type: angel_core::messages::subdao_token::CurveType::Constant {
            value: Uint128::from(10_u128),
            scale: 1,
        },
        unbonding_period: 100_u64,
    };

    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Succeed to "buy"
    let info = mock_info("reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "buyer".to_string(),
        amount: Uint128::from(10_u128),
        msg: to_binary(&Cw20HookMsg::Buy {}).unwrap(),
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap();

    // Succeed to transfer the tokens
    let info = mock_info("buyer", &[]);
    let transfer_msg = ExecuteMsg::Transfer {
        recipient: "recipient".to_string(),
        amount: Uint128::from(5_u128),
    };
    let res = execute(deps.as_mut(), mock_env(), info, transfer_msg).unwrap();
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.messages.len(), 0);

    // query the state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "buyer".to_string(),
        },
    )
    .unwrap();
    let dao_token_bal: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(dao_token_bal.balance, Uint128::from(5_u128));
}

#[test]
fn test_donor_match() {
    let mut deps = mock_dependencies();
    let info = mock_info("creator", &[]);
    let instantiate_msg = InstantiateMsg {
        name: "Dao-Token".to_string(),
        symbol: "DT".to_string(),
        decimals: 6,
        reserve_denom: "reserve-token-address".to_string(),
        reserve_decimals: 6,
        curve_type: angel_core::messages::subdao_token::CurveType::Constant {
            value: Uint128::from(10_u128),
            scale: 1,
        },
        unbonding_period: 100_u64,
    };

    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Fails to "donor_match" because of wrong token
    let info = mock_info("non-reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "executer".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&Cw20HookMsg::DonorMatch {
            amount: Uint128::from(100_u128),
            donor: "donor".to_string(),
            endowment_contract: "endowment-contract".to_string(),
        })
        .unwrap(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fails to "donor_match" because of non-matching amount
    let info = mock_info("reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "executer".to_string(),
        amount: Uint128::from(10_u128),
        msg: to_binary(&Cw20HookMsg::DonorMatch {
            amount: Uint128::from(100_u128),
            donor: "donor".to_string(),
            endowment_contract: "endowment-contract".to_string(),
        })
        .unwrap(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to "donor_match"
    let info = mock_info("reserve-token-address", &[]);
    let cw20_receive_msg = Cw20ReceiveMsg {
        sender: "executer".to_string(),
        amount: Uint128::from(100_u128),
        msg: to_binary(&Cw20HookMsg::DonorMatch {
            amount: Uint128::from(100_u128),
            donor: "donor".to_string(),
            endowment_contract: "endowment-contract".to_string(),
        })
        .unwrap(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20_receive_msg),
    )
    .unwrap();
    assert_eq!(res.attributes.len(), 4);
    assert_eq!(res.messages.len(), 0);

    // query the state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "donor".to_string(),
        },
    )
    .unwrap();
    let dao_token_bal: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(dao_token_bal.balance, Uint128::from(40_u128));

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Balance {
            address: "endowment-contract".to_string(),
        },
    )
    .unwrap();
    let dao_token_bal: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(dao_token_bal.balance, Uint128::from(40_u128));
}
