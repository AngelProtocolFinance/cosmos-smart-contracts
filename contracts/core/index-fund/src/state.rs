use angel_core::structs::{AcceptedTokens, AllianceMember, GenericBalance, IndexFund};
use angel_core::utils::calc_range_start;
use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const TCA_DONATIONS: Map<String, GenericBalance> = Map::new("tca_donation");
pub const ALLIANCE_MEMBERS: Map<Addr, AllianceMember> = Map::new("alliance_members");

static PREFIX_FUND: &[u8] = b"fund";

const MAX_LIMIT: u64 = 30;
const DEFAULT_LIMIT: u64 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,                     // DANO Address
    pub registrar_contract: Addr,        // Address of Registrar SC
    pub fund_rotation: Option<u64>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,     // limit to number of members an IndexFund can have
    pub funding_goal: Option<Uint128>, // donation funding limit (in UUSD) to trigger early cycle of the Active IndexFund
    pub accepted_tokens: AcceptedTokens, // list of approved native and CW20 coins can accept inward
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub total_funds: u64,
    pub active_fund: u64,          // index ID of the Active IndexFund
    pub round_donations: Uint128,  // total donations given to active charity this round
    pub next_rotation_block: u64,  // block height to perform next rotation on
    pub terra_alliance: Vec<Addr>, // Terra Charity Alliance addresses
    pub next_fund_id: u64,
}

impl State {
    pub fn tca_human_addresses(self) -> Vec<String> {
        self.terra_alliance
            .iter()
            .map(|tca| tca.to_string())
            .collect()
    }
}

// FUND Read/Write
pub fn fund_store(storage: &mut dyn Storage) -> Bucket<IndexFund> {
    bucket(storage, PREFIX_FUND)
}

pub fn fund_read(storage: &dyn Storage) -> ReadonlyBucket<IndexFund> {
    bucket_read(storage, PREFIX_FUND)
}

pub fn read_funds<'a>(
    storage: &'a dyn Storage,
    start_after: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<IndexFund>> {
    let funds: ReadonlyBucket<'a, IndexFund> = ReadonlyBucket::new(storage, PREFIX_FUND);
    funds
        .range(
            calc_range_start(start_after).as_deref(),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
