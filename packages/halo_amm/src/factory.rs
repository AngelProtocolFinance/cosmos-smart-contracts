use crate::asset::{AssetInfo, PairInfo};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Pair contract code ID, which is used to
    pub pair_code_id: u64,
    pub token_code_id: u64,
    pub owner: String,
    pub collector_addr: String,
    pub commission_rate: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// UpdateConfig update relevant code IDs
    UpdateConfig {
        owner: Option<String>,
        token_code_id: Option<u64>,
        pair_code_id: Option<u64>,
        pair_contract: String,
        collector_addr: Option<String>,
        commission_rate: Option<String>,
    },
    /// Update asset_infos post-LBP
    UpdateAssetInfos {
        pair_contract: String,
        asset_infos: [AssetInfo; 2],
    },
    /// CreatePair instantiates pair contract
    CreatePair {
        /// Asset infos
        asset_infos: [AssetInfo; 2],
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Pair {
        asset_infos: [AssetInfo; 2],
    },
    Pairs {
        start_after: Option<[AssetInfo; 2]>,
        limit: Option<u32>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub pair_code_id: u64,
    pub token_code_id: u64,
    pub collector_addr: String,
    pub commission_rate: String,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairsResponse {
    pub pairs: Vec<PairInfo>,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FactoryPairInfo {
    pub contract_addr: Addr,
}
