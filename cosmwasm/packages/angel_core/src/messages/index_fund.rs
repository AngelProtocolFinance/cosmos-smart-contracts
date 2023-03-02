use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub fund_rotation: Option<Option<u64>>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: Option<u32>,     // limit to number of members an IndexFund can have
    pub funding_goal: Option<Option<Uint128>>, // donation funding limit to trigger early cycle of the Active IndexFund
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // updates the owner of the contract
    UpdateOwner {
        new_owner: String,
    },
    // registrar SC can update its addr
    UpdateRegistrar {
        new_registrar: String,
    },
    // Add/remove the Alliance member from the list/vec
    UpdateAllianceMemberList {
        address: Addr,
        action: String,
    },
    UpdateConfig(UpdateConfigMsg),
    // endpoint to remove a single member from all index funds that they may in
    RemoveMember(RemoveMemberMsg),
    // create a new index fund
    CreateFund {
        name: String,
        description: String,
        members: Vec<u32>,
        rotating_fund: Option<bool>,
        split_to_liquid: Option<Decimal>,
        expiry_time: Option<u64>,
        expiry_height: Option<u64>,
    },
    // remove a specific index fund
    RemoveFund {
        fund_id: u64,
    },
    // updates the members in a given index fund
    UpdateMembers {
        fund_id: u64,
        add: Vec<u32>,
        remove: Vec<u32>,
    },
    // directly receive native tokens
    Deposit(DepositMsg),
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    // Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub struct RemoveMemberMsg {
    pub member: u32,
}

#[cw_serde]
pub struct UpdateMembersMsg {
    pub fund_id: u64,
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub fund_rotation: Option<u64>,
    pub fund_member_limit: Option<u32>,
    pub funding_goal: Option<Uint128>,
}

#[cw_serde]
pub enum ReceiveMsg {
    // Donor deposits tokens sent for an Index Fund
    Deposit(DepositMsg),
}

#[cw_serde]
pub struct DepositMsg {
    pub fund_id: Option<u64>,
    pub split: Option<Decimal>,
}

#[cw_serde]
pub enum QueryMsg {
    // builds and returns a Deposit CosmosMsg based on query inputs
    // NOTE: Here, we assume that the user wants to deposit "native token"
    //       Hence, it receives the "token_denom" for building message.
    //       This part is prone to future change so that it can also handle "cw20 token".
    Deposit {
        token_denom: String,
        amount: Uint128,
        fund_id: Option<u64>,
        split: Option<Decimal>,
    },
    // returns a single fund if the ID is valid
    FundDetails {
        fund_id: u64,
    },
    // get all funds a given Endowment ID is involved with
    InvolvedFunds {
        endowment_id: u32,
    },
    // return details on the currently active fund
    ActiveFundDetails {},
    // return state details
    State {},
    // return config details
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub alliance_members: Option<Vec<Addr>>,
}
