use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;
use angel_core::msgs::registrar::*;
use angel_core::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails, StrategyApprovalState,
    StrategyLocale, StrategyParams,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Addr, Decimal, StdError};

const MOCK_CW3_CODE_ID: u64 = 18;
const MOCK_CW4_CODE_ID: u64 = 19;
const AP_TEAM: &str = "juno1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const REVIEW_TEAM: &str = "juno1rcznds2le2eflj3y4e8ep3e4upvq04sc65xxxx";
const USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const STRATEGY_KEY: &str = "strategy-native";
const CHAIN_ID: &str = "juno";

fn instantiate_msg() -> InstantiateMsg {
    return InstantiateMsg {
        treasury: AP_TEAM.to_string(),
        tax_rate: Decimal::percent(20),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![USDC.to_string()],
            cw20: vec![],
        }),
        swap_factory: None,
        axelar_gateway: "axelar-gateway".to_string(),
        axelar_ibc_channel: "channel-1".to_string(),
    };
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Check the result
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(None, config_response.accounts_contract);
    assert_eq!(ap_team.clone(), config_response.owner);
    assert_eq!(RebalanceDetails::default(), config_response.rebalance);
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let pleb = "juno17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    let info = mock_info(pleb.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::UpdateOwner {
        new_owner: String::from("alice"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let index_fund_contract = String::from("juno1typpfzq9ynmvrt6tt459epfqn4gqejhy6lmu7d");
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Only config.owner can update the config
    let info = mock_info("anyone", &coins(1000, "earth"));
    let update_config_message = UpdateConfigMsg {
        accounts_contract: Some("accounts_contract_addr".to_string()),
        index_fund_contract: Some(index_fund_contract.clone()),
        treasury: Some(ap_team.clone()),
        rebalance: Some(RebalanceDetails {
            rebalance_liquid_invested_profits: true,
            locked_interests_to_liquid: true,
            interest_distribution: Decimal::one(),
            locked_principle_to_liquid: true,
            principle_distribution: Decimal::one(),
        }),
        split_max: Some(Decimal::one()),
        split_min: Some(Decimal::zero()),
        split_default: Some(Decimal::percent(30)),
        charity_shares_contract: None,
        gov_contract: None,
        halo_token: None,
        cw3_code: Some(MOCK_CW3_CODE_ID),
        cw4_code: Some(MOCK_CW4_CODE_ID),
        applications_review: Some(REVIEW_TEAM.to_string()),
        swaps_router: Some("swaps_router_addr".to_string()),
        fundraising_contract: None,
        accepted_tokens: None,
        halo_token_lp_contract: None,
        donation_match_charites_contract: None,
        collector_addr: None,
        collector_share: None,
        swap_factory: None,
        subdao_gov_code: None,
        subdao_cw20_token_code: None,
        subdao_bonding_token_code: None,
        subdao_cw900_code: None,
        subdao_distributor_code: None,
        donation_match_code: None,
        accounts_settings_controller: Some("accounts-settings-controller".to_string()),
    };
    let msg = ExecuteMsg::UpdateConfig(update_config_message);
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let update_config_message = UpdateConfigMsg {
        accounts_contract: Some("accounts_contract_addr".to_string()),
        index_fund_contract: Some(index_fund_contract.clone()),
        fundraising_contract: None,
        treasury: Some(ap_team.clone()),
        rebalance: Some(RebalanceDetails {
            rebalance_liquid_invested_profits: true,
            locked_interests_to_liquid: true,
            interest_distribution: Decimal::one(),
            locked_principle_to_liquid: true,
            principle_distribution: Decimal::one(),
        }),
        split_max: Some(Decimal::one()),
        split_min: Some(Decimal::zero()),
        split_default: Some(Decimal::percent(30)),
        charity_shares_contract: None,
        gov_contract: None,
        halo_token: None,
        halo_token_lp_contract: None,
        cw3_code: Some(MOCK_CW3_CODE_ID),
        cw4_code: Some(MOCK_CW4_CODE_ID),
        accepted_tokens: None,
        donation_match_charites_contract: None,
        collector_addr: None,
        collector_share: None,
        swap_factory: None,
        subdao_gov_code: None,
        subdao_cw20_token_code: None,
        subdao_bonding_token_code: None,
        subdao_cw900_code: None,
        subdao_distributor_code: None,
        donation_match_code: None,
        applications_review: Some(REVIEW_TEAM.to_string()),
        swaps_router: Some("swaps_router_addr".to_string()),
        accounts_settings_controller: Some("accounts-settings-controller".to_string()),
    };
    let msg = ExecuteMsg::UpdateConfig(update_config_message);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        index_fund_contract.clone(),
        config_response.index_fund.unwrap()
    );
    assert_eq!(MOCK_CW3_CODE_ID, config_response.cw3_code.unwrap());
    assert_eq!(MOCK_CW4_CODE_ID, config_response.cw4_code.unwrap());
}

