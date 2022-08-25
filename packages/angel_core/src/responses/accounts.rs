use crate::structs::{AccountStrategies, RebalanceDetails, SocialMedialUrls, TransactionRecord};
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StateResponse {
    pub donations_received: Uint128,
    pub closing_endowment: bool,
    pub closing_beneficiary: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailsResponse {
    pub owner: Addr,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub strategies: AccountStrategies,
    pub rebalance: RebalanceDetails,
    pub kyc_donors_only: bool,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub pending_redemptions: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ProfileResponse {
    pub name: String,
    pub overview: String,
    pub un_sdg: Option<u8>,
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct TxRecordsResponse {
    pub txs: Vec<TransactionRecord>,
}
