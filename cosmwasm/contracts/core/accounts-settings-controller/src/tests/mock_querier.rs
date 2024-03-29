use angel_core::msgs::registrar::{
    ConfigExtensionResponse as RegistrarConfigExtensionResponse,
    ConfigResponse as RegistrarConfigResponse,
};
use angel_core::structs::{
    AcceptedTokens, BalanceInfo, Categories, DonationsReceived, EndowmentType, Investments,
    RebalanceDetails, SplitDetails,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, CanonicalAddr, Coin, ContractResult, Decimal,
    Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128,
    WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;
use std::marker::PhantomData;

#[cw_serde]
pub enum QueryMsg {
    Config {},
    ConfigExtension {},
    Endowment { id: u32 },
    State { id: u32 },
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
    oracle_price_querier: OraclePriceQuerier,
    oracle_prices_querier: OraclePricesQuerier,
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

#[derive(Clone, Default)]
pub struct OraclePriceQuerier {
    #[allow(dead_code)]
    // this lets us iterate over all pairs that match the first string
    oracle_price: HashMap<(String, String), (Decimal, u64, u64)>,
}

impl OraclePriceQuerier {
    #[allow(dead_code)]
    pub fn new(oracle_price: &[(&(String, String), &(Decimal, u64, u64))]) -> Self {
        OraclePriceQuerier {
            oracle_price: oracle_price_to_map(oracle_price),
        }
    }
}
#[allow(dead_code)]
pub(crate) fn oracle_price_to_map(
    oracle_price: &[(&(String, String), &(Decimal, u64, u64))],
) -> HashMap<(String, String), (Decimal, u64, u64)> {
    let mut oracle_price_map: HashMap<(String, String), (Decimal, u64, u64)> = HashMap::new();
    for (base_quote, oracle_price) in oracle_price.iter() {
        oracle_price_map.insert((*base_quote).clone(), **oracle_price);
    }

    oracle_price_map
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

#[derive(Clone, Default)]
pub struct OraclePricesQuerier {
    #[allow(dead_code)]
    // this lets us iterate over all pairs
    oracle_prices: Vec<PriceStruct>,
}

impl OraclePricesQuerier {
    #[allow(dead_code)]
    pub fn new(oracle_prices: &[(&(String, String), &(Decimal, u64, u64))]) -> Self {
        OraclePricesQuerier {
            oracle_prices: oracle_prices_to_map(oracle_prices),
        }
    }
}

pub(crate) fn oracle_prices_to_map(
    oracle_prices: &[(&(String, String), &(Decimal, u64, u64))],
) -> Vec<PriceStruct> {
    let mut oracle_prices_map: Vec<PriceStruct> = vec![];
    for (base_quote, oracle_prices) in oracle_prices.iter() {
        oracle_prices_map.push(PriceStruct {
            base: base_quote.0.clone(),
            quote: base_quote.1.clone(),
            rate: oracle_prices.0,
            last_updated_base: oracle_prices.1,
            last_updated_quote: oracle_prices.2,
        });
    }

    oracle_prices_map
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
                        treasury: "treasury".to_string(),
                        rebalance: RebalanceDetails::default(),
                        split_to_liquid: SplitDetails {
                            min: Decimal::zero(),
                            max: Decimal::one(),
                            default: Decimal::percent(50),
                        },
                        accepted_tokens: AcceptedTokens {
                            native: vec!["ujuno".to_string()],
                            cw20: vec!["test-cw20".to_string()],
                        },
                        axelar_gateway: "axelar-gateway".to_string(),
                        axelar_ibc_channel: "channel-1".to_string(),
                        axelar_chain_id: "juno".to_string(),
                    })
                    .unwrap(),
                )),
                QueryMsg::ConfigExtension {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&RegistrarConfigExtensionResponse {
                        index_fund: Some("index_fund".to_string()),
                        accounts_contract: Some("accounts-contract".to_string()),
                        subdao_gov_code: Some(333),
                        subdao_cw20_token_code: Some(4_u64),
                        subdao_bonding_token_code: Some(3_u64),
                        subdao_cw900_code: None,
                        subdao_distributor_code: None,
                        donation_match_code: Some(334),
                        donation_match_charites_contract: None,
                        collector_addr: "collector-addr".to_string(),
                        halo_token: Some("halo_token".to_string()),
                        halo_token_lp_contract: Some("halo_token_lp_contract".to_string()),
                        gov_contract: Some("gov_contract".to_string()),
                        charity_shares_contract: Some("charity_shares".to_string()),
                        cw3_code: Some(2),
                        cw4_code: Some(3),
                        swap_factory: None,
                        applications_review: "applications-review".to_string(),
                        swaps_router: Some("swaps_router_addr".to_string()),
                        accounts_settings_controller: Some(
                            "accounts-settings-controller".to_string(),
                        ),
                    })
                    .unwrap(),
                )),
                QueryMsg::Endowment { id } => match id {
                    1 => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&angel_core::msgs::accounts::EndowmentDetailsResponse {
                            owner: Addr::unchecked("endowment-owner"),
                            name: "Test Endowment 1".to_string(),
                            categories: Categories::default(),
                            tier: Some(2),
                            endow_type: EndowmentType::Normal,
                            logo: Some("test-logo".to_string()),
                            image: Some("test-image".to_string()),
                            status: angel_core::structs::EndowmentStatus::Approved,
                            deposit_approved: true,
                            withdraw_approved: true,
                            maturity_time: None,
                            invested_strategies: Investments::default(),
                            rebalance: RebalanceDetails::default(),
                            kyc_donors_only: false,
                            pending_redemptions: 0,
                            proposal_link: None,
                            referral_id: None,
                        })
                        .unwrap(),
                    )),
                    2 => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&angel_core::msgs::accounts::EndowmentDetailsResponse {
                            owner: Addr::unchecked("endowment-owner"),
                            name: "Test Endowment 2".to_string(),
                            categories: Categories::default(),
                            tier: Some(2),
                            endow_type: EndowmentType::Charity,
                            logo: Some("test-logo".to_string()),
                            image: Some("test-image".to_string()),
                            status: angel_core::structs::EndowmentStatus::Approved,
                            deposit_approved: true,
                            withdraw_approved: true,
                            maturity_time: Some(10000000),
                            invested_strategies: Investments::default(),
                            rebalance: RebalanceDetails::default(),
                            kyc_donors_only: false,
                            pending_redemptions: 0,
                            proposal_link: None,
                            referral_id: None,
                        })
                        .unwrap(),
                    )),
                    _ => unreachable!(),
                },
                QueryMsg::State { id } => match id {
                    1 => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&angel_core::msgs::accounts::StateResponse {
                            donations_received: DonationsReceived {
                                locked: Uint128::from(1000000_u128),
                                liquid: Uint128::from(1000000_u128),
                            },
                            tokens_on_hand: BalanceInfo::default(),
                            closing_endowment: false,
                            closing_beneficiary: None,
                        })
                        .unwrap(),
                    )),
                    2 => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&angel_core::msgs::accounts::StateResponse {
                            donations_received: DonationsReceived {
                                locked: Uint128::from(1000000_u128),
                                liquid: Uint128::from(1000000_u128),
                            },
                            tokens_on_hand: BalanceInfo::default(),
                            closing_endowment: true,
                            closing_beneficiary: None,
                        })
                        .unwrap(),
                    )),
                    _ => unreachable!(),
                },
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
            oracle_price_querier: OraclePriceQuerier::default(),
            oracle_prices_querier: OraclePricesQuerier::default(),
        }
    }

    //  Configure oracle price
    #[allow(dead_code)]
    pub fn with_oracle_price(
        &mut self,
        oracle_price: &[(&(String, String), &(Decimal, u64, u64))],
    ) {
        self.oracle_price_querier = OraclePriceQuerier::new(oracle_price);
    }

    //  Configure oracle prices
    #[allow(dead_code)]
    pub fn with_oracle_prices(
        &mut self,
        oracle_prices: &[(&(String, String), &(Decimal, u64, u64))],
    ) {
        self.oracle_prices_querier = OraclePricesQuerier::new(oracle_prices);
    }
}
