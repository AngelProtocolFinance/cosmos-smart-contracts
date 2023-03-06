use cosmwasm_schema::{cw_serde};
use cw_utils::{Duration, Threshold};

/// We currently take no arguments for migrations

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub group_addr: String,
    pub threshold: Threshold,
    pub max_voting_period: Duration,
}
