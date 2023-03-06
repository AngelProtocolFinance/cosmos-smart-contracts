use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Endowment ID
    pub id: u32,
    /// address of the reserve token
    pub reserve_token: String,
    /// address of the [reserve_token]-UST LP pair contract
    pub lp_pair: String,
    /// address of "registrar"
    pub registrar_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// DonerMatch will attempt to send reserve tokens to CS/dao-token contract.
    /// You must send only reserve tokens in that message  
    /// `endowment_id`: Endowment ID
    /// `amount`: UST amount for reserve tokens  
    /// `donor` : Wallet address, which done donation  
    /// `token` : CS/dao-token address  
    DonorMatch {
        endowment_id: u32,
        amount: Uint128,
        donor: Addr,
        token: Addr,
    },
}

#[cw_serde]
pub enum RecieveMsg {
    DonorMatch { endowment_id: u32 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the "config"
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub reserve_token: String,
    pub lp_pair: String,
    pub registrar_contract: String,
}

#[cw_serde]
pub struct MigrateMsg {}
