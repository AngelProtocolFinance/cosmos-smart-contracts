use crate::contract::{execute, instantiate, query_config, reply};
use crate::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Decimal, Reply, ReplyOn, StdError, SubMsg, SubMsgResponse,
    SubMsgResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use halo_token::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        swap_factory: "swapfactory".to_string(),
        gov_contract: "gov".to_string(),
        halo_token: "token-halo".to_string(),
        distributor_contract: "distributor".to_string(),
        reward_factor: Decimal::percent(90),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse = query_config(deps.as_ref()).unwrap();
    assert_eq!("swapfactory", config.swap_factory.as_str());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        swap_factory: "swapfactory".to_string(),
        gov_contract: "gov".to_string(),
        halo_token: "token-halo".to_string(),
        distributor_contract: "distributor".to_string(),
        reward_factor: Decimal::percent(90),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update reward_factor
    let info = mock_info("gov", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        reward_factor: Some(Decimal::percent(80)),
        gov_contract: None,
        swap_factory: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value = query_config(deps.as_ref()).unwrap();
    assert_eq!(Decimal::percent(80), value.reward_factor);

    // Unauthorized err
    let info = mock_info("addr1111", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        reward_factor: None,
        gov_contract: None,
        swap_factory: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn test_sweep() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(100u128),
    }]);

    deps.querier
        .with_terraswap_pairs(&[(&"uusdtoken-halo".to_string(), &"pairANC".to_string())]);

    let msg = InstantiateMsg {
        swap_factory: "swapfactory".to_string(),
        gov_contract: "gov".to_string(),
        halo_token: "token-halo".to_string(),
        distributor_contract: "distributor".to_string(),
        reward_factor: Decimal::percent(90),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::Sweep {
        denom: "uusd".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // No tax deduct
    assert_eq!(
        res.messages,
        vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: "pairANC".to_string(),
                msg: to_binary(&TerraswapExecuteMsg::Swap {
                    offer_asset: Asset {
                        info: AssetInfo::NativeToken {
                            denom: "uusd".to_string()
                        },
                        amount: Uint128::from(100u128),
                    },
                    max_spread: None,
                    belief_price: None,
                    to: None,
                })
                .unwrap(),
                funds: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(100u128),
                }],
            }
            .into(),
            gas_limit: None,
            id: 1,
            reply_on: ReplyOn::Success,
        }]
    );
}

#[test]
fn test_distribute() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_balances(&[(
        &"token-halo".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
    )]);

    let msg = InstantiateMsg {
        swap_factory: "swapfactory".to_string(),
        gov_contract: "gov".to_string(),
        halo_token: "token-halo".to_string(),
        distributor_contract: "distributor".to_string(),
        reward_factor: Decimal::percent(90),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token-halo".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "gov".to_string(),
                    amount: Uint128::from(90u128),
                })
                .unwrap(),
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token-halo".to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "distributor".to_string(),
                    amount: Uint128::from(10u128),
                })
                .unwrap(),
                funds: vec![],
            }))
        ]
    )
}
