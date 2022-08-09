use angel_core::structs::GenericBalance;
use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub campaign_max_days: u8,
    pub tax_rate: Decimal,
    pub accepted_tokens: GenericBalance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Create(CreateMsg),
    /// Adds all sent native tokens to the campaign (locked)
    TopUp {
        id: u64,
    },
    /// Adds all sent native tokens to the campaign (contributions)
    Contribute {
        id: u64,
    },
    /// Sends respective contributed tokens to the creator of a campaign.
    /// Anyone can do this and succeed, so long as the underlying conditions
    /// to conclude a campaign are met (fund amount raised || time elapsed)
    CloseCampaign {
        /// id is a u64 name for the campaign from create
        id: u64,
    },
    /// Contributors to a campaign may claim their rewards due from the locked
    /// balance once a campaign is closed and met the threshold
    ClaimRewards {
        id: u64,
    },
    /// Contributors to a campaign may claim a refund of all contributions made to
    /// a campaign that has closed but failed to met it's threshold
    RefundContributions {
        id: u64,
    },
    /// Allow registrar contract's owner to update configs of this contract
    UpdateConfig {
        campaign_max_days: u8,
        tax_rate: Decimal,
        accepted_tokens: GenericBalance,
    },
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Create(CreateMsg),
    /// Adds all sent CW20 tokens to the campaign (locked)
    TopUp {
        id: u64,
    },
    /// Adds all sent CW20 tokens to the campaign (contributions)
    Contribute {
        id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateMsg {
    /// Title of the campaign
    pub title: String,
    /// Longer description of the campaign, e.g. what conditions should be met
    pub description: String,
    /// Image url to use on a fundraising profile page
    pub image_url: String,
    /// When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    /// block time exceeds this value, the campaign is expired.
    /// Once an campaign is expired, it can be returned to the original funder (via "refund").
    pub end_time: u64,
    /// Funding goal is the amount & addr/demon that a campaign is looking to raise in exchange for their reward tokens
    /// For simplicity, we'll only accept a single token as the input for a given campaign (for now)
    pub funding_goal: GenericBalance,
    /// Funding rewards threshold to trigger release of locked rewards to users.
    /// Must raise X% of the funding goal to trigger release.
    /// Rolls back contributions and locked funds if not hit.
    pub reward_threshold: Decimal,
}

pub fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 20 {
        return false;
    }
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Show all open campaigns. Return type is ListResponse.
    List {},
    /// Returns the details of the named campaign, error if not created
    /// Return type: DetailsResponse.
    Details { id: u64 },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListResponse {
    /// list all registered ids
    pub campaigns: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DetailsResponse {
    /// id of this campaign
    pub id: u64,
    /// if refunded, locked funds go back to the creator
    /// and contribution funds go back to the participants
    pub creator: String,
    /// Title of the campaign
    pub title: String,
    /// Longer description of the campaign, e.g. what conditions should be met
    pub description: String,
    /// Image url to use on a fundraising profile page
    pub image_url: String,
    /// When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    /// block time exceeds this value, the campaign is expired.
    /// Once an campaign is expired, it can be returned to the original funder (via "refund").
    pub end_time: u64,
    /// amount / tokens that a campaign is looking to raise in exchange for their reward tokens
    pub funding_goal: GenericBalance,
    pub funding_threshold: GenericBalance,
    /// Number of contributor addresses for a give campaign
    pub contributor_count: u64,
    /// Balance of native/cw20 tokens contributed to the fundraising campaign
    pub contributed_balance: GenericBalance,
    /// Balance of native/cw20 tokens locked as fundraising reward
    pub locked_balance: GenericBalance,
}
