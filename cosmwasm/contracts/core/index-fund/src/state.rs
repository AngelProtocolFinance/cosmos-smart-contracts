use angel_core::structs::IndexFund;
use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const FUND: Map<&[u8], IndexFund> = Map::new("fund");

const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

#[cw_serde]
pub struct OldConfig {
    pub owner: Addr,                   // DANO Address
    pub registrar_contract: Addr,      // Address of Registrar SC
    pub fund_rotation: Option<u64>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,     // limit to number of members an IndexFund can have
    pub funding_goal: Option<Uint128>, // donation funding limit (in UUSD) to trigger early cycle of the Active IndexFund
}

#[cw_serde]
pub struct Config {
    pub owner: Addr,                   // DANO Address
    pub registrar_contract: Addr,      // Address of Registrar SC
    pub fund_rotation: Option<u64>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,     // limit to number of members an IndexFund can have
    pub funding_goal: Option<Uint128>, // donation funding limit (in UUSD) to trigger early cycle of the Active IndexFund
    pub alliance_members: Vec<Addr>,   // angel alliance wallets
}

#[cw_serde]
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
