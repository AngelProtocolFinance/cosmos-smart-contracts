use cosmwasm_schema::{cw_serde};

use crate::structs::{AccountType, SwapOperation};

#[cw_serde]
pub struct ConfigResponse {
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

    pub minimum_initial_deposit: String,
    pub pending_owner: String,
    pub pending_owner_deadline: u64,
}

#[cw_serde]
pub struct StateResponse {
    pub total_lp_amount: String,
    pub total_shares: String,
}
