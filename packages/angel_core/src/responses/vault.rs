use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::structs::AccountType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub acct_type: AccountType,
    pub sibling_vault: String,
    pub registrar_contract: String,
    pub keeper: String,
    pub tax_collector: String,
    pub lp_pair_contract: String,
    pub lp_staking_contract: String,
    pub lp_token_contract: String,
    pub lp_reward_token: String,
    pub total_lp_amount: String,
    pub total_shares: String,
}
