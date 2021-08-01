use angel_core::registrar_msg::{CreateAccountMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use charity_endowments::state::Config;
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(CreateAccountMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(AssetVault), &out_dir);
    export_schema(&schema_for!(VaultDetailsResponse), &out_dir);
    export_schema(&schema_for!(VaultListResponse), &out_dir);
}
