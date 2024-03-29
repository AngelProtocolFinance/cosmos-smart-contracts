// Contains mock functionality to test multi-contract scenarios
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, BalanceResponse, BankQuery, CanonicalAddr, Coin,
    ContractResult, Decimal, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, StdResult,
    SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;

use std::collections::HashMap;
use std::marker::PhantomData;

use angel_core::msgs::registrar::{ConfigResponse, EndowmentListResponse};
use angel_core::msgs::vault::InfoResponse;
use angel_core::structs::{AcceptedTokens, SplitDetails};

#[cw_serde]
pub enum QueryMsg {
    Info {},
    Balance { address: String },
    Config {},
}

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let mut custom_querier: WasmMockQuerier = WasmMockQuerier::new(
        MockQuerier::new(&[(&contract_addr, contract_balance)]),
        MockApi::default(),
    );
    let contract_balance: Vec<(&String, &Uint128)> = contract_balance
        .into_iter()
        .map(|x| (&(x.denom), &(x.amount)))
        .collect();
    custom_querier.token_querier =
        TokenQuerier::new(&[(&contract_addr.to_string(), &contract_balance)]);
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this allows to iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

#[allow(dead_code)]
#[cw_serde]
pub struct PriceStruct {
    base: String,
    quote: String,
    rate: Decimal,
    last_updated_base: u64,
    last_updated_quote: u64,
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
                    to_binary(&BalanceResponse {
                        amount: Coin {
                            denom: denom.to_string(),
                            amount: Uint128::from(100_u128),
                        },
                    })
                    .unwrap(),
                ))
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => match from_binary(&msg).unwrap() {
                // Simulating the `junoswap::QueryMsg::Info {}`
                QueryMsg::Info {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&InfoResponse {
                        token1_reserve: Uint128::from(100_u128),
                        token1_denom: cw20::Denom::Native("ujuno".to_string()),
                        token2_reserve: Uint128::from(100_u128),
                        token2_denom: cw20::Denom::Cw20(Addr::unchecked("halo-token-contract")),
                        lp_token_address: "lp-token-address".to_string(),
                        lp_token_supply: Uint128::from(100_u128),
                    })
                    .unwrap(),
                )),

                // Simulating the `cw20::QueryMsg::Balance { address: [account_address]}`
                QueryMsg::Balance { address: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&cw20::BalanceResponse {
                        balance: Uint128::from(100_u128),
                    })
                    .unwrap(),
                )),
                // Simulating the `registrar::QueryMsg::Config {}`
                QueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&ConfigResponse {
                        owner: "registrar-owner".to_string(),
                        version: "1.0.0".to_string(),
                        accounts_contract: Some("accounts-contract".to_string()),
                        treasury: "treasury".to_string(),
                        tax_rate: Decimal::from_ratio(5_u128, 100_u128),
                        index_fund: None,
                        split_to_liquid: SplitDetails {
                            max: Decimal::one(),
                            min: Decimal::zero(),
                            default: Decimal::default(),
                        },
                        halo_token: None,
                        gov_contract: None,
                        charity_shares_contract: None,
                        cw3_code: Some(3_u64),
                        cw4_code: Some(4_u64),
                        accepted_tokens: AcceptedTokens {
                            native: vec![],
                            cw20: vec![],
                        },
                        applications_review: "applications-review".to_string(),
                        swaps_router: Some("swaps_router_addr".to_string()),
                        axelar_gateway: "axelar-gateway".to_string(),
                        axelar_ibc_channel: "channel-1".to_string(),
                    })
                    .unwrap(),
                )),
            },
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                let balances: &HashMap<String, Uint128> =
                    match self.token_querier.balances.get(contract_addr) {
                        Some(balances) => balances,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!(
                                    "No balance info exists for the contract {}",
                                    contract_addr
                                ),
                                request: key.into(),
                            })
                        }
                    };

                if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key[prefix_balance.len()..];
                    let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);

                    let api: MockApi = MockApi::default();
                    let address: String = match api.addr_humanize(&address_raw) {
                        Ok(v) => v.to_string(),
                        Err(e) => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!("Parsing query request: {}", e),
                                request: key.into(),
                            })
                        }
                    };

                    let balance = match balances.get(&address) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            })
                        }
                    };

                    SystemResult::Ok(ContractResult::Ok(to_binary(&balance).unwrap()))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<Empty>, _api: A) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    pub fn query_all_balances(&mut self, address: String) -> StdResult<Vec<Coin>> {
        let mut res = vec![];
        for contract_addr in self.token_querier.balances.keys() {
            let balances = self
                .token_querier
                .balances
                .get(&contract_addr.clone())
                .unwrap();
            for account_addr in balances.keys() {
                if (*account_addr.clone()).to_string() == address {
                    res.push(Coin {
                        denom: contract_addr.clone().to_string(),
                        amount: *balances.get(account_addr).unwrap(),
                    })
                }
            }
        }
        Ok(res)
    }
}
