use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Addr;

use crate::structs::{
    DaoSetup, DonationMatch, EndowmentController, EndowmentFee, SettingsPermissions, SplitDetails,
};

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateEndowmentSettings(CreateEndowSettingsMsg),
    // Update config(owner, ...)
    UpdateConfig(UpdateConfigMsg),
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    UpdateEndowmentController(UpdateEndowmentControllerMsg),
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

#[cw_serde]
pub struct UpdateConfigMsg {
    pub owner: Option<String>,
    pub registrar_contract: Option<String>,
}

#[cw_serde]
pub struct CreateEndowSettingsMsg {
    pub id: u32,
    pub donation_match_active: bool,
    pub donation_match_contract: Option<Addr>,
    pub beneficiaries_allowlist: Vec<String>,
    pub contributors_allowlist: Vec<String>,
    pub maturity_allowlist: Vec<String>,
    pub endowment_controller: EndowmentController,
    pub parent: Option<u32>,
    pub split_to_liquid: Option<SplitDetails>,
    pub ignore_user_splits: bool,
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}

#[cw_serde]
pub struct UpdateEndowmentSettingsMsg {
    pub id: u32,
    pub donation_match_active: Option<bool>,
    pub beneficiaries_allowlist: Option<Vec<String>>,
    pub contributors_allowlist: Option<Vec<String>>,
    pub maturity_allowlist: Option<UpdateMaturityAllowlist>,
    pub split_to_liquid: Option<SplitDetails>,
    pub ignore_user_splits: Option<bool>,
}

#[cw_serde]
pub struct UpdateEndowmentControllerMsg {
    pub id: u32,
    pub endowment_controller: Option<SettingsPermissions>,
    pub name: Option<SettingsPermissions>,
    pub image: Option<SettingsPermissions>,
    pub logo: Option<SettingsPermissions>,
    pub categories: Option<SettingsPermissions>,
    pub kyc_donors_only: Option<SettingsPermissions>,
    pub split_to_liquid: Option<SettingsPermissions>,
    pub ignore_user_splits: Option<SettingsPermissions>,
    pub donation_match_active: Option<SettingsPermissions>,
    pub beneficiaries_allowlist: Option<SettingsPermissions>,
    pub contributors_allowlist: Option<SettingsPermissions>,
    pub maturity_allowlist: Option<SettingsPermissions>,
    pub earnings_fee: Option<SettingsPermissions>,
    pub deposit_fee: Option<SettingsPermissions>,
    pub withdraw_fee: Option<SettingsPermissions>,
    pub aum_fee: Option<SettingsPermissions>,
}

#[cw_serde]
pub struct UpdateEndowmentFeesMsg {
    pub id: u32,
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}

#[cw_serde]
pub struct UpdateMaturityAllowlist {
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    EndowmentSettings {
        id: u32,
    },
    EndowmentController {
        id: u32,
    },
    EndowmentPermissions {
        id: u32,
        setting_updater: Addr,
        endowment_owner: Addr,
    },
}
