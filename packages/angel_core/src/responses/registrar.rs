use crate::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails, VaultRate, YieldVault,
};
use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VaultDetailResponse {
    pub vault: YieldVault,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VaultListResponse {
    pub vaults: Vec<YieldVault>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
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
    pub tax_rate: Decimal,
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
    pub swaps_router: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct VaultRateResponse {
    pub vaults_rate: Vec<VaultRate>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct FeesResponse {
    pub tax_rate: Decimal,
    pub endowtype_charity: Option<Decimal>,
    pub endowtype_normal: Option<Decimal>,
}

pub struct AccTokensListResponse {
    pub tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NetworkConnectionResponse {
    pub network_connection: NetworkInfo,
}
