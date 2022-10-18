use crate::structs::{
    AcceptedTokens, Fees, NetworkInfo, RebalanceDetails, SplitDetails, YieldVault,
};
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
    pub accounts_contract: Option<String>,
    pub treasury: String,
    pub fees: Fees,
    pub rebalance: RebalanceDetails,
    pub index_fund: Option<String>,
    pub split_to_liquid: SplitDetails,
    pub halo_token: Option<String>,
    pub gov_contract: Option<String>,
    pub charity_shares_contract: Option<String>,
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub accepted_tokens: AcceptedTokens,
    pub applications_review: String,
    pub swaps_router: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AccTokensListResponse {
    pub tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NetworkConnectionResponse {
    pub network_connection: NetworkInfo,
}
