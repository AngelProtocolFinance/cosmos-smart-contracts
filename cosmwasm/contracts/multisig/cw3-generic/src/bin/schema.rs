use cosmwasm_schema::write_api;
use cw3_generic::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        name: "cw3_generic",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
