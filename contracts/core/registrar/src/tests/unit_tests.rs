use crate::contract::{execute, instantiate, migrate, query, reply};
use angel_core::errors::core::*;
use angel_core::messages::cw3_multisig::Threshold;
use angel_core::messages::registrar::*;
use angel_core::responses::registrar::*;
use angel_core::structs::{AcceptedTokens, SplitDetails};
use angel_core::structs::{EndowmentStatus, EndowmentType, NetworkInfo, Profile, SocialMedialUrls};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    coins, from_binary, Addr, CosmosMsg, Decimal, Event, Reply, SubMsgResponse, SubMsgResult,
    WasmMsg,
};

const MOCK_ACCOUNTS_CODE_ID: u64 = 17;
const MOCK_CW3_CODE_ID: u64 = 18;
const MOCK_CW4_CODE_ID: u64 = 19;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(MOCK_ACCOUNTS_CODE_ID, config_response.accounts_code_id);
    assert_eq!(ap_team.clone(), config_response.owner);
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies();
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
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
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let index_fund_contract = String::from("terra1typpfzq9ynmvrt6tt459epfqn4gqejhy6lmu7d");
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(0),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let _res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let update_config_message = UpdateConfigMsg {
        accounts_code_id: None,
        index_fund_contract: Some(index_fund_contract.clone()),
        approved_charities: None,
        treasury: Some(ap_team.clone()),
        tax_rate: None,
        default_vault: None,
        split_max: Some(Decimal::one()),
        split_min: Some(Decimal::zero()),
        split_default: Some(Decimal::percent(30)),
        charity_shares_contract: None,
        gov_contract: None,
        halo_token: None,
        cw3_code: Some(MOCK_CW3_CODE_ID),
        cw4_code: Some(MOCK_CW4_CODE_ID),
        accepted_tokens_cw20: None,
        accepted_tokens_native: None,
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
    assert_eq!(MOCK_ACCOUNTS_CODE_ID, config_response.accounts_code_id);
    assert_eq!(MOCK_CW3_CODE_ID, config_response.cw3_code.unwrap());
    assert_eq!(MOCK_CW4_CODE_ID, config_response.cw4_code.unwrap());
}

