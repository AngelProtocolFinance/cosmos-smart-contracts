use accounts_settings_controller::state::Config;
use angel_core::messages::accounts_settings_controller::{ExecuteMsg, InstantiateMsg, QueryMsg};
use angel_core::responses::accounts_settings_controller::{
    ConfigResponse, EndowmentSettingsResponse,
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(EndowmentSettingsResponse), &out_dir);
}
