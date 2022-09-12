use angel_core::structs::{
    AcceptedTokens, AccountType, EndowmentType, NetworkInfo, RebalanceDetails, SplitDetails,
    YieldVault,
};
use cosmwasm_std::{Addr, Decimal, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,               // AP TEAM MULTISIG
    pub applications_review: Addr, // Endowment application review team's CW3 (set as owner to start). Owner can set and change/revoke.
    pub index_fund_contract: Option<Addr>,
    pub accounts_contract: Option<Addr>,
    pub treasury: Addr,
    pub tax_rate: Decimal,
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

pub const CONFIG: Item<Config> = Item::new("config");
pub const VAULTS: Map<&[u8], YieldVault> = Map::new("vault");
pub const NETWORK_CONNECTIONS: Map<&str, NetworkInfo> = Map::new("network_connections");

pub fn read_vaults(
    storage: &dyn Storage,
    network: Option<String>,
    endowment_type: Option<EndowmentType>,
    acct_type: Option<AccountType>,
    approved: Option<bool>,
    start_after: Option<Addr>,
    limit: Option<u64>,
) -> StdResult<Vec<YieldVault>> {
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.as_bytes().to_vec()));
    VAULTS
        .range(storage, start, None, Order::Ascending)
        .take(limit.unwrap_or(DEFAULT_LIMIT).max(MAX_LIMIT) as usize)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .filter(|vault| match vault {
            Ok(v) => matches!(
                (approved, v.approved),
                (None, _) | (Some(true), true) | (Some(false), false)
            ),
            &Err(_) => false,
        })
        .filter(|vault| match vault {
            Ok(v) => match &endowment_type {
                Some(et) => !v.restricted_from.iter().any(|t| t == et),
                None => true,
            },
            &Err(_) => false,
        })
        .filter(|vault| match vault {
            Ok(v) => match &acct_type {
                Some(at) => at == &v.acct_type,
                None => true,
            },
            &Err(_) => false,
        })
        .filter(|vault| match vault {
            Ok(v) => match &network {
                Some(n) => n == &v.network,
                None => true,
            },
            &Err(_) => false,
        })
        .collect()
}
