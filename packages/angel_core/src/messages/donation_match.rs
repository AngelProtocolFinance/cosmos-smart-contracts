use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// address of the reserve token
    pub reserve_token: String,
    /// address of the [reserve_token]-UST LP pair contract
    pub lp_pair: String,
    /// address of "registrar"
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// DonerMatch will attempt to send reserve tokens to CS/dao-token contract.
    /// You must send only reserve tokens in that message  
    /// `id`    : endowment's unique ID
    /// `amount`: UST amount for reserve tokens  
    /// `donor` : Wallet address, which done donation  
    /// `token` : CS/dao-token address
    DonorMatch {
        id: String,
        amount: Uint128,
        donor: Addr,
        token: Addr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecieveMsg {
    DonorMatch { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the "config"
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub reserve_token: String,
    pub lp_pair: String,
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