#[test]
fn test_owner_can_add_remove_approved_charities() {
    let mut deps = mock_dependencies();
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let pleb = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn anyone_can_create_endowment_accounts_and_then_update() {
    let mut deps = mock_dependencies();
    // meet the cast of characters
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let good_charity_addr = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v".to_string();
    let good_endowment_addr = "terra1glqvyurcm6elnw2wl90kwlhtzrd2zc7q00prc9".to_string();
    let default_vault_addr = "terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg".to_string();
    let index_fund_contract = "terra1typpfzq9ynmvrt6tt459epfqn4gqejhy6lmu7d".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: Some(Addr::unchecked(default_vault_addr)),
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Config the "index_fund_contract" to avoid the "ContractNotConfigured" error.
    let update_config_msg = UpdateConfigMsg {
        accounts_code_id: None,
        index_fund_contract: Some(index_fund_contract.clone()),
        approved_charities: None,
        treasury: None,
        tax_rate: None,
        default_vault: None,
        split_max: None,
        split_min: None,
        split_default: None,
        charity_shares_contract: None,
        gov_contract: None,
        halo_token: None,
        cw3_code: None,
        cw4_code: None,
        accepted_tokens_cw20: None,
        accepted_tokens_native: None,
    };
    let info = mock_info(ap_team.as_ref(), &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::UpdateConfig(update_config_msg),
    )
    .unwrap();

    let profile: Profile = Profile {
        name: "Test Endowment".to_string(),
        overview: "Endowment to power an amazing charity".to_string(),
        un_sdg: None,
        tier: None,
        logo: None,
        image: None,
        url: None,
        registration_number: None,
        country_of_origin: None,
        street_address: None,
        contact_email: None,
        social_media_urls: SocialMedialUrls {
            facebook: None,
            twitter: None,
            linkedin: None,
        },
        number_of_employees: None,
        average_annual_budget: None,
        annual_revenue: None,
        charity_navigator_rating: None,
        endow_type: EndowmentType::Charity,
    };

    let create_endowment_msg = CreateEndowmentMsg {
        owner: good_charity_addr.clone(),
        beneficiary: good_charity_addr.clone(),
        withdraw_before_maturity: false,
        maturity_time: None,
        maturity_height: None,
        profile: profile,
        cw4_members: vec![],
        kyc_donors_only: false,
        cw3_multisig_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_multisig_max_vote_period: 60,
    };

    // anyone can create Accounts
    let info = mock_info(good_charity_addr.as_ref(), &coins(100000, "earth"));
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreateEndowment(create_endowment_msg.clone()),
    )
    .unwrap();
    assert_eq!(1, res.messages.len());

    // test that with the submessage we can instantiate account sc
    let msg: &CosmosMsg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Wasm(wasm_msg) => {
            match wasm_msg {
                WasmMsg::Instantiate {
                    admin,
                    code_id: _,
                    msg,
                    funds: _,
                    label: _,
                } => {
                    assert_eq!(admin.clone(), Some(ap_team.clone()));
                    let accounts_instantiate_msg: angel_core::messages::accounts::InstantiateMsg =
                        from_binary(msg).unwrap();
                    assert_eq!(accounts_instantiate_msg.owner_sc, ap_team.clone());

                    // let's instantiate account sc with our sub_message
                    let mut deps = mock_dependencies();
                    let info = mock_info("creator", &coins(100000, "earth"));
                    let env = mock_env();

                    // for now we have instantiation error due to another submsg call
                    // from the accounts sc instantiate method
                    // but the instantiation message work well
                    //
                    // TODO: fix test when accounts sc instantiate test will be ready
                    // by removing let err = ... and changing to:
                    // assert_eq!(0, res.messages.len());
                    let err = accounts::contract::instantiate(
                        deps.as_mut(),
                        env,
                        info,
                        accounts_instantiate_msg,
                    )
                    .unwrap_err();
                    match err {
                        ContractError::Std(err) => match err {
                            cosmwasm_std::StdError::GenericErr { msg } => {
                                assert_eq!(
                                    msg,
                                    "Querier system error: No such contract: cosmos2contract"
                                );
                                ()
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                    ()
                }
                _ => {
                    panic!("Not the Wasm instaniation message");
                }
            }
        }
        _ => {
            panic!("Not the Cosmos message");
        }
    }

    assert_eq!(1, res.attributes.len());
    assert_eq!("action", res.attributes[0].key);
    assert_eq!("create_endowment", res.attributes[0].value);

    let events = vec![Event::new("wasm")
        .add_attribute("endow_addr", good_endowment_addr.clone())
        .add_attribute("endow_name", "Test Endowment".to_string())
        .add_attribute("endow_owner", good_charity_addr.clone())
        .add_attribute("endow_type", "charity".to_string())
        .add_attribute("endow_logo", "Test logo".to_string())
        .add_attribute("endow_image", "Test image".to_string())];
    let result = SubMsgResult::Ok(SubMsgResponse { events, data: None });
    let subcall = Reply { id: 0, result };

    // test the reply method
    let res = reply(deps.as_mut(), mock_env(), subcall).unwrap();
    assert_eq!(0, res.messages.len());

    // test that the reply worked properly by querying
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentList {
            name: None,
            owner: None,
            status: None,
            tier: None,
            endow_type: None,
            un_sdg: None,
        },
    )
    .unwrap();
    let endowment_list_response: EndowmentListResponse = from_binary(&res).unwrap();
    assert_eq!(
        endowment_list_response.endowments[0].address,
        Addr::unchecked(good_endowment_addr.clone())
    );
    assert_eq!(
        endowment_list_response.endowments[0].status,
        EndowmentStatus::Inactive
    );

    // let's test update endowment method by admin
    let update_endowment_entry_msg = UpdateEndowmentEntryMsg {
        endowment_addr: good_endowment_addr.clone(),
        name: None,
        owner: None,
        tier: None,
        endow_type: None,
        un_sdg: None,
        logo: None,
        image: None,
    };

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateEndowmentEntry(update_endowment_entry_msg.clone()),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let update_endowment_status_msg = UpdateEndowmentStatusMsg {
        endowment_addr: good_endowment_addr.clone(),
        status: 1,
        beneficiary: None,
    };

    let info = mock_info(ap_team.as_ref(), &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateEndowmentStatus(update_endowment_status_msg.clone()),
    )
    .unwrap();
    assert_eq!(1, res.messages.len());

    // test that the updating worked properly by querying
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::EndowmentList {
            name: None,
            owner: None,
            status: None,
            tier: None,
            endow_type: None,
            un_sdg: None,
        },
    )
    .unwrap();
    let endowment_list_response: EndowmentListResponse = from_binary(&res).unwrap();
    assert_eq!(
        endowment_list_response.endowments[0].address,
        Addr::unchecked(good_endowment_addr.clone())
    );
    assert_eq!(
        endowment_list_response.endowments[0].status,
        EndowmentStatus::Approved
    );
}

