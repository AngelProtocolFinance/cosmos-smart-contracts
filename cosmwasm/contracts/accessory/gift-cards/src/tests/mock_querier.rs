use angel_core::msgs::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::{AcceptedTokens, RebalanceDetails, SplitDetails};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Api, Coin, ContractResult, Decimal, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};

use cosmwasm_schema::cw_serde;
use std::marker::PhantomData;

#[cw_serde]
pub enum QueryMsg {
    // Mock the `registrar::QueryMsg::Config {}` query
    Config {},
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
                        halo_token: Some("halo_token".to_string()),
                        gov_contract: Some("gov_contract".to_string()),
                        charity_shares_contract: Some("charity_shares".to_string()),
                        cw3_code: Some(2),
                        cw4_code: Some(3),
                        accepted_tokens: AcceptedTokens {
                            native: vec!["ujuno".to_string()],
                            cw20: vec!["test-cw20".to_string()],
                        },
                        applications_review: "applications-review".to_string(),
                        swaps_router: Some("swaps_router_addr".to_string()),
                        subdao_gov_code: None,
                        subdao_cw20_token_code: None,
                        subdao_bonding_token_code: None,
                        subdao_cw900_code: None,
                        subdao_distributor_code: None,
                        donation_match_code: None,
                        halo_token_lp_contract: None,
                        donation_match_charites_contract: None,
                        collector_addr: "collector-contract-addr".to_string(),
                        collector_share: Decimal::zero(),
                        swap_factory: None,
                        accounts_settings_controller: "accounts-settings-controller-addr"
                            .to_string(),
                        axelar_gateway: "axelar-gateway".to_string(),
                        axelar_ibc_channel: "channel-1".to_string(),
                    })
                    .unwrap(),
                )),
            },
            QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: _,
                key: _,
            }) => {
                panic!("DO NOT ENTER HERE")
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<Empty>, _api: A) -> Self {
        WasmMockQuerier { base }
    }
}
