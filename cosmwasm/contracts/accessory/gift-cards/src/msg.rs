use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use cw_asset::Asset;

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub keeper: String,
}

#[cw_serde]
pub enum ReceiveMsg {
    // Add tokens sent for a specific address
    Deposit { to_address: Option<String> },
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // Add tokens sent for a specific address
    Deposit {
        to_address: Option<String>,
    },
    // Claim a deposited Asset to Addr Balance
    // for now, this is only doable by Keeper Addr
    Claim {
        deposit: u64,
        recipient: String,
    },
    // Spend token/amount specified from sender balance to Endowment
    Spend {
        asset: Asset,
        endow_id: u32,
        locked_percentage: Decimal,
        liquid_percentage: Decimal,
    },
    UpdateConfig {
        owner: Option<String>,
        keeper: Option<String>,
        registrar_contract: Option<String>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Balance { address: String },
    Config {},
    Deposit { deposit_id: u64 },
}
