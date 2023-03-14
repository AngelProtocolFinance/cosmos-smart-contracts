#[allow(unused_imports)]
use crate::msgs::registrar::{
    ConfigResponse, FeesResponse, NetworkConnectionResponse, StrategyDetailResponse,
    UpdateConfigMsg,
};
use crate::structs::{NetworkInfo, StrategyApprovalState, StrategyParams};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct MigrateMsg {}

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
    // Allows the contract Configs (core) to be updated (only by the owner for now)
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Get details on single strategy
    #[returns(StrategyDetailResponse)]
    Strategy { strategy_key: String },
    // Get Core Config details for the contract
    #[returns(ConfigResponse)]
    Config {},
    // Get a network connection info
    #[returns(NetworkConnectionResponse)]
    NetworkConnection { chain_id: String },
    #[returns(FeesResponse)]
    Fee { name: String },
}
