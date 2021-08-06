use angel_core::structs::{GenericBalance, IndexFund, SplitDetails};
use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw20::Balance;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const CURRENT_DONATIONS: Map<String, GenericBalance> = Map::new("current_donation");

static PREFIX_FUND: &[u8] = b"fund";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,                   // DANO Address
    pub registrar_contract: Addr,      // Address of Registrar SC
    pub terra_alliance: Vec<Addr>,     // Terra Charity Alliance approved addresses
    pub active_fund: Addr,             // index ID of the Active IndexFund
    pub fund_rotation_limit: Uint128, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: u32,       // limit to number of members an IndexFund can have
    pub funding_goal: Option<Balance>, // donation funding limit to trigger early cycle of the Active IndexFund
    pub split_to_liquid: SplitDetails, // default %s to split off into liquid account, if donor provided split is not present
}

// FUND Read/Write
pub fn fund_store(storage: &mut dyn Storage) -> Bucket<IndexFund> {
    bucket(storage, PREFIX_FUND)
}

pub fn fund_read(storage: &dyn Storage) -> ReadonlyBucket<IndexFund> {
    bucket_read(storage, PREFIX_FUND)
}

pub fn read_funds<'a>(storage: &'a dyn Storage) -> StdResult<Vec<IndexFund>> {
    let entries: ReadonlyBucket<'a, IndexFund> = ReadonlyBucket::new(storage, PREFIX_FUND);
    entries
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
