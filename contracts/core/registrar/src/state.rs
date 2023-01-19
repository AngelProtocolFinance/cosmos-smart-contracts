use angel_core::structs::{
    AcceptedTokens, NetworkInfo, RebalanceDetails, SplitDetails, YieldVault,
};
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OldConfig {
    pub owner: Addr,               // AP TEAM MULTISIG
    pub applications_review: Addr, // Endowment application review team's CW3 (set as owner to start). Owner can set and change/revoke.
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,               // AP TEAM MULTISIG
    pub applications_review: Addr, // Endowment application review team's CW3 (set as owner to start). Owner can set and change/revoke.
    pub index_fund_contract: Option<Addr>,
    pub accounts_contract: Option<Addr>,
    pub treasury: Addr,
    pub cw3_code: Option<u64>,                  // multisig wasm code
    pub cw4_code: Option<u64>,                  // multisig wasm code
    pub subdao_gov_code: Option<u64>,           // subdao gov wasm code
    pub subdao_cw20_token_code: Option<u64>,    // subdao gov cw20 token wasm code
    pub subdao_bonding_token_code: Option<u64>, // subdao gov bonding curve token wasm code
    pub subdao_cw900_code: Option<u64>, // subdao gov ve-CURVE contract for locked token voting
    pub subdao_distributor_code: Option<u64>, // subdao gov fee distributor wasm code
    pub donation_match_code: Option<u64>, // donation matching contract wasm code
    pub donation_match_charites_contract: Option<Addr>, // donation matching contract address for "Charities" endowments
    pub split_to_liquid: SplitDetails, // set of max, min, and default Split paramenters to check user defined split input against
    pub halo_token: Option<Addr>,      // TerraSwap HALO token addr
    pub halo_token_lp_contract: Option<Addr>,
    pub gov_contract: Option<Addr>,   // AP governance contract
    pub collector_addr: Option<Addr>, // Collector address for new fee
    pub collector_share: Decimal,
    pub charity_shares_contract: Option<Addr>,
    pub accepted_tokens: AcceptedTokens, // list of approved native and CW20 coins can accept inward
    pub swap_factory: Option<Addr>,
    pub fundraising_contract: Option<Addr>,
    pub rebalance: RebalanceDetails,
    pub swaps_router: Option<Addr>,
    pub accounts_settings_controller: Addr, // contract address used for storing extra Endowment settings
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const VAULTS: Map<&[u8], YieldVault> = Map::new("vault");
pub const NETWORK_CONNECTIONS: Map<&str, NetworkInfo> = Map::new("network_connections");
pub const FEES: Map<&str, Decimal> = Map::new("fee");
