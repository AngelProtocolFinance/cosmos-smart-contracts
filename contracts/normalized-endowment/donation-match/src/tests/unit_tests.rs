use super::mock_querier::mock_dependencies;
use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;
use angel_core::messages::donation_match::*;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, coins, from_binary, Addr, Uint128};

const RESERVE_TOKEN: &str = "reserve-token";
const LP_PAIR_CONTRACT: &str = "lp-pair-contract";
const REGISTRAR_CONTRACT: &str = "registrar-contract";

const ENDOWMENT: &str = "endowment_contract";
const UST_AMT: u128 = 50_u128;
const DONOR: &str = "donor";
const DAO_TOKEN: &str = "dao-token";

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("anyone", &[]);
    let instantiate_msg = InstantiateMsg {
        reserve_token: RESERVE_TOKEN.to_string(),
        lp_pair: LP_PAIR_CONTRACT.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };

    // We call "unwrap" for the success
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0)
}

#[test]
fn test_get_config() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("anyone", &[]);
    let instantiate_msg = InstantiateMsg {
        reserve_token: RESERVE_TOKEN.to_string(),
        lp_pair: LP_PAIR_CONTRACT.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };

    // We call "unwrap" for the success
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Check the config query
    let query_bin = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let res: ConfigResponse = from_binary(&query_bin).unwrap();

    assert_eq!(res.reserve_token, RESERVE_TOKEN.to_string());
    assert_eq!(res.lp_pair, LP_PAIR_CONTRACT.to_string());
    assert_eq!(res.registrar_contract, REGISTRAR_CONTRACT.to_string());
}

#[test]
fn test_execute_donor_match() {
    // Instantiate the contract
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("anyone", &[]);
    let instantiate_msg = InstantiateMsg {
        reserve_token: RESERVE_TOKEN.to_string(),
        lp_pair: LP_PAIR_CONTRACT.to_string(),
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
    };

    // We call "unwrap" for the success
    let _ = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Fail the "donor_match" since info.sender is not endowment contract
    let donor_match_msg = ExecuteMsg::DonorMatch {
        amount: Uint128::from(UST_AMT),
        donor: Addr::unchecked(DONOR),
        token: Addr::unchecked(DAO_TOKEN),
    };
    let info = mock_info("not_endowment", &coins(UST_AMT, "uusd"));

    let res = execute(deps.as_mut(), mock_env(), info, donor_match_msg);
    assert!(
        res.is_err(),
        "This call should fail with \"Unauthorized\" error"
    );

    // Fail the "donor_match" since did not send enough UST
    let donor_match_msg = ExecuteMsg::DonorMatch {
        amount: Uint128::from(UST_AMT),
        donor: Addr::unchecked(DONOR),
        token: Addr::unchecked(DAO_TOKEN),
    };
    let info = mock_info(ENDOWMENT, &coins(30, "uusd"));

    let err = execute(deps.as_mut(), mock_env(), info, donor_match_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed the "donor_match" exeuction
    let donor_match_msg = ExecuteMsg::DonorMatch {
        amount: Uint128::from(UST_AMT),
        donor: Addr::unchecked(DONOR),
        token: Addr::unchecked(DAO_TOKEN),
    };
    let info = mock_info(ENDOWMENT, &coins(UST_AMT, "uusd"));

    let res = execute(deps.as_mut(), mock_env(), info, donor_match_msg).unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.attributes,
        vec![
            attr("method", "donor_match"),
            attr("reserve_token", RESERVE_TOKEN),
            attr("dao_token", DAO_TOKEN),
            attr("reserve_token_amt", "100"),
        ]
    );
}
