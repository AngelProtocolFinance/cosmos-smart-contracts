#![cfg(test)]

use cosmwasm_std::testing::{mock_env, MockApi, StorageFactory};
use cosmwasm_std::{coins, to_binary, Addr, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};

use crate::msg::{
    AccountDetailsResponse, AccountListResponse, ConfigResponse, CreateAcctMsg, ExecuteMsg,
    InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg, UpdateConfigMsg, VaultDetailsResponse,
    VaultListResponse,
};

fn mock_app() -> App {
    let env = mock_env();
    let api = Box::new(MockApi::default());
    let bank = BankKeeper::new();

    App::new(api, env.block, bank, StorageFactory::new())
}

pub fn contract_account_ledgers() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_charity_endowments() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

#[test]
// receive cw20 tokens from safe contract
fn safe_deposit_to_ledger_with_cw20_tokens() {
    let mut router = mock_app();

    // meet the cast of characters
    let ap_team = Addr::unchecked("angelprotocolteamdano");
    let donor = Addr::unchecked("plebAccount");

    // account details for a fictional Charity Endowment
    let eid = String::from("GWRGDRGERGRGRGDRGDRGSGSDFS");
    let account_type = String::from("locked");

    // set AP team's personal balance
    let init_funds = coins(2000, "uusd");
    router.init_bank_balance(&ap_team, init_funds).unwrap();

    // set up safe contract with some tokens (Charity Endowments Contract)
    let charity_endowments_contract = router.store_code(contract_charity_endowments());
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Cash Money".to_string(),
        symbol: "CASH".to_string(),
        decimals: 2,
        initial_balances: vec![Cw20Coin {
            address: ap_team.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
    };
    let safe_addr = router
        .instantiate_contract(
            charity_endowments_contract,
            ap_team.clone(),
            &msg,
            &[],
            "CASH",
        )
        .unwrap();

    // set up Account Ledger contract
    let contract_id = router.store_code(contract_account_ledgers());
    let ledgers_addr = router
        .instantiate_contract(
            contract_id,
            ap_team.clone(),
            &InstantiateMsg {},
            &[],
            "Escrow",
        )
        .unwrap();

    // they are different
    assert_ne!(safe_addr, ledgers_addr);

    // set up cw20 helpers
    let cash = Cw20Contract(safe_addr.clone());

    // ensure our balances
    let ap_team_balance = cash.balance(&router, ap_team.clone()).unwrap();
    assert_eq!(ap_team_balance, Uint128::new(5000));
    let ledgers_balance = cash.balance(&router, ledgers_addr.clone()).unwrap();
    assert_eq!(ledgers_balance, Uint128::zero());

    // send some tokens to create an Account
    let arb = Addr::unchecked("arbiter");
    let ben = String::from("beneficiary");
    let id = "demo".to_string();
    let create_msg = ExecuteMsg::CreateAcct(CreateAcctMsg { eid: eid.clone() });
    let send_msg = Cw20ExecuteMsg::Send {
        contract: ledgers_addr.to_string(),
        amount: Uint128::new(1200),
        msg: to_binary(&create_msg).unwrap(),
    };
    let res = router
        .execute_contract(ap_team.clone(), safe_addr.clone(), &send_msg, &[])
        .unwrap();

    assert_eq!(2, res.events.len());
    println!("{:?}", res.events);
    let cw20_attr = res.custom_attrs(0);
    println!("{:?}", cw20_attr);
    assert_eq!(4, cw20_attr.len());
    let escrow_attr = res.custom_attrs(1);
    println!("{:?}", escrow_attr);
    assert_eq!(2, escrow_attr.len());

    // ensure ledger balances updated
    let ap_team_balance = cash.balance(&router, ap_team.clone()).unwrap();
    assert_eq!(ap_team_balance, Uint128::new(3800));
    let ledgers_balance = cash.balance(&router, ledgers_addr.clone()).unwrap();
    assert_eq!(ledgers_balance, Uint128::new(1200));

    // ensure Accounts were properly created
    let details: ConfigResponse = router
        .wrap()
        .query_wasm_smart(&ledgers_addr, &QueryMsg::Config {})
        .unwrap();

    assert_eq!(eid.clone(), details.eid);
    assert_eq!(account_type.clone(), details.account_type);
    assert_eq!(
        vec![Cw20Coin {
            address: safe_addr.to_string(),
            amount: Uint128::new(1200)
        }],
        details.balance
    );

    // release escrow
    let approve_msg = ExecuteMsg::Approve { id };
    let _ = router
        .execute_contract(arb, ledgers_addr.clone(), &approve_msg, &[])
        .unwrap();

    // ensure balances updated - release to ben
    let ap_team_balance = cash.balance(&router, ap_team).unwrap();
    assert_eq!(ap_team_balance, Uint128::new(3800));
    let ledgers_balance = cash.balance(&router, ledgers_addr).unwrap();
    assert_eq!(ledgers_balance, Uint128::zero());
    let ben_balance = cash.balance(&router, ben).unwrap();
    assert_eq!(ben_balance, Uint128::new(1200));
}

//     let instantiate_msg = InstantiateMsg {};
//     let info = mock_info(ap_team.as_ref(), &coins(100000, "bar_token"));
//     let env = mock_env();
//     let res = instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();
//     assert_eq!(0, res.messages.len());

//     // update the approved coins list and trusted SC addresses
//     let msg = UpdateConfigMsg {
//         charity_endowment_contract: charity_endowment_contract.clone(),
//         index_fund_contract: index_fund_contract.clone(),
//         approved_coins: Some(vec![String::from("earth"), String::from("mars")]),
//     };
//     let res = execute(
//         deps.as_mut(),
//         env.clone(),
//         info.clone(),
//         ExecuteMsg::UpdateConfig(msg),
//     )
//     .unwrap();
//     assert_eq!(0, res.messages.len());

//     .unwrap();
//     assert_eq!(0, res.messages.len());

//     let deposit_msg = DepositMsg {
//         eid: eid.clone(),
//         account_type: account_type.clone(),
//     };

//     // test that non-safe SC addresses cannot deposit directly
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: String::from("mars"),
//         amount: Uint128::new(1000),
//         msg: to_binary(&ExecuteMsg::Deposit(deposit_msg.clone())).unwrap(),
//     });
//     let info = mock_info(pleb.as_ref(), &coins(100000, "mars"));
//     let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//     assert_eq!(err, ContractError::Unauthorized {});

//     // test that approved SC addresses cannot send zero balances
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: String::from("mars"),
//         amount: Uint128::zero(),
//         msg: to_binary(&ExecuteMsg::Deposit(deposit_msg.clone())).unwrap(),
//     });
//     let info = mock_info(charity_endowment_contract.as_ref(), &coins(100000, "mars"));
//     let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
//     assert_eq!(err, ContractError::EmptyBalance {});

//     // test that approved SC addresses cannot deposit unapproved coins
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: String::from("bar_token"),
//         amount: Uint128::new(1000),
//         msg: to_binary(&ExecuteMsg::Deposit(deposit_msg.clone())).unwrap(),
//     });
//     let info = mock_info(index_fund_contract.as_ref(), &coins(100000, "bar_token"));
//     let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
//     assert_eq!(err, ContractError::NotInApprovedCoins {});

//     // test that approved SC addresses can deposit approved coins
//     // with greater than zero balances and are credited to the correct EID's Ledger
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: String::from("mars"),
//         amount: Uint128::new(1000),
//         msg: to_binary(&ExecuteMsg::Deposit(deposit_msg.clone())).unwrap(),
//     });
//     let info = mock_info(charity_endowment_contract.as_ref(), &coins(100000, "mars"));
//     let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
//     assert_eq!(0, res.messages.len());

//     // check deposit saved and can be recalled
//     let res = query_account_details(deps.as_ref(), eid.clone(), account_type.clone()).unwrap();
//     assert_eq!(eid.clone(), res.eid);
//     assert_eq!(account_type.clone(), res.account_type);
//     assert_eq!(
//         vec![Cw20Coin {
//             address: String::from("mars"),
//             amount: Uint128::new(1000)
//         }],
//         res.balance
//     );
// }
