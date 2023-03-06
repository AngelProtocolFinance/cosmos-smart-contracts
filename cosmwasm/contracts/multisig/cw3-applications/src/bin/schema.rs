use cosmwasm_schema::write_api;
use cw3_applications::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        name: "cw3-applications",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
