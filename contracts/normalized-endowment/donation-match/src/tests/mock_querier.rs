use angel_core::responses::accounts::EndowmentDetailsResponse;
// Contains mock functionality to test multi-contract scenarios
use angel_core::responses::registrar::{ConfigResponse, VaultDetailResponse};
use angel_core::responses::settings_controller::EndowmentSettingsResponse;
use angel_core::structs::{
    AcceptedTokens, AccountStrategies, AccountType, Categories, OneOffVaults, RebalanceDetails,
    SettingsController, SplitDetails, VaultType, YieldVault,
};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Coin, ContractResult, Decimal, Empty, OwnedDeps,
    Querier, QuerierResult, QueryRequest, StdResult, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::BalanceResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::pair::SimulationResponse;

use std::collections::HashMap;
use std::marker::PhantomData;
use terraswap::asset::Asset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Simulation { offer_asset: Asset },
    Balance { address: String },
    // Mock the `registrar` config
    Config {},
    Vault { vault_addr: String },
    // Mock the `accounts` endowment
    Endowment { id: u32 },
    // Mock the "settings_controller::EndowmentSettings {id: [EndowmentID]}" query
    EndowmentSettings { id: u32 },
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
                contract_addr,
                msg,
            }) => match from_binary(&msg).unwrap() {
                QueryMsg::Endowment { id: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&EndowmentDetailsResponse {
                        owner: Addr::unchecked("endow-cw3"),
                        name: "Test Endowment".to_string(),
                        strategies: AccountStrategies::default(),
                        status: angel_core::structs::EndowmentStatus::Approved,
                        endow_type: angel_core::structs::EndowmentType::Charity,
                        maturity_time: None,
                        oneoff_vaults: OneOffVaults::default(),
                        rebalance: RebalanceDetails::default(),
                        kyc_donors_only: false,
                        deposit_approved: true,
                        withdraw_approved: true,
                        pending_redemptions: 0,
                        proposal_link: None,
                        categories: Categories::default(),
                        tier: None,
                        image: None,
                        logo: None,
                    }).unwrap()
                )),
                QueryMsg::Simulation { offer_asset: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&SimulationResponse {
                        return_amount: Uint128::from(100_u128),
                        spread_amount: Uint128::from(5_u128),
                        commission_amount: Uint128::from(5_u128),
                    })
                    .unwrap(),
                )),
                QueryMsg::Balance { address: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&BalanceResponse {
                        balance: Uint128::from(100_u128),
                    })
                    .unwrap(),
                )),
                QueryMsg::Config {} => {
                    match contract_addr.as_str() {
                        "accounts-contract" => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&angel_core::responses::accounts::ConfigResponse {
                                owner: "owner".to_string(),
                                registrar_contract: "registrar-contract".to_string(),
                                next_account_id: 2,
                                max_general_category_id: 1,
                            }).unwrap())),
                        "registrar-contract" => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&ConfigResponse {
                                version: "1.7.0".to_string(),
                                owner: "Test-Endowment-Owner".to_string(),
                                accounts_contract: Some("accounts-contract".to_string()),
                                rebalance: RebalanceDetails::default(),
                                applications_review: "applications-review".to_string(),
                                swaps_router: Some("swaps-router".to_string()),
                                cw3_code: Some(124),
                                cw4_code: Some(125),
                                subdao_gov_code: Some(126),
                                subdao_bonding_token_code: Some(127),
                                subdao_cw20_token_code: Some(129),
                                subdao_cw900_code: Some(128),
                                subdao_distributor_code: None,
                                donation_match_code: None,
                                halo_token: None,
                                halo_token_lp_contract: None,
                                gov_contract: None,
                                treasury: "treasury-address".to_string(),
                                index_fund: None,
                                split_to_liquid: SplitDetails::default(),
                                donation_match_charites_contract: Some(MOCK_CONTRACT_ADDR.to_string()),
                                collector_addr: "collector-addr".to_string(),
                                collector_share: Decimal::percent(50),
                                charity_shares_contract: None,
                                accepted_tokens: AcceptedTokens {
                                    native: vec![
                                        "uluna".to_string(),
                                        "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                    ],
                                    cw20: vec![],
                                },
                                swap_factory: Some("swap-factory".to_string()),
                                settings_controller: Some("settings-controller".to_string()),
                            })
                            .unwrap()
                        )),
                        _ => unreachable!(),
                    }
                }
                QueryMsg::Vault { vault_addr: _ } => SystemResult::Ok(ContractResult::Ok(
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
            },
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

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
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
