use angel_core::responses::index_fund::AllianceMemberResponse;
use angel_core::structs::{AcceptedTokens, AllianceMember, GenericBalance, IndexFund};
use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const TCA_DONATIONS: Map<String, GenericBalance> = Map::new("tca_donation");
pub const ALLIANCE_MEMBERS: Map<Addr, AllianceMember> = Map::new("alliance_members");
pub const FUND: Map<&[u8], IndexFund> = Map::new("fund");

const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,                   // DANO Address
    pub registrar_contract: Addr,      // Address of Registrar SC
    pub fund_rotation: Option<u64>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,     // limit to number of members an IndexFund can have
    pub funding_goal: Option<Uint128>, // donation funding limit (in UUSD) to trigger early cycle of the Active IndexFund
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub total_funds: u64,
    pub active_fund: u64,         // index ID of the Active IndexFund
    pub round_donations: Uint128, // total donations given to active charity this round
    pub next_rotation_block: u64, // block height to perform next rotation on
    pub next_fund_id: u64,
}

// FUND pagination read util
pub fn read_funds(
    storage: &dyn Storage,
    start_after: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<IndexFund>> {
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.to_be_bytes().to_vec()));
    FUND.range(storage, start, None, Order::Ascending)
        .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

pub fn read_alliance_members(
    storage: &dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u64>,
) -> StdResult<Vec<AllianceMemberResponse>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|v| Bound::exclusive(v));
    // let start: Option<Vec<u8>> = calc_range_start_addr(start_after);
    // let end: Option<Vec<u8>> = None;
    let end_before: Option<Addr> = None;
    ALLIANCE_MEMBERS
        .range(
            storage,
            start_after.map(|v| Bound::inclusive(v)),
            end_before.map(|v| Bound::inclusive(v)),
            Order::Ascending,
        )
        .take(limit)
        .map(|member| {
            let (addr, mem) = member?;
            Ok(AllianceMemberResponse {
                wallet: std::str::from_utf8(&addr.as_bytes()).unwrap().to_string(),
                name: mem.name,
                logo: mem.logo,
                website: mem.website,
            })
        })
        .collect()
}
