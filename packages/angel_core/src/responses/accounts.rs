use crate::structs::{
    AccountStrategies, BalanceInfo, Beneficiary, Categories, DonationsReceived, EndowmentEntry,
    EndowmentFee, EndowmentStatus, EndowmentType, OneOffVaults, RebalanceDetails,
    SettingsController,
};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StateResponse {
    pub donations_received: DonationsReceived,
    pub balances: BalanceInfo,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<Beneficiary>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub registrar_contract: String,
    pub next_account_id: u32,
    pub max_general_category_id: u8,
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
    pub description: String,
    pub strategies: AccountStrategies,
    pub status: EndowmentStatus,
    pub endow_type: EndowmentType,
    pub maturity_time: Option<u64>,
    pub oneoff_vaults: OneOffVaults,
    pub rebalance: RebalanceDetails,
    pub donation_match_contract: String,
    pub kyc_donors_only: bool,
    pub maturity_whitelist: Vec<String>,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub pending_redemptions: u8,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub name: String,
    pub categories: Categories,
    pub tier: Option<u8>,
    pub copycat_strategy: Option<u32>,
    pub proposal_link: Option<u64>,
    pub parent: Option<u64>,
    pub settings_controller: SettingsController,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentFeesResponse {
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}
