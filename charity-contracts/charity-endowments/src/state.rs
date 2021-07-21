use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Env, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,              // AP TEAM / DANO Address
    pub account_ledgers_sc: Addr, // Address of Account Ledgers SC
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SplitDetails {
    pub max: Decimal,
    pub min: Decimal,
    pub default: Decimal, // for when a split parameter is not provided
}

impl SplitDetails {
    pub fn default() -> Self {
        SplitDetails {
            min: Decimal::zero(),
            max: Decimal::one(),
            default: Decimal::percent(50),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub originator: Addr,  // address that originally setup the endowment account
    pub beneficiary: Addr, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub approved: bool,    // DANO has approved to receive donations & transact
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub split_to_liquid: SplitDetails   // set of max, min, and default Split paramenters to check user defined split input against
}

impl Endowment {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(maturity_height) = self.maturity_height {
            if env.block.height > maturity_height {
                return true;
            }
        }
        if let Some(maturity_time) = self.maturity_time {
            if env.block.time > Timestamp::from_seconds(maturity_time) {
                return true;
            }
        }
        false
    }
}
pub const CONFIG: Item<Config> = Item::new("config");
pub const ENDOWMENTS: Map<String, Endowment> = Map::new("endowment");
