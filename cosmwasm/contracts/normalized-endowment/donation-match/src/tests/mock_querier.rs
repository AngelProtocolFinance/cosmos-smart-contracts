use angel_core::msgs::accounts::EndowmentDetailsResponse;
// Contains mock functionality to test multi-contract scenarios
use angel_core::msgs::accounts_settings_controller::EndowmentSettingsResponse;
use angel_core::msgs::registrar::{
    ConfigExtensionResponse as RegistrarConfigExtensionResponse,
    ConfigResponse as RegistrarConfigResponse, StrategyDetailResponse,
};
use angel_core::structs::{
    AcceptedTokens, Categories, Investments, RebalanceDetails, SplitDetails, StrategyApprovalState,
    StrategyLocale, StrategyParams,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, StdResult, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::BalanceResponse;
use std::collections::HashMap;
use std::marker::PhantomData;
use terraswap::{asset::Asset, pair::SimulationResponse};

#[cw_serde]
pub enum QueryMsg {
    Simulation { offer_asset: Asset },
    Balance { address: String },
    // Mock the `registrar` config
    Config {},
    ConfigExtension {},
    Strategy { strategy_key: String },
    // Mock the `accounts` endowment
    Endowment { id: u32 },
    // Mock the "endowment_controller::EndowmentSettings {id: [EndowmentID]}" query
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
                        status: angel_core::structs::EndowmentStatus::Approved,
                        endow_type: angel_core::structs::EndowmentType::Charity,
                        maturity_time: None,
                        invested_strategies: Investments::default(),
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
                        referral_id: None,
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
                            to_binary(&angel_core::msgs::accounts::ConfigResponse {
                                owner: "owner".to_string(),
                                version: "accounts-v2.0.0".to_string(),
                                registrar_contract: "registrar-contract".to_string(),
                                next_account_id: 2,
                                max_general_category_id: 1,
                            }).unwrap())),
                        "registrar-contract" => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&RegistrarConfigResponse {
                                version: "1.7.0".to_string(),
                                owner: "Test-Endowment-Owner".to_string(),
                                treasury: "treasury-address".to_string(),
                                split_to_liquid: SplitDetails::default(),
                                rebalance: RebalanceDetails::default(),
                                accepted_tokens: AcceptedTokens {
                                    native: vec![
                                        "uluna".to_string(),
                                        "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
                                    ],
                                    cw20: vec![],
                                },
                                axelar_gateway: "axelar-gateway".to_string(),
                                axelar_ibc_channel: "channel-1".to_string(),
                            })
                            .unwrap()
                        )),
                        _ => unreachable!(),
                    }
                }
                QueryMsg::ConfigExtension {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&RegistrarConfigExtensionResponse {
                        index_fund: Some("index_fund".to_string()),
                        accounts_contract: Some("accounts-contract".to_string()),
                        cw3_code: Some(124),
                                cw4_code: Some(125),
                                subdao_gov_code: Some(126),
                                subdao_bonding_token_code: Some(127),
                                subdao_cw20_token_code: Some(129),
                                subdao_cw900_code: Some(128),
                                subdao_distributor_code: None,
                        donation_match_charites_contract: Some(MOCK_CONTRACT_ADDR.to_string()),
                        donation_match_code: None,
                        collector_addr: "collector-addr".to_string(),
                        halo_token: Some("halo_token".to_string()),
                        halo_token_lp_contract: Some("halo_token_lp_contract".to_string()),
                        gov_contract: Some("gov_contract".to_string()),
                        charity_shares_contract: Some("charity_shares".to_string()),
                        swap_factory: None,
                        applications_review: "applications-review".to_string(),
                        swaps_router: Some("swaps_router_addr".to_string()),
                        accounts_settings_controller: Some(
                            "accounts-settings-controller".to_string(),
                        ),
                    })
                    .unwrap(),
                )),
                QueryMsg::Strategy { strategy_key: _ } => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&StrategyDetailResponse {
                        strategy: StrategyParams {
                            approval_state: StrategyApprovalState::Approved,
                            locale: StrategyLocale::Native,
                            chain: "juno".to_string(),
                            input_denom: "input-denom".to_string(),
                            locked_addr: Some(Addr::unchecked("vault1-locked-contract")),
                            liquid_addr: Some(Addr::unchecked("vault1-liquid-contract")),
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
                        split_to_liquid: None,
                        ignore_user_splits: false,
                        parent: None,
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
