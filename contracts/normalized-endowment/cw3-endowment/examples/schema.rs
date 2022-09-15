use angel_core::messages::cw3_multisig::{EndowmentInstantiateMsg as InstantiateMsg, QueryMsg};
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cw3_endowment::msg::ExecuteMsg;
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema_with_title(&schema_for!(ExecuteMsg), &out_dir, "ExecuteMsg");
    export_schema_with_title(&schema_for!(QueryMsg), &out_dir, "QueryMsg");
}
