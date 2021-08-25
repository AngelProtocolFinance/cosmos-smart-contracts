use crate::contract::instantiate;
use crate::msg::InitMsg;
use crate::queriers;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Deps, DepsMut, Uint128};
use cw20::TokenInfoResponse;

fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    queriers::query_balance(deps, address.into()).balance
}

// this will set up the instantiation for other tests
fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
    _do_instantiate(deps, addr, amount)
}

// this will set up the instantiation for other tests
fn _do_instantiate(mut deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
    let instantiate_msg = InitMsg {
        name: "Auto Gen".to_string(),
        symbol: "AUTO".to_string(),
        decimals: 6,
        moneymarket: "anchorprotocolmoneymrk".to_string(),
        registrar_contract: "angelprotocolteamdano".to_string(),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let meta = queriers::query_token_info(deps.as_ref());
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            total_supply: amount,
        }
    );
    assert_eq!(get_balance(deps.as_ref(), addr), amount);
    meta
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InitMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
        moneymarket: "anchorprotocolmoneymrk".to_string(),
        registrar_contract: "angelprotocolteamdano".to_string(),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(1, res.messages.len());

    assert_eq!(
        queriers::query_token_info(deps.as_ref()),
        TokenInfoResponse {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );
    assert_eq!(get_balance(deps.as_ref(), "addr0000"), Uint128::zero());
}
