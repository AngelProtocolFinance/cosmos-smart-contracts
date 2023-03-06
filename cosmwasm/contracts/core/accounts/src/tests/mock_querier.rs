use angel_core::msgs::accounts_settings_controller::{
    EndowmentPermissionsResponse, EndowmentSettingsResponse,
};
use angel_core::msgs::registrar::{
    ConfigResponse as RegistrarConfigResponse, NetworkConnectionResponse, StrategyDetailResponse,
};
use angel_core::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails, StrategyApprovalState,
    StrategyLocale, StrategyParams,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Coin, ContractResult, Decimal, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::marker::PhantomData;

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Strategy {
        strategy_key: String,
    },
    // Mock the "vault::balance { endowment_id: u32 }" query
    Balance {
        endowment_id: u32,
    },
    // Mock the "registrar::fee { name: String }" query
    Fee {
        name: String,
    },
    // Mock the "endowment_controller::EndowmentSettings {id: [EndowmentID]}" query
    EndowmentSettings {
        id: u32,
    },
    // Mock the "endowment_controller::EndowmentPermissions {id: [EndowmentID]}" query
    EndowmentPermissions {
        id: u32,
        setting_updater: Addr,
        endowment_owner: Addr,
    },
    // Mock Network Connection from Registrar for an EVM and Native chain
    NetworkConnection {
        chain_id: String,
    },
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(
        MockQuerier::new(&[(&contract_addr, contract_balance)]),
        MockApi::default(),
    );
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_binary(&msg).unwrap() {
                QueryMsg::Balance { endowment_id: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&Uint128::from(1000000_u128)).unwrap(),
                )),
                QueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&RegistrarConfigResponse {
                        owner: "juno1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly".to_string(), // APT TEAM ADDR
                        version: "registrar-0.1.0".to_string(),
                        accounts_contract: Some("accounts_contract_addr".to_string()),
                        treasury: "treasury".to_string(),
                        rebalance: RebalanceDetails::default(),
                        index_fund: Some("index_fund".to_string()),
                        split_to_liquid: SplitDetails {
                            min: Decimal::zero(),
                            max: Decimal::one(),
                            default: Decimal::percent(50),
                        },
                        subdao_gov_code: None,
                        subdao_cw20_token_code: Some(4_u64),
                        subdao_bonding_token_code: Some(3_u64),
                        subdao_cw900_code: None,
                        subdao_distributor_code: None,
                        donation_match_code: None,
                        donation_match_charites_contract: None,
                        collector_addr: "collector-addr".to_string(),
                        collector_share: Decimal::one(),
                        halo_token: Some("halo_token".to_string()),
                        halo_token_lp_contract: Some("halo_token_lp_contract".to_string()),
                        gov_contract: Some("gov_contract".to_string()),
                        charity_shares_contract: Some("charity_shares".to_string()),
                        cw3_code: Some(2),
                        cw4_code: Some(3),
                        accepted_tokens: AcceptedTokens {
                            native: vec!["ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string()],
                            cw20: vec!["test-cw20".to_string()],
                        },
                        swap_factory: None,
                        applications_review: "applications-review".to_string(),
                        swaps_router: Some("swaps_router_addr".to_string()),
                        accounts_settings_controller: "accounts-settings-controller".to_string(),
                        axelar_gateway: "axelar-gateway".to_string(),
                        axelar_ibc_channel: "channel-1".to_string(),
                    })
                    .unwrap(),
                )),
                QueryMsg::Strategy { strategy_key } => match strategy_key.as_str() {
                    "strategy-native" => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&StrategyDetailResponse {
                            strategy: StrategyParams {
                                approval_state: StrategyApprovalState::Approved,
                                locale: StrategyLocale::Native,
                                chain: "juno".to_string(),
                                input_denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                locked_addr: Some(Addr::unchecked("vault1-locked-contract")),
                                liquid_addr: Some(Addr::unchecked("vault1-liquid-contract")),
                            },
                        })
                        .unwrap(),
                    )),
                    "strategy-ethereum" => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&StrategyDetailResponse {
                            strategy: StrategyParams {
                                approval_state: StrategyApprovalState::Approved,
                                locale: StrategyLocale::Evm,
                                chain: "ethereum".to_string(),
                                input_denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                locked_addr: None,
                                liquid_addr: None,
                            },
                        })
                        .unwrap(),
                    )),
                    "shady-strategy" => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&StrategyDetailResponse {
                            strategy: StrategyParams {
                                approval_state: StrategyApprovalState::NotApproved,
                                locale: StrategyLocale::Ibc,
                                chain: "injective".to_string(),
                                input_denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                locked_addr: None,
                                liquid_addr: None,
                            },
                        })
                        .unwrap(),
                    )),
                    "wrong-chain-strategy" => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&StrategyDetailResponse {
                            strategy: StrategyParams {
                                approval_state: StrategyApprovalState::Approved,
                                locale: StrategyLocale::Ibc,
                                chain: "injective".to_string(),
                                input_denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                locked_addr: None,
                                liquid_addr: None,
                            },
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&StrategyDetailResponse {
                            strategy: StrategyParams {
                                approval_state: StrategyApprovalState::Deprecated,
                                locale: StrategyLocale::Ibc,
                                chain: "injective".to_string(),
                                input_denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                locked_addr: None,
                                liquid_addr: None,
                            },
                        })
                        .unwrap(),
                    )),
                },
                QueryMsg::Fee { name: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&Decimal::from_ratio(10_u128, 100_u128)).unwrap(),
                )),
                QueryMsg::EndowmentSettings { id: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&EndowmentSettingsResponse {
                        dao: None,
                        dao_token: None,
                        donation_match_active: false,
                        donation_match_contract: Some(Addr::unchecked("donation-match-contract")),
                        beneficiaries_allowlist: vec![],
                        contributors_allowlist: vec![],
                        maturity_allowlist: vec![Addr::unchecked(
                            "juno1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v", // CHARITY_ADDR
                        )],
                        earnings_fee: None,
                        withdraw_fee: None,
                        deposit_fee: None,
                        aum_fee: None,
                        parent: None,
                        split_to_liquid: None,
                        ignore_user_splits: false,
                    })
                    .unwrap(),
                )),
                QueryMsg::EndowmentPermissions {
                    id: _,
                    setting_updater: _,
                    endowment_owner: _,
                } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&EndowmentPermissionsResponse {
                        endowment_controller: false,
                        strategies: false,
                        beneficiaries_allowlist: false,
                        contributors_allowlist: false,
                        maturity_allowlist: false,
                        earnings_fee: false,
                        withdraw_fee: false,
                        deposit_fee: false,
                        aum_fee: false,
                        kyc_donors_only: false,
                        name: false,
                        image: false,
                        logo: false,
                        categories: false,
                        ignore_user_splits: false,
                        split_to_liquid: false,
                    })
                    .unwrap(),
                )),
                QueryMsg::NetworkConnection { chain_id } => match chain_id.as_str() {
                    // EVM chain (w/o Accounts contract)
                    "ethereum" => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&NetworkConnectionResponse {
                            chain: chain_id,
                            network_connection: NetworkInfo {
                                router_contract: Some("vault-router".to_string()),
                                accounts_contract: None,
                            },
                        })
                        .unwrap(),
                    )),
                    // Native cosmos chain (w/ Vaults)
                    "juno" => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&NetworkConnectionResponse {
                            chain: chain_id,
                            network_connection: NetworkInfo {
                                router_contract: Some("vault-router".to_string()),
                                accounts_contract: Some("accounts_contract_addr".to_string()),
                            },
                        })
                        .unwrap(),
                    )),
                    // Some chain (w/o Vaults)
                    _ => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&NetworkConnectionResponse {
                            chain: chain_id,
                            network_connection: NetworkInfo {
                                router_contract: None,
                                accounts_contract: Some("accounts_contract_addr".to_string()),
                            },
                        })
                        .unwrap(),
                    )),
                },
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<Empty>, _api: A) -> Self {
        WasmMockQuerier { base }
    }
}
