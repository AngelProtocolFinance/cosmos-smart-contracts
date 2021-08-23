use cosmwasm_std::{
    coins, from_binary, to_vec, Addr, AllBalanceResponse, BankMsg, Binary, ContractResult, Empty,
    Response, SubMsg,
};

use cosmwasm_vm::{
    call_execute, from_slice,
    testing::{
        execute, instantiate, migrate, mock_env, mock_info, mock_instance,
        mock_instance_with_balances, query, sudo, test_io, MOCK_CONTRACT_ADDR,
    },
    Storage, VmError,
};

static WASM: &[u8] = include_bytes!("../../../../../artifacts/registrar.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);
    // assert_eq!(deps.required_features().len(), 0);

    // let verifier = String::from("verifies");
    // let beneficiary = String::from("benefits");
    // let creator = String::from("creator");
    // let expected_state = State {
    //     verifier: Addr::unchecked(&verifier),
    //     beneficiary: Addr::unchecked(&beneficiary),
    //     funder: Addr::unchecked(&creator),
    // };

    // let msg = InstantiateMsg {
    //     verifier,
    //     beneficiary,
    // };
    // let info = mock_info(&creator, &coins(1000, "earth"));
    // let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    // assert_eq!(res.messages.len(), 0);
    // assert_eq!(res.attributes, [("Let the", "hacking begin")]);

    // // it worked, let's check the state
    // let state: State = deps
    //     .with_storage(|store| {
    //         let data = store
    //             .get(CONFIG_KEY)
    //             .0
    //             .expect("error reading db")
    //             .expect("no data stored");
    //         from_slice(&data, DESERIALIZATION_LIMIT)
    //     })
    //     .unwrap();
    // assert_eq!(state, expected_state);
}
