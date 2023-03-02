use crate::structs::{AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails, StrategyParams};
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct StrategyDetailResponse {
    pub strategy: StrategyParams,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub subdao_gov_code: Option<u64>,
    pub subdao_cw20_token_code: Option<u64>,
    pub subdao_bonding_token_code: Option<u64>,
    pub subdao_cw900_code: Option<u64>,
    pub subdao_distributor_code: Option<u64>,
    pub donation_match_code: Option<u64>,
    pub halo_token: Option<String>,
    pub halo_token_lp_contract: Option<String>,
    pub gov_contract: Option<String>,
    pub accounts_contract: Option<String>,
    pub treasury: String,
    pub rebalance: RebalanceDetails,
    pub index_fund: Option<String>,
    pub split_to_liquid: SplitDetails,
    pub donation_match_charites_contract: Option<String>,
    pub collector_addr: String,
    pub collector_share: Decimal,
    pub charity_shares_contract: Option<String>,
    pub accepted_tokens: AcceptedTokens,
    pub swap_factory: Option<String>,
    pub applications_review: String,
    pub applications_impact_review: String,
    pub swaps_router: Option<String>,
    pub accounts_settings_controller: String,
    pub axelar_gateway: String,
    pub axelar_ibc_channel: String,
    pub vault_router: Option<String>,
}

#[cw_serde]
pub struct FeesResponse {
    pub tax_rate: Decimal,
    pub endowtype_charity: Option<Decimal>,
    pub endowtype_normal: Option<Decimal>,
}

pub struct AccTokensListResponse {
    pub tokens: Vec<String>,
}

#[cw_serde]
pub struct NetworkConnectionResponse {
    pub chain: String,
    pub network_connection: NetworkInfo,
}
