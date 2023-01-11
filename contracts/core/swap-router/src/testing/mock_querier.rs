use angel_core::messages::dexs::InfoResponse;
use angel_core::responses::registrar::{ConfigResponse, VaultDetailResponse};
use angel_core::structs::{AcceptedTokens, AccountType, RebalanceDetails, SplitDetails, VaultType};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, BankQuery, Coin, ContractResult, Decimal, Empty,
    OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Denom};
use cw_asset::AssetInfo;
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
    Balance {
        address: String,
    },
    Info {},
    TokenAmount {
        id: u32,
        asset_info: AssetInfo,
        acct_type: AccountType,
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
            QueryRequest::Bank(BankQuery::Balance { address: _, denom }) => {
                SystemResult::Ok(ContractResult::Ok(
                    to_binary(&cosmwasm_std::BalanceResponse {
                        amount: Coin {
                            denom: denom.to_string(),
                            amount: Uint128::from(1000000_u128),
                        },
                    })
                    .unwrap(),
                ))
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_binary(&msg).unwrap() {
                QueryMsg::Balance { address: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&BalanceResponse {
                        balance: Uint128::from(1000000_u128),
                    })
                    .unwrap(),
                )),
                QueryMsg::Info {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&InfoResponse {
                        token1_reserve: Uint128::from(1000000_u128),
                        token1_denom: Denom::Native("usdc".to_string()),
                        token2_reserve: Uint128::from(1000000_u128),
                        token2_denom: Denom::Cw20(Addr::unchecked("asset0000")),
                        lp_token_supply: Uint128::from(1000000_u128),
                        lp_token_address: "contract-2".to_string(),
                    })
                    .unwrap(),
                )),
                QueryMsg::TokenAmount {
                    id: _,
                    asset_info: _,
                    acct_type: _,
                } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&Uint128::from(1000000_u128)).unwrap(),
                )),
                QueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&ConfigResponse {
                        owner: "registrar-owner".to_string(),
                        version: "v1.0".to_string(),
                        accounts_contract: None,
                        treasury: "treasury".to_string(),
                        rebalance: RebalanceDetails::default(),
                        index_fund: None,
                        split_to_liquid: SplitDetails::default(),
                        halo_token: None,
                        gov_contract: None,
                        charity_shares_contract: None,
                        cw3_code: None,
                        cw4_code: None,
                        accepted_tokens: AcceptedTokens::default(),
                        applications_review: "applications_review".to_string(),
                        swaps_router: None,
                        donation_match_charites_contract: Some(MOCK_CONTRACT_ADDR.to_string()),
                        collector_addr: "collector-addr".to_string(),
                        collector_share: Decimal::percent(50),
                        swap_factory: Some("swap-factory".to_string()),
                        settings_controller: "settings-controller".to_string(),
                        subdao_gov_code: None,
                        subdao_cw20_token_code: None,
                        subdao_bonding_token_code: None,
                        subdao_cw900_code: None,
                        subdao_distributor_code: None,
                        donation_match_code: None,
                        halo_token_lp_contract: None,
                    })
                    .unwrap(),
                )),
                QueryMsg::Vault { vault_addr: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&VaultDetailResponse {
                        vault: angel_core::structs::YieldVault {
                            address: "vault-1".to_string(),
                            network: "juno-1".to_string(),
                            input_denom: "ujuno".to_string(),
                            yield_token: "yield-token-contract".to_string(),
                            approved: true,
                            restricted_from: vec![],
                            acct_type: AccountType::Locked,
                            vault_type: VaultType::Native,
                        },
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
