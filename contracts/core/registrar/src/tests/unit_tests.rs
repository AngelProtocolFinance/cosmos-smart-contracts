use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;
use angel_core::messages::registrar::*;
use angel_core::responses::registrar::*;
use angel_core::structs::{
    AcceptedTokens, AccountType, NetworkInfo, RebalanceDetails, SplitDetails, VaultType,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_binary, Decimal, StdError};

const MOCK_CW3_CODE_ID: u64 = 18;
const MOCK_CW4_CODE_ID: u64 = 19;
const AP_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const REVIEW_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65xxxx";

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(20),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

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
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(20),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
    };
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
    let index_fund_contract = String::from("terra1typpfzq9ynmvrt6tt459epfqn4gqejhy6lmu7d");
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(0),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let update_config_message = UpdateConfigMsg {
        accounts_contract: Some("accounts_contract_addr".to_string()),
        index_fund_contract: Some(index_fund_contract.clone()),
        fundraising_contract: None,
        approved_charities: None,
        treasury: Some(ap_team.clone()),
        tax_rate: None,
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
        accepted_tokens_cw20: None,
        accepted_tokens_native: None,
        applications_review: Some(REVIEW_TEAM.to_string()),
        swaps_router: Some("swaps_router_addr".to_string()),
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
fn test_add_update_and_remove_vault() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let vault_addr = "terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg".to_string();
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(20),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // add vault
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let add_vault_message = VaultAddMsg {
        acct_type: AccountType::Locked,
        vault_type: VaultType::Native,
        network: None,
        vault_addr: vault_addr.clone(),
        input_denom: String::from("input_denom"),
        yield_token: String::from("yield_token"),
        restricted_from: vec![],
    };
    let msg = ExecuteMsg::VaultAdd(add_vault_message);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Vault {
            vault_addr: vault_addr.clone(),
        },
    )
    .unwrap();
    let vault_detail_response: VaultDetailResponse = from_binary(&res).unwrap();
    assert_eq!(vault_addr.clone(), vault_detail_response.vault.address);
    assert_eq!(true, vault_detail_response.vault.approved);

    // update vault status
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::VaultUpdate {
        vault_addr: String::from("terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg"),
        approved: false,
        restricted_from: vec![],
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Vault {
            vault_addr: vault_addr.clone(),
        },
    )
    .unwrap();
    let vault_detail_response: VaultDetailResponse = from_binary(&res).unwrap();
    assert_eq!(vault_addr.clone(), vault_detail_response.vault.address);
    assert_eq!(false, vault_detail_response.vault.approved);

    // remove vault
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::VaultRemove {
        vault_addr: vault_addr.clone(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Vault {
            vault_addr: vault_addr.clone(),
        },
    )
    .unwrap_err();
    assert_ne!(
        err,
        StdError::NotFound {
            kind: "YieldVault".to_string()
        }
    );
}

#[test]
fn test_add_update_and_remove_accepted_tokens() {
    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(20),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(config_response.accepted_tokens.native.len(), 2);
    assert_eq!(config_response.accepted_tokens.cw20.len(), 0);

    // add new token denom "new_token" to "accepted_tokens"
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let update_config_msg = UpdateConfigMsg {
        accounts_contract: None,
        index_fund_contract: None,
        fundraising_contract: None,
        approved_charities: None,
        treasury: None,
        tax_rate: None,
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
        accepted_tokens_cw20: Some(vec!["new_token".to_string()]),
        accepted_tokens_native: None,
        applications_review: Some(REVIEW_TEAM.to_string()),
        swaps_router: Some("swaps_router_addr".to_string()),
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
        name: "juno mainnet".to_string(),
        chain_id: "juno-1".to_string(),
        ibc_channel: None,
        ibc_host_contract: None,
        gas_limit: None,
    };

    let mut deps = mock_dependencies();
    let ap_team = AP_TEAM.to_string();
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(20),
        rebalance: None,
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Should fail since NETWORK_CONNECTIONS is empty
    let _err = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::NetworkConnection {
            chain_id: mock_network_info.chain_id.to_string(),
        },
    )
    .unwrap_err();

    // Add new network_info

    // Should fail since invalid action mode
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
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
            chain_id: mock_network_info.chain_id.to_string(),
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
            chain_id: mock_network_info.chain_id.to_string(),
        },
    )
    .unwrap_err();
}

#[test]
fn test_update_endow_type_fees() {
    let mut deps = mock_dependencies();
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let instantiate_msg = InstantiateMsg {
        treasury: ap_team.clone(),
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "ujuno".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
        swap_factory: None,
        rebalance: None,
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let update_endow_type_fees_msg = UpdateEndowTypeFeesMsg {
        endowtype_charity: Some(Decimal::MAX),
        endowtype_normal: Some(Decimal::one()),
    };

    // non-config_owner failes to update endowment_type_fees
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowTypeFees(update_endow_type_fees_msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // config_owner succeeds to update endowment type fees
    let info = mock_info(&ap_team, &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateEndowTypeFees(update_endow_type_fees_msg),
    )
    .unwrap();

    // Query the "endow_type_fees"
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Fees {}).unwrap();
    let fees: FeesResponse = from_binary(&res).unwrap();
    assert_eq!(fees.endowtype_charity, Some(Decimal::MAX));
    assert_eq!(fees.endowtype_normal, Some(Decimal::one()));
}
