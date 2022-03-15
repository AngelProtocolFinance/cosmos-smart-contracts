use std::collections::HashSet;

use angel_core::messages::registrar::InstantiateMsg;
use cosmwasm_std::{coins, Addr, Decimal, Response};

use cosmwasm_vm::{
    from_slice,
    testing::{instantiate, mock_env, mock_info, mock_instance},
    Storage,
};

const DESERIALIZATION_LIMIT: usize = 20_000;
use crate::state::{Config};

static WASM: &[u8] = include_bytes!("../../../../../artifacts/registrar.wasm");
const MOCK_ACCOUNTS_CODE_ID: u64 = 17;

// #[test]
// fn proper_initialization() {
//     let mut deps = mock_instance(WASM, &[]);
//     assert_eq!(deps.required_features().len(), 1);
//     let required_features: HashSet<String> = [String::from("staking")].iter().cloned().collect();
//     assert_eq!(deps.required_features(), required_features);

//     let ap_team = "angelprotocolteamdano".to_string();
//     let instantiate_msg = InstantiateMsg {
//         accounts_code_id: Some(MOCK_ACCOUNTS_CODE_ID),
//         treasury: ap_team.clone(),
//         default_vault: None,
//         tax_rate: 1,
//     };

//     let info = mock_info(&ap_team.as_ref(), &coins(1000, "earth"));
//     let res: Response = instantiate(&mut deps, mock_env(), info, instantiate_msg).unwrap();
//     assert_eq!(res.messages.len(), 0);

//     let state: Config = deps
//         .with_storage(|store| {
//             let data = store
//                 .get(CONFIG_KEY.as_bytes())
//                 .0
//                 .expect("error reading db")
//                 .expect("no data stored");
//             from_slice(&data, DESERIALIZATION_LIMIT)
//         })
//         .unwrap();

//     let expected_state: Config = Config {
//         owner: Addr::unchecked("angelprotocolteamdano"),
//         index_fund_contract: Addr::unchecked("angelprotocolteamdano"),
//         accounts_code_id: MOCK_ACCOUNTS_CODE_ID,
//         approved_charities: vec![],
//         treasury: Addr::unchecked("angelprotocolteamdano"),
//         tax_rate: Decimal::percent(1),
//         default_vault: Addr::unchecked("angelprotocolteamdano"),
//     };

//     assert_eq!(state, expected_state);
// }
