use cosmwasm_std::{Addr, Binary, Decimal, Deps, StdResult, Storage, Uint128};
use cw0::Duration;
use cw_controllers::Claims;
use cw_storage_plus::{Bound, Item, Map};
use halo_token::common::OrderBy;
use halo_token::gov::{PollStatus, VoterInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const CLAIMS: Claims = Claims::new("claims");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub halo_token: Addr,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub registrar_contract: Addr,
    /// This is the unbonding period of HALO tokens
    /// We need this to only allow claims to be redeemed after this period
    pub unbonding_period: Duration,
    /// Contract that holds HALO Claims while they are maturing
    pub gov_hodler: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128,                        // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub creator: Addr,
    pub status: PollStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub proposal_type: Option<String>,
    pub execute_data: Option<Vec<ExecuteData>>,
    pub deposit_amount: Uint128,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
    pub staked_amount: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct ExecuteData {
    pub order: u64,
    pub contract: Addr,
    pub msg: Binary,
    pub funding_goal: Option<Uint128>,
    pub fund_rotation: Option<u64>,
    pub split_to_liquid: Option<Decimal>,
    pub treasury_tax_rate: Option<Decimal>,
}
impl Eq for ExecuteData {}

impl Ord for ExecuteData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for ExecuteData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExecuteData {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

// "Config" storage item
pub const CONFIG: Item<Config> = Item::new("config");

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

// "state" storage item
pub const STATE: Item<State> = Item::new("state");
pub fn store_state(storage: &mut dyn Storage, data: &State) -> StdResult<()> {
    STATE.save(storage, data)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

// "tmp_poll_id" storage item
pub const TMP_POLL_ID: Item<u64> = Item::new("tmp_poll_id");
pub fn store_tmp_poll_id(storage: &mut dyn Storage, tmp_poll_id: u64) -> StdResult<()> {
    TMP_POLL_ID.save(storage, &tmp_poll_id)
}

pub fn read_tmp_poll_id(storage: &dyn Storage) -> StdResult<u64> {
    TMP_POLL_ID.load(storage)
}

// "poll" storage map.
pub const POLL: Map<&[u8], Poll> = Map::new("poll");
pub fn store_poll(storage: &mut dyn Storage, poll_id: &[u8], poll: &Poll) -> StdResult<()> {
    POLL.save(storage, poll_id, poll)
}

pub fn read_poll(storage: &dyn Storage, poll_id: &[u8]) -> StdResult<Poll> {
    POLL.load(storage, poll_id)
}

// "poll_indexer_store" storage map
pub const POLL_INDEXER_STORE: Map<(&str, &[u8]), bool> = Map::new("poll_indexer_store");
pub fn store_poll_indexer(
    storage: &mut dyn Storage,
    status: &PollStatus,
    poll_id: u64,
    data: &bool,
) -> StdResult<()> {
    POLL_INDEXER_STORE.save(
        storage,
        (status.to_string().as_str(), &poll_id.to_be_bytes()),
        data,
    )
}

pub fn remove_poll_indexer(storage: &mut dyn Storage, status: &PollStatus, poll_id: u64) {
    POLL_INDEXER_STORE.remove(
        storage,
        (status.to_string().as_str(), &poll_id.to_be_bytes()),
    )
}

// "poll_voter" storage map.
pub const POLL_VOTER: Map<(&[u8], &[u8]), VoterInfo> = Map::new("poll_voter");
pub fn read_poll_voter(storage: &dyn Storage, poll_id: u64, user: Addr) -> StdResult<VoterInfo> {
    POLL_VOTER.load(storage, (&poll_id.to_be_bytes(), &user.as_bytes()))
}

pub fn store_poll_voter(
    storage: &mut dyn Storage,
    poll_id: u64,
    user: Addr,
    data: &VoterInfo,
) -> StdResult<()> {
    POLL_VOTER.save(storage, (&poll_id.to_be_bytes(), &user.as_bytes()), data)
}

pub fn remove_poll_voter(storage: &mut dyn Storage, poll_id: u64, user: &Addr) {
    POLL_VOTER.remove(storage, (&poll_id.to_be_bytes(), user.as_bytes()))
}

pub fn read_poll_voters<'a>(
    deps: Deps,
    storage: &'a dyn Storage,
    poll_id: u64,
    start_after: Option<Addr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(Addr, VoterInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
    };

    let voters = POLL_VOTER.prefix(&poll_id.to_be_bytes());
    voters
        .range(
            storage,
            start.and_then(|v| Some(Bound::inclusive(&*v))),
            end.and_then(|v| Some(Bound::inclusive(&*v))),
            order_by.into(),
        )
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((deps.api.addr_validate(&String::from_utf8_lossy(&k))?, v))
        })
        .collect()
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_polls<'a>(
    storage: &'a dyn Storage,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<Poll>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end(start_after), OrderBy::Desc),
    };

    if let Some(status) = filter {
        let poll_indexer = POLL_INDEXER_STORE.prefix(&status.to_string().as_str());
        poll_indexer
            .range(
                storage,
                start.and_then(|v| Some(Bound::inclusive(&*v))),
                end.and_then(|v| Some(Bound::inclusive(&*v))),
                order_by.into(),
            )
            .take(limit)
            .map(|item| {
                let (k, _) = item?;
                read_poll(storage, &k)
            })
            .collect()
    } else {
        POLL.range(
            storage,
            start.and_then(|v| Some(Bound::inclusive(&*v))),
            end.and_then(|v| Some(Bound::inclusive(&*v))),
            order_by.into(),
        )
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
    }
}

// "bank" storage map
pub const BANK: Map<&[u8], TokenManager> = Map::new("bank");
pub fn store_bank(storage: &mut dyn Storage, key: &&[u8], data: &TokenManager) -> StdResult<()> {
    BANK.save(storage, key, data)
}

pub fn read_bank(storage: &dyn Storage, key: &&[u8]) -> StdResult<Option<TokenManager>> {
    BANK.may_load(storage, key)
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| id.to_be_bytes().to_vec())
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    match start_after {
        Some(addr) => {
            let mut v = addr.as_bytes().to_vec();
            v.push(1);
            Some(v)
        }
        _ => None,
    }
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| addr.as_bytes().to_vec())
}
