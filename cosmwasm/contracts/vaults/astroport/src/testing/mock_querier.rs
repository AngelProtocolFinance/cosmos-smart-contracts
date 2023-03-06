// Contains mock functionality to test multi-contract scenarios
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, BalanceResponse, BankQuery, CanonicalAddr, Coin,
    ContractResult, Decimal, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;

use std::collections::HashMap;
use std::marker::PhantomData;

use angel_core::msgs::{accounts::EndowmentDetailsResponse, registrar::ConfigResponse};
use angel_core::structs::{
    AcceptedTokens, AccountStrategies, Categories, OneOffVaults, RebalanceDetails, SplitDetails,
};

use astroport::{
    asset::{AssetInfo, PairInfo},
    factory::PairType,
};

#[cw_serde]
pub enum QueryMsg {
    Endowment { id: u32 },
    Balance { address: String },
    Config {},
    Pair {},
    QueryFlpTokenFromPoolAddress { pool_address: String },
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
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(&msg).unwrap() {
                    // Simulating the `Registrar::QueryMsg::EndowmentList {...}`
                    QueryMsg::Endowment { id: _ } => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&EndowmentDetailsResponse {
                            owner: Addr::unchecked("owner"),
                            status: angel_core::structs::EndowmentStatus::Approved,
                            endow_type: angel_core::structs::EndowmentType::Charity,
                            maturity_time: None,
                            strategies: AccountStrategies::default(),
                            oneoff_vaults: OneOffVaults::default(),
                            rebalance: RebalanceDetails::default(),
                            kyc_donors_only: false,
                            deposit_approved: true,
                            withdraw_approved: true,
                            pending_redemptions: 0,
                            proposal_link: None,
                            name: "Test Endowment".to_string(),
                            categories: Categories::default(),
                            tier: Some(3),
                            logo: Some("Some fancy logo".to_string()),
                            image: Some("Nice banner image".to_string()),
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
                    // Simulating the `registrar::QueryMsg::Config {}` or `vault::QueryMsg::config {}`
                    QueryMsg::Config {} => match contract_addr.as_str() {
                        "locked_sibling_vault" => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&crate::responses::ConfigResponse {
                                ibc_host: "ibc-host".to_string(),
                                ibc_controller: "ibc-controller".to_string(),
                                owner: "owner".to_string(),
                                acct_type: angel_core::structs::AccountType::Locked,
                                sibling_vault: MOCK_CONTRACT_ADDR.to_string(),
                                registrar_contract: "registrar".to_string(),
                                keeper: "keeper".to_string(),
                                tax_collector: "tax-collector".to_string(),
                                native_token: "ujuno".to_string(),
                                lp_token: "lp-token".to_string(),
                                lp_pair_token0: "token0".to_string(),
                                lp_pair_token1: "token1".to_string(),
                                lp_reward_token: "lp-reward-token".to_string(),
                                reward_to_native_rotue: vec![],
                                native_to_lp0_route: vec![],
                                native_to_lp1_route: vec![],
                                lp_factory_contract: "lp-factory".to_string(),
                                lp_staking_contract: "lp-staking".to_string(),
                                lp_pair_contract: "astroport-usdc-usdt-pair".to_string(),
                                minimum_initial_deposit: "100".to_string(),
                                pending_owner: "".to_string(),
                                pending_owner_deadline: 0,
                            })
                            .unwrap(),
                        )),
                        "liquid_sibling_vault" => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&crate::responses::ConfigResponse {
                                ibc_host: "ibc-host".to_string(),
                                ibc_controller: "ibc-controller".to_string(),
                                owner: "owner".to_string(),
                                acct_type: angel_core::structs::AccountType::Liquid,
                                sibling_vault: MOCK_CONTRACT_ADDR.to_string(),
                                registrar_contract: "registrar".to_string(),
                                keeper: "keeper".to_string(),
                                tax_collector: "tax-collector".to_string(),
                                native_token: "ujuno".to_string(),
                                lp_token: "lp-token".to_string(),
                                lp_pair_token0: "token0".to_string(),
                                lp_pair_token1: "token1".to_string(),
                                lp_reward_token: "lp-reward-token".to_string(),
                                reward_to_native_rotue: vec![],
                                native_to_lp0_route: vec![],
                                native_to_lp1_route: vec![],
                                lp_factory_contract: "lp-factory".to_string(),
                                lp_staking_contract: "lp-staking".to_string(),
                                lp_pair_contract: "astroport-usdc-usdt-pair".to_string(),
                                minimum_initial_deposit: "100".to_string(),
                                pending_owner: "".to_string(),
                                pending_owner_deadline: 0,
                            })
                            .unwrap(),
                        )),
                        _ => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&ConfigResponse {
                                owner: "registrar-owner".to_string(),
                                version: "1.0.0".to_string(),
                                accounts_contract: Some("accounts-contract".to_string()),
                                treasury: "treasury".to_string(),
                                rebalance: RebalanceDetails::default(),
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
                                swaps_router: None,
                                axelar_gateway: "axelar-gateway".to_string(),
                                axelar_ibc_channel: "channel-1".to_string(),
                            })
                            .unwrap(),
                        )),
                    },
                    // Simulating the `astroport::pair::Pair {}` query
                    QueryMsg::Pair {} => SystemResult::Ok(ContractResult::Ok(
                        to_binary(&PairInfo {
                            pair_type: PairType::Stable {},
                            asset_infos: [
                                AssetInfo::NativeToken {
                                    denom: "ujuno".to_string(),
                                },
                                AssetInfo::Token {
                                    contract_addr: Addr::unchecked("halo-token"),
                                },
                            ],
                            contract_addr: Addr::unchecked("astroport-usdc-usdt-pair"),
                            liquidity_token: Addr::unchecked("astroport-lp-token"),
                        })
                        .unwrap(),
                    )),
                    // Simulating the `astroport::generator::QueryFlpTokenFromPoolAddress { pool_address: String }` query
                    QueryMsg::QueryFlpTokenFromPoolAddress { pool_address: _ } => SystemResult::Ok(
                        ContractResult::Ok(to_binary(&"flp-token-contract").unwrap()),
                    ),
                }
            }
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
}
