use std::marker::PhantomData;

use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};
use cw20::MinterResponse;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    minter_querier: MinterQuerier,
}

#[cw_serde]
pub struct MinterQuerier {
    minter_addr: String,
}

impl MinterQuerier {
    pub fn new(minter: String) -> Self {
        MinterQuerier {
            minter_addr: minter,
        }
    }
}

#[cw_serde]
pub enum QueryMsg {
    Minter {},
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
            }) => match from_binary(msg) {
                Ok(QueryMsg::Minter {}) => {
                    SystemResult::Ok(ContractResult::from(to_binary(&MinterResponse {
                        minter: self.minter_querier.minter_addr.clone(),
                        cap: None,
                    })))
                }
                _ => panic!("query not mocked"),
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            minter_querier: MinterQuerier::default(),
        }
    }

    pub fn with_halo_minter(&mut self, minter: String) {
        self.minter_querier = MinterQuerier::new(minter);
    }
}
