use angel_core::msgs::cw4_group::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        name: "cw4-group",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
