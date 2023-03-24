use crate::{
    contract::{execute, instantiate, query},
    state::Config,
    tests::mock_querier::mock_dependencies,
};
use angel_core::errors::core::ContractError;
use angel_core::msgs::vault_router::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    Addr,
};

const REGISTRAR_CONTRACT: &str = "registrar-contract";
const OWNER: &str = "contract-owner";

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InstantiateMsg {
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
}

#[test]
fn test_udpate_config() {
    let mut deps = mock_dependencies(&[]);

    // Instantiate the contract
    let instantiate_msg = InstantiateMsg {
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
            registrar_contract: Some("new-registrar-contract".to_string()),
        },
    )
    .unwrap();

    // Check the updated config
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: Config = from_binary(&res).unwrap();
    assert_eq!(config.owner, Addr::unchecked("new-owner"));
    assert_eq!(
        config.registrar_contract,
        Addr::unchecked("new-registrar-contract")
    );
}
