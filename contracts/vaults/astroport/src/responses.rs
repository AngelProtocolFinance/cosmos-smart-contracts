use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport::router::SwapOperation;

use angel_core::structs::AccountType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub ibc_relayer: String,
    pub ibc_sender: String,

    pub owner: String,
    pub acct_type: AccountType,
    pub sibling_vault: String,
    pub registrar_contract: String,
    pub keeper: String,
    pub tax_collector: String,

    pub native_token: String,
    pub lp_token: String,
    pub lp_pair_token0: String,
    pub lp_pair_token1: String,
    pub lp_reward_token: String,

    pub reward_to_native_rotue: Vec<SwapOperation>,
    pub native_to_lp0_route: Vec<SwapOperation>,
    pub native_to_lp1_route: Vec<SwapOperation>,

    pub lp_factory_contract: String,
    pub lp_staking_contract: String,
    pub lp_pair_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_lp_amount: String,
    pub total_shares: String,
}
