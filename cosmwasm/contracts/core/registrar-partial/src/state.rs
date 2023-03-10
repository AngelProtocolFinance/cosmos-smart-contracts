use angel_core::structs::{NetworkInfo, RegistrarConfigCore, StrategyParams};
use cosmwasm_std::Decimal;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<RegistrarConfigCore> = Item::new("config");
pub const STRATEGIES: Map<&[u8], StrategyParams> = Map::new("strategies");
pub const NETWORK_CONNECTIONS: Map<&str, NetworkInfo> = Map::new("network_connections");
pub const FEES: Map<&str, Decimal> = Map::new("fee");
