use cosmwasm_schema::cw_serde;
use std::ops::Deref;

use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg};
use cw4::{Cw4Contract, Member};

use angel_core::msgs::cw4_group::ExecuteMsg;

/// Cw4GroupContract is a wrapper around Cw4Contract that provides a lot of helpers
/// for working with cw4-group contracts.
///
/// It extends Cw4Contract to add the extra calls from cw4-group.
#[cw_serde]
pub struct Cw4GroupContract(pub Cw4Contract);

impl Deref for Cw4GroupContract {
    type Target = Cw4Contract;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Cw4GroupContract {
    pub fn new(addr: Addr) -> Self {
        Cw4GroupContract(Cw4Contract(addr))
    }

    fn encode_msg(&self, msg: ExecuteMsg) -> StdResult<CosmosMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
            funds: vec![],
        }
        .into())
    }

    pub fn update_members(&self, remove: Vec<String>, add: Vec<Member>) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateMembers { remove, add };
        self.encode_msg(msg)
    }
}
