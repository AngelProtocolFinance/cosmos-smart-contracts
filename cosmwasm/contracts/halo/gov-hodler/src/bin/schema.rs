use cosmwasm_schema::write_api;
use halo_token::gov_hodler::{ExecuteMsg, InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        name: "gov-hodler",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
