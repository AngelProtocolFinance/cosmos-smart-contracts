use crate::structs::{
    AcceptedTokens, EndowmentEntry, NetworkInfo, SplitDetails, VaultRate, YieldVault,
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
pub struct EndowmentDetailResponse {
    pub endowment: EndowmentEntry,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct EndowmentListResponse {
    pub endowments: Vec<EndowmentEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub accounts_contract: Option<String>,
    pub treasury: String,
    pub tax_rate: Decimal,
    pub default_vault: Option<String>,
    pub index_fund: Option<String>,
    pub split_to_liquid: SplitDetails,
    pub halo_token: Option<String>,
    pub gov_contract: Option<String>,
    pub charity_shares_contract: Option<String>,
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub accepted_tokens: AcceptedTokens,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct VaultRateResponse {
    pub vaults_rate: Vec<VaultRate>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AccTokensListResponse {
    pub tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NetworkConnectionResponse {
    pub network_connection: NetworkInfo,
}
