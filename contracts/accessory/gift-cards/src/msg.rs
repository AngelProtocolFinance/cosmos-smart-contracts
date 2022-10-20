use angel_core::messages::accounts::DepositMsg;
use angel_core::structs::GenericBalance;
use cosmwasm_schema::QueryResponses;
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Add tokens sent for a specific address
    TopUp { to_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // Add tokens sent for a specific address
    TopUp {
        to_address: String,
    },
    // Spend token/amount specified from sender balance to Endowment
    Spend {
        asset: Asset,
        deposit_msg: DepositMsg,
    },
    UpdateConfig {
        owner: Option<String>,
        registrar_contract: Option<String>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, QueryResponses)]
pub enum QueryMsg {
    #[returns(GenericBalance)]
    Balance { address: String },
}