#[test]
fn test_add_update_and_remove_strategy() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Add the mocked network_info
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        chain_id: CHAIN_ID.to_string(),
        network_info: NetworkInfo {
            router_contract: None,
            accounts_contract: Some("accounts_contract_addr".to_string()),
        },
        action: "post".to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, add_network_info_msg).unwrap();

    let add_strategy_msg = ExecuteMsg::StrategyAdd {
        strategy_key: STRATEGY_KEY.to_string(),
        strategy: StrategyParams {
            approval_state: StrategyApprovalState::Approved,
            locale: StrategyLocale::Native,
            chain: CHAIN_ID.to_string(),
            input_denom: USDC.to_string(),
            locked_addr: Some(Addr::unchecked("vault1-locked-contract")),
            liquid_addr: Some(Addr::unchecked("vault1-liquid-contract")),
        },
    };

    let update_strategy_msg = ExecuteMsg::StrategyUpdate {
        strategy_key: STRATEGY_KEY.to_string(),
        approval_state: StrategyApprovalState::WithdrawOnly,
    };

    let remove_strategy_msg = ExecuteMsg::StrategyRemove {
        strategy_key: STRATEGY_KEY.to_string(),
    };

    // Only contract owner can add/remove/update a strategy
    let info = mock_info("anyone", &coins(1000, "earth"));
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        add_strategy_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        update_strategy_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        remove_strategy_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // add strategy
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = execute(deps.as_mut(), mock_env(), info, add_strategy_msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Strategy {
            strategy_key: STRATEGY_KEY.to_string(),
        },
    )
    .unwrap();
    let strategy_detail_response: StrategyDetailResponse = from_binary(&res).unwrap();
    assert_eq!(
        StrategyApprovalState::Approved,
        strategy_detail_response.strategy.approval_state
    );

    // Cannot add strategies twice with same key
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let err = execute(deps.as_mut(), mock_env(), info, add_strategy_msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::StrategyAlreadyExists {});

    // update strategy status
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::StrategyUpdate {
        strategy_key: STRATEGY_KEY.to_string(),
        approval_state: StrategyApprovalState::WithdrawOnly,
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Strategy {
            strategy_key: STRATEGY_KEY.to_string(),
        },
    )
    .unwrap();
    let strategy_detail_response: StrategyDetailResponse = from_binary(&res).unwrap();
    assert_eq!(
        StrategyApprovalState::WithdrawOnly,
        strategy_detail_response.strategy.approval_state
    );

    // remove strategy
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::StrategyRemove {
        strategy_key: STRATEGY_KEY.to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Strategy {
            strategy_key: STRATEGY_KEY.to_string(),
        },
    )
    .unwrap_err();
    assert_ne!(
        err,
        StdError::NotFound {
            kind: "NetworkInfo".to_string()
        }
    );
}

