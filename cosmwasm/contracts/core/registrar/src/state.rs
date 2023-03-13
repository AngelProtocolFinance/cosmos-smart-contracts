use angel_core::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, RegistrarConfigCore, RegistrarConfigExtension,
    SplitDetails, StrategyParams,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct OldConfig {
    pub owner: Addr,               // AP TEAM MULTISIG
    pub applications_review: Addr, // Charity Endowment application review team's CW3 (set as owner to start). Owner can set and change/revoke.
    pub applications_impact_review: Addr,
    pub index_fund_contract: Option<Addr>,
    pub accounts_contract: Option<Addr>,
    pub treasury: Addr,
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub split_to_liquid: SplitDetails, // set of max, min, and default Split paramenters to check user defined split input against
    pub halo_token: Option<Addr>,      // TerraSwap HALO token addr
    pub gov_contract: Option<Addr>,    // AP governance contract
    pub charity_shares_contract: Option<Addr>, // Charity Shares staking contract
    pub swaps_router: Option<Addr>,    // swaps router contract
    pub cw3_code: Option<u64>,
    pub cw4_code: Option<u64>,
    pub accepted_tokens: AcceptedTokens, // list of approved native and CW20 coins can accept inward
}

pub const CONFIG: Item<RegistrarConfigCore> = Item::new("config");
pub const CONFIG_EXTENSION: Item<RegistrarConfigExtension> = Item::new("config_extension");
pub const STRATEGIES: Map<&[u8], StrategyParams> = Map::new("strategies");
pub const NETWORK_CONNECTIONS: Map<&str, NetworkInfo> = Map::new("network_connections");
pub const FEES: Map<&str, Decimal> = Map::new("fee");
