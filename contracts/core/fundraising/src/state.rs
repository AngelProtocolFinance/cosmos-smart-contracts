use angel_core::structs::GenericBalance;
use cosmwasm_std::{Addr, Decimal, Env, Order, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub registrar_contract: Addr,
    /// auto-incrememnted campaign ID (default to 1 at init)
    pub next_id: u64,
    /// max number of days that a campaign can be open for
    /// don't want to allow contributors funds to be locked
    /// up forever
    pub campaign_max_days: u8,
    /// Platform fee charged to AP Treasury
    /// Applied upon successful closing of fundraising
    pub tax_rate: Decimal,
    /// Besides any possible tokens sent with the CreateMsg, this is a list of all cw20 token addresses
    /// that are accepted by the campaign during a top-up. This is required to avoid a DoS attack by topping-up
    /// with an invalid cw20 contract. See https://github.com/CosmWasm/cosmwasm-plus/issues/19
    pub accepted_tokens: GenericBalance,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Campaign {
    pub creator: Addr,
    /// whether the campaign is open for new contributions / top-ups
    pub open: bool,
    /// Whether or not a campaign was successful in fundraising
    /// If TRUE: users can claim their rewards
    /// If FALSE: users can claim back contributed funds
    pub success: bool,
    /// Title of the campaign, for example for a bug bounty "Fix issue in contract.rs"
    pub title: String,
    /// Description of the campaign, a more in depth description of how to meet the campaign condition
    pub description: String,
    /// Image url to use on a fundraising profile page
    pub image_url: String,
    /// When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    /// block time exceeds this value, the campaign is expired.
    /// Once an campaign is expired, it can be returned to the original funder (via "refund").
    pub end_time: u64,
    /// amount / tokens that a campaign is looking to raise in exchange for their reward tokens
    pub funding_goal: GenericBalance,
    /// Balance that represents % of funding goal that a campaign must meet in order to
    /// release their reward tokens to users and to be able to access the contributed funds
    pub funding_threshold: GenericBalance,
    /// Locked Balance in Native and Cw20 tokens
    pub locked_balance: GenericBalance,
    /// Contribution balance in Native and CW20 tokens
    pub contributed_balance: GenericBalance,
    /// All wallets that have contributed to a given campaign
    pub contributors: Vec<Addr>,
}

impl Campaign {
    pub fn is_expired(&self, env: &Env) -> bool {
        if env.block.time > Timestamp::from_seconds(self.end_time) {
            return true;
        }
        false
    }

    pub fn human_whitelist(&self) -> Vec<String> {
        self.funding_goal
            .cw20
            .iter()
            .map(|a| a.to_string())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContributorInfo {
    pub campaign: u64,
    pub balance: GenericBalance,
    pub rewards_claimed: bool,
    pub contribution_refunded: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CAMPAIGNS: Map<u64, Campaign> = Map::new("campaign");
pub const CONTRIBUTORS: Map<&Addr, Vec<ContributorInfo>> = Map::new("contributions");

/// This returns the list of ids for all registered campaigns
pub fn all_campaign_ids(storage: &dyn Storage) -> StdResult<Vec<u64>> {
    CAMPAIGNS
        .keys(storage, None, None, Order::Ascending)
        .collect()
}
