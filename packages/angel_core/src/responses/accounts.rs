use crate::structs::{
    AccountStrategies, Beneficiary, Categories, DonationsReceived, EndowmentEntry, EndowmentFee,
    EndowmentStatus, EndowmentType, EndowmentType, OneOffVaults, RebalanceDetails,
    RebalanceDetails, SettingsController, SocialMedialUrls, SocialMedialUrls, StrategyComponent,
    TransactionRecord,
};
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StateResponse {
    pub donations_received: DonationsReceived,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<Beneficiary>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub last_earnings_harvest: u64,
    pub last_harvest_fx: String,
    pub settings_controller: SettingsController,
    pub pending_redemptions: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct EndowmentListResponse {
    pub endowments: Vec<EndowmentEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailsResponse {
    pub owner: Addr,
    pub dao: Option<Addr>,
    pub dao_token: Option<Addr>,
    pub name: String,
    pub description: String,
    pub withdraw_before_maturity: bool,
    pub strategies: Vec<StrategyComponent>,
    pub status: EndowmentStatus,
    pub endow_type: EndowmentType,
    pub maturity_time: Option<u64>,
    pub strategies: AccountStrategies,
    pub oneoff_vaults: OneOffVaults,
    pub rebalance: RebalanceDetails,
    pub donation_match_contract: String,
    pub kyc_donors_only: bool,
    pub maturity_whitelist: Vec<String>,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub pending_redemptions: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ProfileResponse {
    pub name: String,
    pub overview: String,
    pub categories: Categories,
    pub tier: Option<u8>,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub url: Option<String>,
    pub registration_number: Option<String>,
    pub country_of_origin: Option<String>,
    pub street_address: Option<String>,
    pub contact_email: Option<String>,
    pub social_media_urls: SocialMedialUrls,
    pub number_of_employees: Option<u16>,
    pub average_annual_budget: Option<String>,
    pub annual_revenue: Option<String>,
    pub charity_navigator_rating: Option<String>,
    pub endowment_type: EndowmentType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentFeesResponse {
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}