#[test]
fn test_add_update_and_remove_vault() {
    let mut deps = mock_dependencies();
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let vault_addr = "terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
    };
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // add vault
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let add_vault_message = VaultAddMsg {
        network: None,
        vault_addr: vault_addr.clone(),
        input_denom: String::from("input_denom"),
        yield_token: String::from("yield_token"),
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
    assert_eq!(false, vault_detail_response.vault.approved);

    // update vault status
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let msg = ExecuteMsg::VaultUpdateStatus {
        vault_addr: String::from("terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg"),
        approved: true,
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
    assert_eq!(true, vault_detail_response.vault.approved);
}

#[test]
fn test_add_update_and_remove_accepted_tokens() {
    let mut deps = mock_dependencies();
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
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
        accounts_code_id: None,
        index_fund_contract: None,
        approved_charities: None,
        treasury: None,
        tax_rate: None,
        default_vault: None,
        split_max: None,
        split_min: None,
        split_default: None,
        charity_shares_contract: None,
        gov_contract: None,
        halo_token: None,
        cw3_code: None,
        cw4_code: None,
        accepted_tokens_cw20: Some(vec!["new_token".to_string()]),
        accepted_tokens_native: None,
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
    assert_eq!(config_response.accepted_tokens.native.len(), 2);
    assert_eq!(config_response.accepted_tokens.cw20.len(), 1);
}

#[test]
fn test_add_update_and_remove_network_infos() {
    let mock_network_info = NetworkInfo {
        name: "juno mainnet".to_string(),
        chain_id: "juno-1".to_string(),
        ibc_channel: None,
        gas_limit: None,
    };

    let mut deps = mock_dependencies();
    let ap_team = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string();
    let instantiate_msg = InstantiateMsg {
        accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
        treasury: ap_team.clone(),
        default_vault: None,
        tax_rate: Decimal::percent(20),
        split_to_liquid: Some(SplitDetails::default()),
        accepted_tokens: Some(AcceptedTokens {
            native: vec![
                "uluna".to_string(),
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
            cw20: vec![],
        }),
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
        action: "add".to_string(),
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

    // Should fail since invalid action mode
    let info = mock_info(ap_team.as_ref(), &coins(1000, "earth"));
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        network_info: mock_network_info.clone(),
        action: "wowo".to_string(),
    };
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        add_network_info_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Succeed to remove the network_info
    let add_network_info_msg = ExecuteMsg::UpdateNetworkConnections {
        network_info: mock_network_info.clone(),
        action: "remove".to_string(),
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