#[test]
fn test_add_update_and_remove_accepted_tokens() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_response.accepted_tokens.native.len(), 1);
    assert_eq!(config_response.accepted_tokens.cw20.len(), 0);

    // add new token denom "new_token" to "accepted_tokens"
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let update_config_msg = UpdateConfigMsg {
        accounts_contract: None,
        index_fund_contract: None,
        fundraising_contract: None,
        treasury: None,
        rebalance: None,
        split_max: None,
        split_min: None,
        split_default: None,
        charity_shares_contract: None,
        gov_contract: None,
        halo_token: None,
        halo_token_lp_contract: None,
        cw3_code: None,
        cw4_code: None,
        accepted_tokens: Some(AcceptedTokens {
            native: vec!["new_token".to_string()],
            cw20: vec!["terraFloki4Life".to_string()],
        }),
        donation_match_charites_contract: None,
        collector_addr: None,
        collector_share: None,
        swap_factory: None,
        subdao_gov_code: None,
        subdao_cw20_token_code: None,
        subdao_bonding_token_code: None,
        subdao_cw900_code: None,
        subdao_distributor_code: None,
        donation_match_code: None,
        applications_review: Some(REVIEW_TEAM.to_string()),
        swaps_router: Some("swaps_router_addr".to_string()),
        accounts_settings_controller: Some("accounts-settings-controller".to_string()),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_response.accepted_tokens.native.len(), 1);
    assert_eq!(config_response.accepted_tokens.cw20.len(), 1);
}

#[test]
fn test_add_update_and_remove_network_infos() {
    let mock_network_info = NetworkInfo {
        router_contract: None, // router must exist if strategies exist on that chain
        accounts_contract: Some("accounts_contract_addr".to_string()), // accounts contract may exist if endowments are on that chain
    };

    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Should fail since NETWORK_CONNECTIONS is empty
    let _err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::NetworkConnection {
            chain_id: CHAIN_ID.to_string(),
        },
    )
    .unwrap_err();

    // Only owner can update the network info
    let info = mock_info("anyone", &coins(1000, "earth"));
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        chain_id: CHAIN_ID.to_string(),
        network_info: mock_network_info.clone(),
        action: "blahblah".to_string(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        add_network_info_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Add new network_info

    // Should fail since invalid action mode
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        chain_id: CHAIN_ID.to_string(),
        network_info: mock_network_info.clone(),
        action: "blahblah".to_string(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        add_network_info_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Succeed to add the network_info
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        chain_id: CHAIN_ID.to_string(),
        network_info: mock_network_info.clone(),
        action: "post".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, add_network_info_msg).unwrap();
    assert_eq!(1, res.attributes.len());

    // check the added network info
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::NetworkConnection {
            chain_id: CHAIN_ID.to_string(),
        },
    )
    .unwrap();
    let network_info_response: NetworkConnectionResponse = from_binary(&res).unwrap();
    assert_eq!(
        network_info_response.network_connection,
        mock_network_info.clone()
    );

    // Remove the network_info
    // Succeed to remove the network_info
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        chain_id: CHAIN_ID.to_string(),
        network_info: mock_network_info.clone(),
        action: "delete".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, add_network_info_msg).unwrap();
    assert_eq!(1, res.attributes.len());

    // check the added network info
    // should fail since removed
    let _err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::NetworkConnection {
            chain_id: CHAIN_ID.to_string(),
        },
    )
    .unwrap_err();
}

#[test]
fn test_update_fees() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = instantiate_msg();
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    // Only the config.owner can update the fees
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateFees {
            fees: vec![("new_fees_1".to_string(), Decimal::default())],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // The rates should be <= 100%
    let info = mock_info(ap_team.as_str(), &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateFees {
            fees: vec![(
                "new_fees_1".to_string(),
                Decimal::from_ratio(101_u128, 100_u128),
            )],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Suceed to update the fees
    let info = mock_info(ap_team.as_str(), &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateFees {
            fees: vec![(
                "new_fees_1".to_string(),
                Decimal::from_ratio(20_u128, 100_u128),
            )],
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 0);

    // Check the result
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Fee {
            name: "new_fees_1".to_string(),
        },
    )
    .unwrap();
    let fee: Decimal = from_binary(&res).unwrap();
    assert_eq!(fee, Decimal::from_ratio(20_u128, 100_u128));
}
