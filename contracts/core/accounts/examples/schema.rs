use std::env::current_dir;
use std::fs::create_dir_all;

use accounts::state::Config;
use angel_core::messages::accounts::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg};
use angel_core::responses::accounts::{
    AccountBalanceResponse, AccountDetailsResponse, AccountListResponse, ConfigResponse,
    EndowmentDetailsResponse,
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

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
    export_schema(&schema_for!(ReceiveMsg), &out_dir);
    export_schema(&schema_for!(AccountDetailsResponse), &out_dir);
    export_schema(&schema_for!(AccountListResponse), &out_dir);
    export_schema(&schema_for!(EndowmentDetailsResponse), &out_dir);
    export_schema(&schema_for!(AccountBalanceResponse), &out_dir);
}
