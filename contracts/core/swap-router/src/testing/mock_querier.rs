use angel_core::structs::EndowmentType;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, Api, Coin, Decimal, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest,
    StdResult, SystemError, SystemResult, Uint128,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
        approved: Option<bool>,
        start_after: Option<String>,
        limit: Option<u64>,
    },
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
    terraswap_factory_querier: TerraswapFactoryQuerier,
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

pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

#[derive(Clone, Default)]
pub struct OraclePriceQuerier {
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
#[derive(Clone, Default)]
pub struct PriceStruct {
    base: String,
    quote: String,
    rate: Decimal,
    last_updated_base: u64,
    last_updated_quote: u64,
}

#[derive(Clone, Default)]
pub struct OraclePricesQuerier {
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

#[derive(Clone, Default)]
pub struct TerraswapFactoryQuerier {
    pairs: HashMap<String, String>,
}

impl TerraswapFactoryQuerier {
    pub fn new(pairs: &[(&String, &String)]) -> Self {
        TerraswapFactoryQuerier {
            pairs: pairs_to_map(pairs),
        }
    }
}

pub(crate) fn pairs_to_map(pairs: &[(&String, &String)]) -> HashMap<String, String> {
    let mut pairs_map: HashMap<String, String> = HashMap::new();
    for (key, pair) in pairs.iter() {
        pairs_map.insert(key.to_string(), pair.to_string());
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
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<Empty>, _api: A) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            terraswap_factory_querier: TerraswapFactoryQuerier::default(),
            oracle_price_querier: OraclePriceQuerier::default(),
            oracle_prices_querier: OraclePricesQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    // configure the terraswap pair
    pub fn with_terraswap_pairs(&mut self, pairs: &[(&String, &String)]) {
        self.terraswap_factory_querier = TerraswapFactoryQuerier::new(pairs);
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
