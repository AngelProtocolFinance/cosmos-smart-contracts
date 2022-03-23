use crate::structs::{RebalanceDetails, SocialMedialUrls, StrategyComponent};
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StateResponse {
    pub donations_received: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailsResponse {
    pub owner: Addr,
    pub beneficiary: Addr,
    pub name: String,
    pub description: String,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub strategies: Vec<StrategyComponent>,
    pub rebalance: RebalanceDetails,
    pub guardians: Vec<String>,
}

impl EndowmentDetailsResponse {
    pub fn is_guardian(&self, addr: String) -> bool {
        match self.guardians.iter().position(|g| *g == addr) {
            Some(_guardian) => true,
            None => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ProfileResponse {
    pub overview: String,
    pub un_sdg: Option<u64>,
    pub tier: Option<u64>,
    pub charity_logo: String,
    pub charity_image: String,
    pub url: Option<String>,
    pub registration_number: Option<String>,
    pub country_city_origin: Option<String>,
    pub contact_email: Option<String>,
    pub social_media_urls: SocialMedialUrls,
    pub number_of_employees: Option<u64>,
    pub average_annual_budget: Option<String>,
    pub annual_revenue: Option<String>,
    pub charity_navigator_rating: Option<String>,
}
