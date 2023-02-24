use cosmwasm_std::{Addr, IbcPacketAckMsg, Timestamp};
use cw_storage_plus::{Item, Map};
use ica_vaults::utils::AccountData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IbcQueryResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    pub response: IbcPacketAckMsg,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ACCOUNTS: Map<&str, AccountData> = Map::new("accounts");
pub const LATEST_QUERIES: Map<&str, IbcQueryResponse> = Map::new("queries");
