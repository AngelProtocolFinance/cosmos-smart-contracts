use angel_core::msgs::cw3_apteam::ExecuteMsg;
use cosmwasm_schema::write_api;
use cw3_apteam::msg::{InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        name: "cw3-apteam",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
