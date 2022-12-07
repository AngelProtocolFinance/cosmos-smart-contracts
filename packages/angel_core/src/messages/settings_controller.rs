use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::structs::{DaoSetup, DonationMatch, EndowmentFee, SettingsController, SplitDetails};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Update config(owner, ...)
    UpdateConfig(UpdateConfigMsg),
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    // Update various "EndowmentFee"s
    UpdateEndowmentFees(UpdateEndowmentFeesMsg),
    // Set up dao token for "Endowment"
    SetupDao {
        endowment_id: u32,
        setup: DaoSetup,
    },
    // Setup Donation match contract for the Endowment
    SetupDonationMatch {
        endowment_id: u32,
        setup: DonationMatch,
    },
    UpdateDelegate {
        endowment_id: u32,
        setting: String,
        action: String,
        delegate_address: String,
        delegate_expiry: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub registrar_contract: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentSettingsMsg {
    pub id: u32,
    pub donation_match_active: bool,
    pub whitelisted_beneficiaries: Option<Vec<String>>,
    pub whitelisted_contributors: Option<Vec<String>>,
    pub maturity_whitelist: Option<UpdateMaturityWhitelist>,
    pub settings_controller: Option<SettingsController>,
    pub parent: Option<u64>,
    pub split_to_liquid: Option<SplitDetails>,
    pub ignore_user_split: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentFeesMsg {
    pub id: u32,
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateMaturityWhitelist {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    EndowmentSettings { id: u32 },
}
