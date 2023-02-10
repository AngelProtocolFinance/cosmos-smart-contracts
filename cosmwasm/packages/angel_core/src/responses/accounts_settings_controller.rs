use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::structs::{EndowmentFee, SplitDetails};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EndowmentSettingsResponse {
    pub dao: Option<Addr>,
    pub dao_token: Option<Addr>,
    pub donation_match_active: bool,
    pub donation_match_contract: Option<Addr>,
    pub beneficiaries_allowlist: Vec<String>,
    pub contributors_allowlist: Vec<String>,
    pub maturity_allowlist: Vec<Addr>,
    pub earnings_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
    pub parent: Option<u32>,
    pub split_to_liquid: Option<SplitDetails>,
    pub ignore_user_splits: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EndowmentPermissionsResponse {
    pub endowment_controller: bool,
    pub strategies: bool,
    pub beneficiaries_allowlist: bool,
    pub contributors_allowlist: bool,
    pub maturity_allowlist: bool,
    pub earnings_fee: bool,
    pub withdraw_fee: bool,
    pub deposit_fee: bool,
    pub aum_fee: bool,
    pub kyc_donors_only: bool,
    pub name: bool,
    pub image: bool,
    pub logo: bool,
    pub categories: bool,
    pub split_to_liquid: bool,
    pub ignore_user_splits: bool,
}
