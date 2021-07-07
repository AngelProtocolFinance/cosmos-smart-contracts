use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use locked_account::msg::{
    ConfigResponse, CreateAcctMsg, DetailsResponse, ExecuteMsg, InstantiateMsg, ListResponse,
    MigrateMsg, QueryMsg, ReceiveMsg, UpdateConfigMsg,
};
use locked_account::state::{Account, Config, SplitParameters, Splits};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(Splits), &out_dir);
    export_schema(&schema_for!(SplitParameters), &out_dir);
    export_schema(&schema_for!(Account), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(CreateAcctMsg), &out_dir);
    export_schema(&schema_for!(DetailsResponse), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ListResponse), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ReceiveMsg), &out_dir);
    export_schema(&schema_for!(UpdateConfigMsg), &out_dir);
}
