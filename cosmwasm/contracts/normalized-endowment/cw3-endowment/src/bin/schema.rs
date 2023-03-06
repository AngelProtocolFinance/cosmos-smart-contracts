use angel_core::msgs::cw3_multisig::EndowmentInstantiateMsg as InstantiateMsg;
use cosmwasm_schema::write_api;
use cw3_endowment::msg::{ExecuteMsg, MigrateMsg};

fn main() {
    write_api! {
        name: "cw3-endowment",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
