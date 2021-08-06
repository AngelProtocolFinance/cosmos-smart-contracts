use crate::structs::{AssetVault, EndowmentEntry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VaultDetailResponse {
    pub vault: AssetVault,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VaultListResponse {
    pub vaults: Vec<AssetVault>,
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
    pub approved_coins: Vec<String>,
    pub accounts_code_id: u64,
}
