use crate::structs::{
    AccountStrategies, BalanceInfo, Beneficiary, Categories, DonationsReceived, EndowmentFee,
    EndowmentStatus, EndowmentType, OneOffVaults, RebalanceDetails,
};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StateResponse {
    pub tokens_on_hand: BalanceInfo,
    pub donations_received: DonationsReceived,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<Beneficiary>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub version: String,
    pub registrar_contract: String,
    pub next_account_id: u32,
    pub max_general_category_id: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentDetailsResponse {
    pub owner: Addr,
    pub name: String,
    pub categories: Categories,
    pub tier: Option<u8>,
    pub endow_type: EndowmentType,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub status: EndowmentStatus,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub maturity_time: Option<u64>,
    pub strategies: AccountStrategies,
    pub oneoff_vaults: OneOffVaults,
    pub rebalance: RebalanceDetails,
    pub kyc_donors_only: bool,
    pub pending_redemptions: u8,
    pub proposal_link: Option<u64>,
    pub referral_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct EndowmentFeesResponse {
    pub earnings_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
}
