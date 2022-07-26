use angel_core::{
    messages::cw3_multisig::Threshold,
    structs::{BalanceInfo, Profile, RebalanceDetails, StrategyComponent, TransactionRecord},
};
use cosmwasm_std::{Addr, Env, Timestamp, Uint128};
use cw_storage_plus::Item;
use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,             // DANO/AP Team Address
    pub cw4_group: Option<Addr>, // CW4 Group Contract
    pub registrar_contract: Addr,
    pub deposit_approved: bool, // DANO has approved to receive donations & transact
    pub withdraw_approved: bool, // DANO has approved to withdraw funds
    pub pending_redemptions: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub owner: Addr,       // address that originally setup the endowment account
    pub beneficiary: Addr, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub strategies: Vec<StrategyComponent>, // list of vaults and percentage for locked/liquid accounts
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub kyc_donors_only: bool, // allow owner to state a preference for receiving only kyc'd donations (where possible)
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub donations_received: Uint128,
    pub balances: BalanceInfo,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<String>,
    pub transactions: Vec<TransactionRecord>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Cw3MultiSigConfig {
    pub threshold: Threshold,
    pub max_voting_period: Duration,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const ENDOWMENT: Item<Endowment> = Item::new("endowment");
pub const PROFILE: Item<Profile> = Item::new("profile");
pub const CW3MULTISIGCONFIG: Item<Cw3MultiSigConfig> = Item::new("cw3_multisig_config");
