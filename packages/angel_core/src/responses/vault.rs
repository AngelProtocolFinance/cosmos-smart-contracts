use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub registrar_contract: String,
    pub keeper: String,
    pub pair_contract: String,
    pub lp_staking_contract: String,
    pub lp_reward_token: String,
}
