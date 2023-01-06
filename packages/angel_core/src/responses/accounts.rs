use crate::structs::{
    AccountStrategies, Beneficiary, Categories, DonationsReceived, EndowmentEntry, EndowmentStatus,
    EndowmentType, OneOffVaults, RebalanceDetails,
};

use cosmwasm_std::Addr;
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct EndowmentListResponse {
    pub endowments: Vec<EndowmentEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailsResponse {
    pub owner: Addr,
    pub status: EndowmentStatus,
    pub endow_type: EndowmentType,
    pub maturity_time: Option<u64>,
    pub strategies: AccountStrategies,
    pub oneoff_vaults: OneOffVaults,
    pub rebalance: RebalanceDetails,
    pub kyc_donors_only: bool,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub pending_redemptions: u8,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub name: String,
    pub categories: Categories,
    pub tier: Option<u8>,
    pub proposal_link: Option<u64>,
}
