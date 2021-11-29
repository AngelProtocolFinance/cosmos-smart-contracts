use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Coin, Empty, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, WasmQuery,
};
use halo_lbp::asset::PairInfo;
use halo_lbp::pair::QueryMsg;
use std::collections::HashMap;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    halo_lbp_pair_querier: HaloLBPPairQuerier,
}

#[derive(Clone, Default)]
pub struct HaloLBPPairQuerier {
    pairs: HashMap<Addr, PairInfo>,
}

impl HaloLBPPairQuerier {
    pub fn new(pairs: &[(&Addr, &PairInfo)]) -> Self {
        HaloLBPPairQuerier {
            pairs: pairs_to_map(pairs),
        }
    }
}

pub(crate) fn pairs_to_map(pairs: &[(&Addr, &PairInfo)]) -> HashMap<Addr, PairInfo> {
    let mut pairs_map: HashMap<Addr, PairInfo> = HashMap::new();
    for (key, pair) in pairs.iter() {
        pairs_map.insert((*key).clone(), (*pair).clone());
    }
    pairs_map
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
            QueryRequest::Wasm(WasmQuery::Smart {contract_addr, msg})// => {
                => match from_binary(&msg).unwrap() {
                    QueryMsg::Pair {} => {
                       let pair_info: PairInfo =
                        match self.halo_lbp_pair_querier.pairs.get(&Addr::unchecked(contract_addr)) {
                            Some(v) => v.clone(),
                            None => {
                                return SystemResult::Err(SystemError::NoSuchContract {
                                    addr: contract_addr.clone(),
                                })
                            }
                        };

                        SystemResult::Ok(to_binary(&pair_info).into())
                    }
                    _ => panic!("DO NOT ENTER HERE")
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            halo_lbp_pair_querier: HaloLBPPairQuerier::default(),
        }
    }

    // configure the halo-lbp pair
    pub fn with_halo_lbp_pairs(&mut self, pairs: &[(&Addr, &PairInfo)]) {
        self.halo_lbp_pair_querier = HaloLBPPairQuerier::new(pairs);
    }

    // pub fn with_balance(&mut self, balances: &[(&Addr, &[Coin])]) {
    //     for (addr, balance) in balances {
    //         self.base.update_balance(addr, balance.to_vec());
    //     }
    // }
}
