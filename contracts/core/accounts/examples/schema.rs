use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use accounts::state::{Account, Config, Endowment, GenericBalance, RebalanceDetails, SplitDetails};
use angel_core::accounts_msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, UpdateConfigMsg};
use angel_core::accounts_rsp::{AccountDetailsResponse, AccountListResponse, ConfigResponse};
use angel_core::structs::{Strategy, StrategyComponent};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(Account), &out_dir);
    export_schema(&schema_for!(AccountDetailsResponse), &out_dir);
    export_schema(&schema_for!(AccountListResponse), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(Endowment), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(GenericBalance), &out_dir);
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(RebalanceDetails), &out_dir);
    export_schema(&schema_for!(ReceiveMsg), &out_dir);
    export_schema(&schema_for!(SplitDetails), &out_dir);
    export_schema(&schema_for!(Strategy), &out_dir);
    export_schema(&schema_for!(StrategyComponent), &out_dir);
    export_schema(&schema_for!(UpdateConfigMsg), &out_dir);
}
