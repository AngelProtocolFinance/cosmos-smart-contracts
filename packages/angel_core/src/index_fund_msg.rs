use crate::structs::{IndexFund, SplitDetails};
use cosmwasm_std::{Addr, Uint128};
use cw20::Balance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub terra_alliance: Option<Vec<Addr>>, // Terra Charity Alliance approved addresses
    pub active_fund_index: Option<Uint128>, // index ID of the Active IndexFund
    pub fund_rotation_limit: Option<Uint128>, // how many blocks are in a rotation cycle for the active IndexFund
    pub fund_member_limit: Option<u32>,       // limit to number of members an IndexFund can have
    pub funding_goal: Option<Option<Balance>>, // donation funding limit to trigger early cycle of the Active IndexFund
    pub split_to_liquid: Option<SplitDetails>, // default %s to split off into liquid account, if donor provided split is not present
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // updates the owner of the contract
    UpdateOwner { new_owner: String },
    // endpoint to remove a single member from all index funds that they may in
    RemoveMember(RemoveMemberMsg),
    // create a new index fund
    CreateFund(CreateFundMsg),
    // remove a specific index fund
    RemoveFund(RemoveFundMsg),
    // updates the members in a given index fund
    UpdateMembers(UpdateMembersMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemoveMemberMsg {
    pub member: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateFundMsg {
    pub fund_id: String,
    pub fund: IndexFund,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemoveFundMsg {
    pub fund_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateMembersMsg {
    pub fund_id: String,
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
