use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, VaultDetailResponse, VaultListResponse,
};
use angel_core::responses::settings_controller::{
    EndowmentPermissionsResponse, EndowmentSettingsResponse,
};
use angel_core::structs::{
    AcceptedTokens, AccountType, EndowmentType, RebalanceDetails, SettingsController, SplitDetails,
    VaultType, YieldVault,
};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Coin, ContractResult, Decimal, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Vault {
        vault_addr: String,
    },
    VaultList {
        network: Option<String>,
        endowment_type: Option<EndowmentType>,
        acct_type: Option<AccountType>,
        vault_type: Option<VaultType>,
        approved: Option<bool>,
        start_after: Option<String>,
        limit: Option<u64>,
    },
    // Mock the "vault::balance { endowment_id: u32 }" query
    Balance {
        endowment_id: u32,
    },
    // Mock the "registrar::fee { name: String }" query
    Fee {
        name: String,
    },
    // Mock the "settings_controller::EndowmentSettings {id: [EndowmentID]}" query
    EndowmentSettings {
        id: u32,
    },
    // Mock the "settings_controller::EndowmentPermissions {id: [EndowmentID]}" query
    EndowmentPermissions {
        id: u32,
        setting_updater: Addr,
        endowment_owner: Addr,
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
                        owner: "registrar_owner".to_string(),
                        version: "0.1.0".to_string(),
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
                            native: vec!["ujuno".to_string()],
                            cw20: vec!["test-cw20".to_string()],
                        },
                        swap_factory: None,
                        applications_review: "applications-review".to_string(),
                        swaps_router: Some("swaps_router_addr".to_string()),
                        settings_controller: Some("settings-controller".to_string()),
                    })
                    .unwrap(),
                )),
                QueryMsg::Vault { vault_addr } => {
                    if let "liquid-vault" = vault_addr.as_str() {
                        SystemResult::Ok(ContractResult::Ok(
                            to_binary(&VaultDetailResponse {
                                vault: YieldVault {
                                    network: "juno".to_string(),
                                    address: Addr::unchecked("liquid-vault").to_string(),
                                    input_denom: "input-denom".to_string(),
                                    yield_token: Addr::unchecked("yield-token").to_string(),
                                    approved: true,
                                    restricted_from: vec![],
                                    acct_type: AccountType::Liquid,
                                    vault_type: VaultType::Native,
                                },
                            })
                            .unwrap(),
                        ))
                    } else {
                        SystemResult::Ok(ContractResult::Ok(
                            to_binary(&VaultDetailResponse {
                                vault: YieldVault {
                                    network: "juno".to_string(),
                                    address: Addr::unchecked("vault").to_string(),
                                    input_denom: "input-denom".to_string(),
                                    yield_token: Addr::unchecked("yield-token").to_string(),
                                    approved: true,
                                    restricted_from: vec![],
                                    acct_type: AccountType::Locked,
                                    vault_type: VaultType::Native,
                                },
                            })
                            .unwrap(),
                        ))
                    }
                }
                QueryMsg::VaultList {
                    network: _,
                    endowment_type: _,
                    acct_type: Some(AccountType::Locked),
                    vault_type: _,
                    approved: _,
                    start_after: _,
                    limit: _,
                } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&VaultListResponse {
                        vaults: vec![
                            YieldVault {
                                address: Addr::unchecked("vault").to_string(),
                                network: "juno-1".to_string(),
                                input_denom: "input-denom".to_string(),
                                yield_token: Addr::unchecked("yield-token").to_string(),
                                approved: true,
                                restricted_from: vec![],
                                acct_type: AccountType::Locked,
                                vault_type: VaultType::Native,
                            },
                            YieldVault {
                                address: Addr::unchecked("tech_strategy_component_addr")
                                    .to_string(),
                                network: "juno-1".to_string(),
                                input_denom: "input-denom".to_string(),
                                yield_token: Addr::unchecked("yield-token").to_string(),
                                approved: true,
                                restricted_from: vec![],
                                acct_type: AccountType::Locked,
                                vault_type: VaultType::Native,
                            },
                        ],
                    })
                    .unwrap(),
                )),
                QueryMsg::VaultList {
                    network: _,
                    endowment_type: _,
                    acct_type: Some(AccountType::Liquid),
                    vault_type: _,
                    approved: _,
                    start_after: _,
                    limit: _,
                } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&VaultListResponse {
                        vaults: vec![YieldVault {
                            address: Addr::unchecked("cash_strategy_component_addr").to_string(),
                            network: "juno-1".to_string(),
                            input_denom: "input-denom".to_string(),
                            yield_token: Addr::unchecked("yield-token").to_string(),
                            approved: true,
                            restricted_from: vec![],
                            acct_type: AccountType::Liquid,
                            vault_type: VaultType::Native,
                        }],
                    })
                    .unwrap(),
                )),
                QueryMsg::VaultList {
                    network: _,
                    endowment_type: _,
                    acct_type: _,
                    vault_type: _,
                    approved: _,
                    start_after: _,
                    limit: _,
                } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&VaultListResponse {
                        vaults: vec![
                            YieldVault {
                                address: Addr::unchecked("vault").to_string(),
                                network: "juno-1".to_string(),
                                input_denom: "input-denom".to_string(),
                                yield_token: Addr::unchecked("yield-token").to_string(),
                                approved: true,
                                restricted_from: vec![],
                                acct_type: AccountType::Locked,
                                vault_type: VaultType::Native,
                            },
                            YieldVault {
                                address: Addr::unchecked("cash_strategy_component_addr")
                                    .to_string(),
                                network: "juno-1".to_string(),
                                input_denom: "input-denom".to_string(),
                                yield_token: Addr::unchecked("yield-token").to_string(),
                                approved: true,
                                restricted_from: vec![],
                                acct_type: AccountType::Liquid,
                                vault_type: VaultType::Native,
                            },
                            YieldVault {
                                address: Addr::unchecked("tech_strategy_component_addr")
                                    .to_string(),
                                network: "juno-1".to_string(),
                                input_denom: "input-denom".to_string(),
                                yield_token: Addr::unchecked("yield-token").to_string(),
                                approved: true,
                                restricted_from: vec![],
                                acct_type: AccountType::Locked,
                                vault_type: VaultType::Native,
                            },
                        ],
                    })
                    .unwrap(),
                )),
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
                            "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v", // CHARITY_ADDR
                        )],
                        earnings_fee: None,
                        withdraw_fee: None,
                        deposit_fee: None,
                        aum_fee: None,
                        settings_controller: SettingsController::default(),
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
                        settings_controller: false,
                        strategies: false,
                        beneficiaries_allowlist: false,
                        contributors_allowlist: false,
                        maturity_time: false,
                        profile: false,
                        earnings_fee: false,
                        withdraw_fee: false,
                        deposit_fee: false,
                        aum_fee: false,
                        kyc_donors_only: false,
                        name: false,
                        image: false,
                        logo: false,
                        categories: false,
                    })
                    .unwrap(),
                )),
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
