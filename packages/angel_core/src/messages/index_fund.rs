use crate::structs::{AcceptedTokens, IndexFund};
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub fund_rotation: Option<Option<u64>>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: Option<u32>,     // limit to number of members an IndexFund can have
    pub funding_goal: Option<Option<Uint128>>, // donation funding limit to trigger early cycle of the Active IndexFund
    pub accepted_tokens: Option<AcceptedTokens>, // list of approved native and CW20 coins can accept inward
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // updates the owner of the contract
    UpdateOwner {
        new_owner: String,
    },
    // registrar SC can update its addr
    UpdateRegistrar {
        new_registrar: String,
    },
    // replace TCA Member list with a new one
    UpdateTcaList {
        new_list: Vec<String>,
    },
    UpdateConfig(UpdateConfigMsg),
    // endpoint to remove a single member from all index funds that they may in
    RemoveMember(RemoveMemberMsg),
    // create a new index fund
    CreateFund {
        fund: IndexFund,
    },
    // remove a specific index fund
    RemoveFund {
        fund_id: u64,
    },
    // updates the members in a given index fund
    UpdateMembers {
        fund_id: u64,
        add: Vec<String>,
        remove: Vec<String>,
    },
    // directly receive native tokens
    Deposit(DepositMsg),
    // This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Recieve(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemoveMemberMsg {
    pub member: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateMembersMsg {
    pub fund_id: u64,
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub fund_rotation: Option<u64>,
    pub fund_member_limit: Option<u32>,
    pub funding_goal: Option<Uint128>,
    pub accepted_tokens_native: Option<Vec<String>>,
    pub accepted_tokens_cw20: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Donor deposits tokens sent for an Index Fund
    Deposit(DepositMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub fund_id: Option<u64>,
    pub split: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // builds and returns a Deposit CosmosMsg based on query inputs
    Deposit {
        amount: Uint128,
        fund_id: Option<u64>,
    },
    // returns a list of all funds
    FundsList {},
    // returns a single fund if the ID is valid
    FundDetails {
        fund_id: u64,
    },
    // get all funds a given Accounts SC address is involved with
    InvolvedFunds {
        address: String,
    },
    // return details on the currently active fund
    ActiveFundDetails {},
    // get total donations given to Active Fund for a round
    ActiveFundDonations {},
    // return state details
    State {},
    // return config details
    Config {},
    // return list of TCA Members
    TcaList {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
