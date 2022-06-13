use angel_core::structs::{
    AcceptedTokens, BalanceInfo, EndowmentFee, Profile, RebalanceDetails, SettingsController,
    StrategyComponent, TransactionRecord,
};
use cosmwasm_std::{Addr, Decimal256, Env, Timestamp, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub accepted_tokens: AcceptedTokens,
    pub deposit_approved: bool, // DANO has approved to receive donations & transact
    pub withdraw_approved: bool, // DANO has approved to withdraw funds
    pub pending_redemptions: Option<u64>,
    pub last_earnings_harvest: u64,
    pub last_harvest_fx: Option<Decimal256>,
    pub settings_controller: SettingsController,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OldConfig {
    pub owner: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub accepted_tokens: AcceptedTokens,
    pub deposit_approved: bool, // DANO has approved to receive donations & transact
    pub withdraw_approved: bool, // DANO has approved to withdraw funds
    pub pending_redemptions: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub owner: Addr,             // address that originally setup the endowment account
    pub dao: Option<Addr>,       // subdao governance contract address
    pub dao_token: Option<Addr>, // dao gov token contract address
    pub donation_match: bool,    // whether to do donation matching
    pub whitelisted_beneficiaries: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub name: String,                          // name of the Charity Endowment
    pub description: String,                   // description of the Charity Endowment
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub strategies: Vec<StrategyComponent>, // list of vaults and percentage for locked/liquid accounts
    pub locked_endowment_configs: Vec<String>, // list of endowment configs that cannot be changed/altered once set at creation
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub earnings_fee: Option<EndowmentFee>, // Earnings Fee
    pub withdraw_fee: Option<EndowmentFee>, // Withdraw Fee
    pub deposit_fee: Option<EndowmentFee>, // Deposit Fee
    pub aum_fee: Option<EndowmentFee>, // AUM(Assets Under Management) Fee
    pub donation_matching_contract: Option<Addr>, // donation matching contract address
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

// This is just for the purpose of "migrate" contract.
// After the contract is migrated into "RC-v1.6", this should be cleaned.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OldState {
    pub donations_received: Uint128,
    pub balances: BalanceInfo,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const ENDOWMENT: Item<Endowment> = Item::new("endowment");
pub const PROFILE: Item<Profile> = Item::new("profile");
