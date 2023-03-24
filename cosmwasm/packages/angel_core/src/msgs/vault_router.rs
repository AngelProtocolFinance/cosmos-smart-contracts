use crate::structs::VaultActionData;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub registrar_contract: String,
}

#[cw_serde]
pub enum ReceiveMsg {
    Invest { action: VaultActionData },
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: Option<String>,
        registrar_contract: Option<String>,
    },
    Invest {
        action: VaultActionData,
    },
    Redeem {
        action: VaultActionData,
    },
    RedeemAll {
        action: VaultActionData,
    },
    Harvest {
        action: VaultActionData,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {}
