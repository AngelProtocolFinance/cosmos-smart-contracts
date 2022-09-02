use angel_core::errors::core::ContractError;
use angel_core::structs::Pair;
use cosmwasm_std::{to_binary, Addr, Deps, StdError};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub registrar_contract: Addr,
    pub accounts_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PAIRS: Map<&[u8], Pair> = Map::new("pairs");

pub fn pair_key(asset_infos: &[AssetInfo; 2]) -> Vec<u8> {
    let mut asset_infos = asset_infos.to_vec();
    asset_infos.sort_by_key(|a| to_binary(a).unwrap());
    to_binary(&asset_infos).unwrap().to_vec()
}

pub fn read_pair(deps: Deps, asset_infos: &[AssetInfo; 2]) -> Result<Pair, ContractError> {
    match PAIRS.load(deps.storage, &pair_key(&asset_infos.clone())) {
        Ok(v) => Ok(v),
        Err(_e) => Err(StdError::generic_err("no pair data stored").into()),
    }
}
