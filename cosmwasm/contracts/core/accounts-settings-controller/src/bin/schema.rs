use angel_core::msgs::accounts_settings_controller::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        name: "accounts-settings-controller",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        migrate: MigrateMsg,
    }
}
