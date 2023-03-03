use crate::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails, StrategyApprovalState,
    StrategyParams,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct MigrateMsg {
    pub accounts_settings_controller: String,
    pub axelar_gateway: String,
    pub axelar_ibc_channel: String,
}

#[cw_serde]
pub struct MigrateEndowment {
    pub addr: String,
    pub status: u64,
    pub name: String,
    pub owner: String,
    pub tier: Option<u64>,
    pub un_sdg: Option<u64>,
    pub logo: Option<String>,
    pub image: Option<String>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury: String,
    pub tax_rate: Decimal,
    pub rebalance: Option<RebalanceDetails>,
    pub split_to_liquid: Option<SplitDetails>, // default %s to split off into liquid account, if donor provided split is not present
    pub accepted_tokens: Option<AcceptedTokens>, // list of approved native and CW20 coins can accept inward
    pub swap_factory: Option<String>,
    pub accounts_settings_controller: String,
    pub axelar_gateway: String,
    pub axelar_ibc_channel: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    StrategyAdd {
        strategy_key: String,
        strategy: StrategyParams,
    },
    StrategyRemove {
        strategy_key: String,
    },
    StrategyUpdate {
        strategy_key: String,
        approval_state: StrategyApprovalState,
    },
    // Allows the contract parameter to be updated (only by the owner...for now)
    UpdateConfig(UpdateConfigMsg),
    // Allows the SC owner to change ownership
    UpdateOwner {
        new_owner: String,
    },
    // Updates the NETWORK_CONNECTIONS
    UpdateNetworkConnections {
        chain_id: String,
        network_info: NetworkInfo,
        action: String,
    },
    UpdateFees {
        fees: Vec<(String, Decimal)>,
    },
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub accounts_contract: Option<String>,
    pub rebalance: Option<RebalanceDetails>,
    pub split_max: Option<Decimal>,
    pub split_min: Option<Decimal>,
    pub split_default: Option<Decimal>,
    pub collector_share: Option<Decimal>,
    pub accepted_tokens: Option<AcceptedTokens>,
    /// WASM CODES
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub subdao_gov_code: Option<u64>,        // subdao gov wasm code
    pub subdao_cw20_token_code: Option<u64>, // subdao gov token (basic CW20) wasm code
    pub subdao_bonding_token_code: Option<u64>, // subdao gov token (w/ bonding-curve) wasm code
    pub subdao_cw900_code: Option<u64>,      // subdao gov ve-CURVE contract for locked token voting
    pub subdao_distributor_code: Option<u64>, // subdao gov fee distributor wasm code
    pub donation_match_code: Option<u64>,    // donation matching contract wasm code
    /// CONTRACT ADDRESSES
    pub index_fund_contract: Option<String>,
    pub gov_contract: Option<String>,
    pub treasury: Option<String>,
    pub donation_match_charites_contract: Option<String>,
    pub halo_token: Option<String>,
    pub halo_token_lp_contract: Option<String>,
    pub charity_shares_contract: Option<String>,
    pub collector_addr: Option<String>,
    pub swap_factory: Option<String>,
    pub fundraising_contract: Option<String>,
    pub applications_review: Option<String>,
    pub swaps_router: Option<String>,
    pub accounts_settings_controller: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Get details on single strategy
    #[returns(StrategyDetailResponse)]
    Strategy { strategy_key: String },
    // Get all Config details for the contract
    #[returns(ConfigResponse)]
    Config {},
    // Get a network connection info
    #[returns(NetworkConnectionResponse)]
    NetworkConnection { chain_id: String },
    #[returns(FeesResponse)]
    Fee { name: String },
}

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
