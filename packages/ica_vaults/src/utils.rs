use cosmwasm_std::{Coin, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct AccountData {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    /// In normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty.
    ///
    /// Since we do not have a way to validate the remote address format, this
    /// must not be of type `Addr`.
    pub remote_addr: Option<String>,
    pub remote_balance: Vec<Coin>,
}
