use super::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::contract::{execute, instantiate, migrate, query};
use angel_core::errors::core::*;
use angel_core::msgs::accounts::{ConfigResponse, EndowmentDetailsResponse, StateResponse};
use angel_core::msgs::accounts::{
    CreateEndowmentMsg, DepositMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg,
    UpdateEndowmentDetailsMsg, UpdateEndowmentStatusMsg,
};
use angel_core::structs::{
    AccountType, Allowances, Beneficiary, Categories, EndowmentType, SplitDetails,
    StrategyInvestment, SwapOperation,
};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, Coin, Decimal, Env, OwnedDeps, StdError, Uint128,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetUnchecked};
use cw_utils::{Expiration, Threshold};

const AP_TEAM: &str = "juno1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const CHARITY_ID: u32 = 1;
const CHARITY_ADDR: &str = "juno1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v";
const REGISTRAR_CONTRACT: &str = "juno18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "juno17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
const DEPOSITOR: &str = "depositor";
const USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
const STRATEGY_KEY: &str = "strategy-native";

fn create_endowment() -> (
    OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    Env,
    String,
    EndowmentDetailsResponse,
) {
    let mut deps = mock_dependencies(&[]);

    let create_endowment_msg = CreateEndowmentMsg {
        owner: CHARITY_ADDR.to_string(),
        name: "Test Endowment".to_string(),
        endow_type: EndowmentType::Normal,
        categories: Categories {
            sdgs: vec![],
            general: vec![],
        },
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
        maturity_time: Some(mock_env().block.time.seconds() + 10),
        cw4_members: vec![],
        kyc_donors_only: true,
        cw3_threshold: Threshold::AbsolutePercentage {
            percentage: Decimal::percent(10),
        },
        cw3_max_voting_period: 60,
        beneficiaries_allowlist: vec![],
        contributors_allowlist: vec![],
        earnings_fee: None,
        withdraw_fee: None,
        deposit_fee: None,
        aum_fee: None,
        dao: None,
        proposal_link: None,
        endowment_controller: None,
        parent: None,
        split_to_liquid: Some(SplitDetails::default()),
        ignore_user_splits: false,
        referral_id: None,
    };

    // Setup the Accounts Contract
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let env = mock_env();
    let acct_contract = env.contract.address.to_string();
    let _res = instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner_sc: AP_TEAM.to_string(),
            registrar_contract: REGISTRAR_CONTRACT.to_string(),
        },
    )
    .unwrap();

    // create a new endowment
    let info = mock_info(CHARITY_ADDR, &coins(100000, "earth"));
    let _ = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::CreateEndowment(create_endowment_msg),
    )
    .unwrap();

    // pull the new endowment's details
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let value: EndowmentDetailsResponse = from_binary(&res).unwrap();

    (deps, env, acct_contract, value)
}

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        registrar_contract: REGISTRAR_CONTRACT.to_string(),
        owner_sc: CHARITY_ADDR.to_string(),
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len()); // no news is good news! :)
}

#[test]
fn test_update_endowment_details() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    let info = mock_info(&endow_details.owner.to_string(), &coins(100000, "earth"));
    // update the endowment "owner" & "kyc_donors_only"
    let msg = UpdateEndowmentDetailsMsg {
        id: CHARITY_ID,
        owner: Some(CHARITY_ADDR.to_string()),
        kyc_donors_only: Some(false),
        name: Some("Test Endowment".to_string()),
        endow_type: Some("normal".to_string()),
        categories: Some(Categories {
            sdgs: vec![2],
            general: vec![],
        }),
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
        rebalance: None,
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateEndowmentDetails(msg),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Not just anyone can update the Endowment's details! Only Endowment owner can.
    let msg = UpdateEndowmentDetailsMsg {
        id: CHARITY_ID,
        owner: Some(CHARITY_ADDR.to_string()),
        kyc_donors_only: Some(false),
        name: Some("Test Endowment name goes here".to_string()),
        endow_type: Some("normal".to_string()),
        categories: Some(Categories {
            sdgs: vec![2],
            general: vec![],
        }),
        tier: Some(3),
        logo: Some("Some fancy logo".to_string()),
        image: Some("Nice banner image".to_string()),
        rebalance: None,
    };
    let info = mock_info(PLEB, &coins(100000, "earth "));
    // This should fail with an error!
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::UpdateEndowmentDetails(msg),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_update_endowment_status() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // Fail to update the endowment status since caller is not config owner
    let update_endowment_status_msg = UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    };
    let info = mock_info("non-registrar", &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::UpdateEndowmentStatus(update_endowment_status_msg.clone()),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let info = mock_info(AP_TEAM, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::UpdateEndowmentStatus(update_endowment_status_msg),
    )
    .unwrap();
    assert_eq!(0, res.attributes.len());

    // Check the update status
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.deposit_approved, true);
    assert_eq!(endow.withdraw_approved, true);
}

#[test]
fn test_change_configs() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // change the owner to some pleb
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateConfig {
            new_owner: None,
            new_registrar: Some(PLEB.to_string()),
            max_general_category_id: Some(2),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(PLEB, value.registrar_contract);

    // Check that the "PLEB" registrar contract should not be able to affect/update the configs
    let msg = ExecuteMsg::UpdateConfig {
        new_owner: None,
        new_registrar: Some(PLEB.to_string()),
        max_general_category_id: Some(100),
    };
    let info = mock_info(PLEB, &coins(100000, "earth "));
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_change_admin() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // change the admin to some pleb
    let info = mock_info(AP_TEAM, &coins(100000, "earth"));
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateConfig {
            new_owner: Some(PLEB.to_string()),
            new_registrar: None,
            max_general_category_id: None,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // check changes saved and can be recalled
    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(PLEB, value.owner);

    // Original owner should not be able to update the configs now
    let msg = ExecuteMsg::UpdateConfig {
        new_owner: Some(CHARITY_ADDR.to_string()),
        new_registrar: None,
        max_general_category_id: None,
    };
    let info = mock_info(AP_TEAM, &coins(100000, "earth "));
    // This should fail with an error!
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn test_donate() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit" w/o "Auto Invest" turned on. No Vault deposits should take place.
    let donation_amt = 20000_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, USDC));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    assert_eq!(0, res.messages.len());

    // Check the "STATE" for "transactions" field
    let query_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&query_res).unwrap();
    assert_eq!(state.donations_received.locked.u128(), donation_amt / 2);
    assert_eq!(state.donations_received.liquid.u128(), donation_amt / 2);

    // Cannot deposit several tokens at once.
    let info = mock_info(
        DEPOSITOR,
        &[
            Coin {
                denom: USDC.to_string(),
                amount: Uint128::from(10000_u128),
            },
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(10000_u128),
            },
        ],
    );
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let err = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});
}

#[test]
fn test_deposit_cw20() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 20000_u128;
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(donation_amt),
        msg: to_binary(&deposit_msg).unwrap(),
    });
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(0, res.messages.len());
}

#[test]
fn test_withdraw_locked() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 20000_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, USDC));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    // "withdraw"(locked) fails since the endowment is not mature yet.
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(10000_u128),
        }],
    };
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        withdraw_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Endowment is not mature. Cannot withdraw before maturity time is reached."
                .to_string()
        })
    );

    // "withdraw"(locked) fails since the caller is not listed in "maturity_allowlist"
    let mut matured_env = mock_env();
    matured_env.block.time = mock_env().block.time.plus_seconds(1001); // Mock the matured state
    let info = mock_info(&"fake_beneficiary".to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        beneficiary_wallet: Some("fake_beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(10000_u128),
        }],
    };
    let err = execute(
        deps.as_mut(),
        matured_env,
        info.clone(),
        withdraw_msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Sender address is not listed in maturity_allowlist.".to_string()
        })
    );

    // Success to withdraw locked balances
    let mut matured_env = mock_env();
    matured_env.block.time = mock_env().block.time.plus_seconds(1001); // Mock the matured state
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(10000_u128),
        }],
    };
    let res = execute(deps.as_mut(), matured_env.clone(), info, withdraw_msg).unwrap();
    assert_eq!(1, res.messages.len());

    // Try to "withdraw" cw20 tokens
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(10000_u128),
        }],
    };
    let err = execute(deps.as_mut(), env.clone(), info, withdraw_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Deposit cw20 token first & withdraw
    let donation_amt = 20000_u128;
    let info = mock_info("test-cw20", &[]);
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: DEPOSITOR.to_string(),
        amount: Uint128::from(donation_amt),
        msg: to_binary(&deposit_msg).unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(100_u128),
        }],
    };
    let res = execute(deps.as_mut(), env.clone(), info, withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 2);
}

#[test]
fn test_withdraw_liquid() {
    let (mut deps, env, _acct_contract, endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try the "Deposit"
    let donation_amt = 200_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, USDC));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    // "Withdraw"(liquid) fails since the sender/caller is neither of endowment owner or address in "beneficiaries_allowlist"
    let info = mock_info(&"anyone".to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(1000_u128),
        }],
    };
    let err = execute(deps.as_mut(), env.clone(), info, withdraw_liquid_msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Sender is not Endowment owner or is not listed in beneficiary whitelist."
                .to_string()
        })
    );

    // "Withdraw"(liquid) fails since the amount is too big
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(1000_u128),
        }],
    };
    let err = execute(deps.as_mut(), env.clone(), info, withdraw_liquid_msg).unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to withdraw liquid amount
    let info = mock_info(&endow_details.owner.to_string(), &[]);
    let withdraw_liquid_msg = ExecuteMsg::Withdraw {
        id: CHARITY_ID,
        acct_type: AccountType::Liquid,
        beneficiary_wallet: Some("beneficiary".to_string()),
        beneficiary_endow: None,
        assets: vec![AssetUnchecked {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(100_u128),
        }],
    };
    let res = execute(deps.as_mut(), env.clone(), info, withdraw_liquid_msg).unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn test_vault_receipt() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    // Update the Endowment status to APPROVED
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // Try to run "vault_receipt"
    // Fails since no funds
    let info = mock_info("vault", &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::VaultReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidCoinsDeposited {});

    // Success, but no messages since "config.pending_redemptions == None"
    let info = mock_info(
        "vault",
        &[Coin {
            denom: USDC.to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::VaultReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Success, check if the "config.redemptions" is decreased
    let info = mock_info(
        "vault",
        &[Coin {
            denom: USDC.to_string(),
            amount: Uint128::from(100_u128),
        }],
    );
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::VaultReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(0, endow.pending_redemptions);

    // Same logic applies to the cw20 token vault_receipt
    let info = mock_info("test-cw20", &[]);
    let msg = ReceiveMsg::VaultReceipt {
        id: CHARITY_ID,
        acct_type: AccountType::Locked,
    };
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "vault".to_string(),
            msg: to_binary(&msg).unwrap(),
            amount: Uint128::from(100_u128),
        }),
    )
    .unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_close_endowment() {
    let (mut deps, env, acct_contract, _endow_details) = create_endowment();

    // Update the Endowment status
    let info = mock_info(AP_TEAM, &[]);
    let update_status_msg = ExecuteMsg::UpdateEndowmentStatus(UpdateEndowmentStatusMsg {
        endowment_id: CHARITY_ID,
        status: 1,
        beneficiary: None,
    });
    let _res = execute(deps.as_mut(), env.clone(), info, update_status_msg).unwrap();

    // confirm we have true for deposit and withdraw
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.withdraw_approved, true);
    assert_eq!(endow.deposit_approved, true);

    // Fails since external address / non-accounts contract calls the entry
    let info = mock_info(AP_TEAM, &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: Beneficiary::Wallet {
                address: CHARITY_ADDR.to_string(),
            },
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Success
    let info = mock_info(&acct_contract, &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: Beneficiary::Wallet {
                address: CHARITY_ADDR.to_string(),
            },
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Check the config & state
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Endowment { id: CHARITY_ID },
    )
    .unwrap();
    let endow: EndowmentDetailsResponse = from_binary(&res).unwrap();
    assert_eq!(endow.withdraw_approved, true);
    assert_eq!(endow.deposit_approved, false);
    assert_eq!(endow.pending_redemptions, 0);

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.closing_endowment, true);
    assert_eq!(
        state.closing_beneficiary,
        Some(Beneficiary::Wallet {
            address: CHARITY_ADDR.to_string()
        })
    );
}

#[test]
fn test_swap_token() {
    let (mut deps, _, _, _) = create_endowment();

    // Should deposit some funds before swap operation
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("anyone", &coins(1000000000_u128, USDC)),
        ExecuteMsg::Deposit(DepositMsg {
            id: CHARITY_ID,
            locked_percentage: Decimal::percent(100),
            liquid_percentage: Decimal::percent(0),
        }),
    )
    .unwrap();

    // Fail to swap token since non-authorized call
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::from(1000000_u128),
            operations: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to swap token since no operations
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::from(1000000_u128),
            operations: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Fail to swap token since no amount
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::zero(),
            operations: vec![SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native(USDC.to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Succeed to swap token
    let info = mock_info(CHARITY_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapToken {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            amount: Uint128::from(1000000_u128),
            operations: vec![SwapOperation::JunoSwap {
                offer_asset_info: AssetInfo::Native(USDC.to_string()),
                ask_asset_info: AssetInfo::Cw20(Addr::unchecked("loop")),
            }],
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_swap_receipt() {
    let (mut deps, _env, _acct_contract, _endow_details) = create_endowment();

    // Fail to swap receipt since non-authorized call
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            final_asset: Asset::native(USDC, 1000000_u128),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Succeed to swap receipt & update the state
    let info = mock_info("swaps_router_addr", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Locked,
            final_asset: Asset::native(USDC, 1000000_u128),
        },
    )
    .unwrap();

    let info = mock_info("swaps_router_addr", &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SwapReceipt {
            id: CHARITY_ID,
            acct_type: AccountType::Liquid,
            final_asset: Asset::native(USDC, 2000000_u128),
        },
    )
    .unwrap();

    // Check the result(state.balances)
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.tokens_on_hand.liquid.native[0].amount,
        Uint128::from(2000000_u128)
    );
    assert_eq!(
        state.tokens_on_hand.locked.native[0].amount,
        Uint128::from(1000000_u128)
    );

    // Same logic applies to "SwapReceipt" of cw20 tokens
    let info = mock_info("test-cw20", &[]);
    let msg = ReceiveMsg::SwapReceipt {
        id: CHARITY_ID,
        final_asset: Asset {
            info: AssetInfoBase::Cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(1000000_u128),
        },
        acct_type: AccountType::Liquid,
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "swaps_router_addr".to_string(),
            msg: to_binary(&msg).unwrap(),
            amount: Uint128::from(1000000_u128),
        }),
    )
    .unwrap();

    let info = mock_info("test-cw20", &[]);
    let msg = ReceiveMsg::SwapReceipt {
        id: CHARITY_ID,
        final_asset: Asset {
            info: AssetInfoBase::Cw20(Addr::unchecked("test-cw20")),
            amount: Uint128::from(2000000_u128),
        },
        acct_type: AccountType::Locked,
    };
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: "swaps_router_addr".to_string(),
            msg: to_binary(&msg).unwrap(),
            amount: Uint128::from(1000000_u128),
        }),
    )
    .unwrap();

    // Check the result
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.tokens_on_hand.liquid.cw20[0].amount,
        Uint128::from(1000000_u128)
    );
    assert_eq!(
        state.tokens_on_hand.locked.cw20[0].amount,
        Uint128::from(2000000_u128)
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.tokens_on_hand.liquid.cw20.len(), 1);
}

#[test]
fn test_strategies_invest() {
    let (mut deps, _, _, _) = create_endowment();

    // Fail to invest to strategies since no endowment owner calls
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to invest to strategies since not approved
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: "shady-strategy".to_string(),
                locked_amount: Uint128::from(1000000_u128),
                liquid_amount: Uint128::from(0_u128),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Strategy is not approved to accept deposits".to_string(),
        })
    );

    // Fail to invest to strategies since native chain does not have a vault router
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: "wrong-chain-strategy".to_string(),
                locked_amount: Uint128::from(1000000_u128),
                liquid_amount: Uint128::from(0_u128),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Vault Router not set for chain in the Registrar Network Connection.".to_string()
        })
    );

    // Fail to invest to strategies since strategies input is empty
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Fail to invest to strategies since insufficient funds
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: "strategy-ethereum".to_string(),
                locked_amount: Uint128::from(100000_u128),
                liquid_amount: Uint128::from(0_u128),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // first, need to make a deposit of funds
    let donation_amt = 10000_u128;
    let info = mock_info(DEPOSITOR, &coins(donation_amt, USDC));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, deposit_msg).unwrap();

    // Fail to invest to strategies since insufficient funds in locked account
    let info = mock_info(CHARITY_ADDR, &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: STRATEGY_KEY.to_string(),
                locked_amount: Uint128::from(6000_u128), // exceeds the current amount of 5000
                liquid_amount: Uint128::from(2000_u128),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Succeed to invest in a strategy
    let info = mock_info(CHARITY_ADDR, &[]);
    let _res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesInvest {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: STRATEGY_KEY.to_string(),
                locked_amount: Uint128::from(3000_u128),
                liquid_amount: Uint128::from(2000_u128),
            }],
        },
    )
    .unwrap();

    // Check the result(state.tokens_on_hand)
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.tokens_on_hand.locked.native[0].denom,
        USDC.to_string(),
    );
    assert_eq!(
        state.tokens_on_hand.locked.native[0].amount,
        Uint128::from(2000_u128)
    );
    assert_eq!(
        state.tokens_on_hand.liquid.native[0].amount,
        Uint128::from(3000_u128)
    );
}

#[test]
fn test_strategies_redeem() {
    let (mut deps, _, _, _) = create_endowment();

    // Fail to redeem strategies since strategies input is empty
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesRedeem {
            id: CHARITY_ID,
            strategies: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InvalidInputs {});

    // Fail to redeem strategies since no endowment owner calls
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesRedeem {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: STRATEGY_KEY.to_string(),
                locked_amount: Uint128::from(10000_u128),
                liquid_amount: Uint128::from(0_u128),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Fail to redeem strategies since no Vault Router is set for a native strategy
    let info = mock_info(CHARITY_ADDR, &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesRedeem {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: "wrong-chain-strategy".to_string(),
                locked_amount: Uint128::from(10000_u128),
                liquid_amount: Uint128::from(0_u128),
            }],
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Vault Router not set for chain in the Registrar Network Connection.".to_string(),
        })
    );

    // // Fail to invest to strategies since insufficient funds
    // let info = mock_info(CHARITY_ADDR, &[]);
    // let err = execute(
    //     deps.as_mut(),
    //     mock_env(),
    //     info,
    //     ExecuteMsg::StrategiesRedeem {
    //         id: CHARITY_ID,
    //         strategies: vec![StrategyInvestment { strategy_key: STRATEGY_KEY.to_string(), locked_amount: 4200000_u128, liquid_amount: 0_u128 }],
    //     },
    // )
    // .unwrap_err();
    // assert_eq!(err, ContractError::BalanceTooSmall {});

    // Succeed to redeem from strategies
    let info = mock_info(CHARITY_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::StrategiesRedeem {
            id: CHARITY_ID,
            strategies: vec![StrategyInvestment {
                strategy_key: "strategy-native".to_string(),
                locked_amount: Uint128::from(10000_u128),
                liquid_amount: Uint128::from(0_u128),
            }],
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
}

#[test]
fn test_distribute_to_beneficiary() {
    let (mut deps, _, _, _) = create_endowment();

    // Only contract itself can call this entry. In other words, it is internal entry.
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::DistributeToBeneficiary { id: CHARITY_ID },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Since "state.closing_beneficiary" is None, it just defaults the "state.balances".
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::DistributeToBeneficiary { id: CHARITY_ID },
    )
    .unwrap();

    // Set the "closing_beneficiary" for the tests
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::CloseEndowment {
            id: CHARITY_ID,
            beneficiary: Beneficiary::Wallet {
                address: CHARITY_ADDR.to_string(),
            },
        },
    )
    .unwrap();
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.closing_beneficiary,
        Some(Beneficiary::Wallet {
            address: CHARITY_ADDR.to_string()
        })
    );

    // Succeed to distribute to "wallet beneficiary"
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::DistributeToBeneficiary { id: CHARITY_ID },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_manage_allowances() {
    let (mut deps, env, _, _) = create_endowment();

    // Only endowment owner can execute the entry
    let info = mock_info("anyone", &[]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "add".to_string(),
            spender: "spender".to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::from(100_u128),
            },
            expires: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Invalid query(no owner || no spender) just returns EMPTY
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Allowances {
            id: CHARITY_ID,
            spender: "spender".to_string(),
        },
    )
    .unwrap();
    let allowances: Allowances = from_binary(&res).unwrap();
    assert!(allowances.assets.is_empty());

    // Endowment owner can "add" the allowance
    let info = mock_info(CHARITY_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "add".to_string(),
            spender: "spender".to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::from(100_u128),
            },
            expires: None,
        },
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Allowances {
            id: CHARITY_ID,
            spender: "spender".to_string(),
        },
    )
    .unwrap();
    let allowances: Allowances = from_binary(&res).unwrap();
    assert_eq!(allowances.assets.len(), 1);
    assert_eq!(allowances.assets[0].amount, Uint128::from(100_u128));
    assert_eq!(
        allowances.assets[0].info.to_string(),
        format!("native:{}", USDC)
    );
    assert_eq!(allowances.expires[0], Expiration::Never {});

    // Try to re-"add" the allowance
    let info = mock_info(CHARITY_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "add".to_string(),
            spender: "spender".to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::from(100_u128),
            },
            expires: Some(Expiration::AtHeight(env.block.height + 100)),
        },
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Allowances {
            id: CHARITY_ID,
            spender: "spender".to_string(),
        },
    )
    .unwrap();
    let allowances: Allowances = from_binary(&res).unwrap();
    assert_eq!(allowances.assets.len(), 1);
    assert_eq!(allowances.assets[0].amount, Uint128::from(200_u128));
    assert_eq!(
        allowances.assets[0].info.to_string(),
        format!("native:{}", USDC)
    );
    assert_eq!(
        allowances.expires[0],
        Expiration::AtHeight(env.block.height + 100)
    );

    // Cannot "add/remove" the invalid asset amount
    let info = mock_info(CHARITY_ADDR, &[]);
    let _err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "add".to_string(),
            spender: "spender".to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::MAX,
            },
            expires: None,
        },
    )
    .unwrap_err();

    let info = mock_info(CHARITY_ADDR, &[]);
    let _err = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "remove".to_string(),
            spender: "spender".to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::from(1000_u128),
            },
            expires: None,
        },
    )
    .unwrap_err();

    // Endowment owner can "remove" the allowance
    let info = mock_info(CHARITY_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "remove".to_string(),
            spender: "spender".to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::from(60_u128),
            },
            expires: None,
        },
    )
    .unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Allowances {
            id: CHARITY_ID,
            spender: "spender".to_string(),
        },
    )
    .unwrap();
    let allowances: Allowances = from_binary(&res).unwrap();
    assert_eq!(allowances.assets.len(), 1);
    assert_eq!(allowances.assets[0].amount, Uint128::from(140_u128));
    assert_eq!(
        allowances.assets[0].info.to_string(),
        format!("native:{}", USDC)
    );
    assert_eq!(allowances.expires[0], Expiration::Never {});
}

#[test]
fn test_spend_allowance() {
    let donation_amt = 2000_u128;
    let liquid_amt = donation_amt / 2;
    let spender = "spender";
    let spend_amt = 100_u128;

    let (mut deps, env, _, _) = create_endowment();

    // "Deposit" the JUNO tokens
    let info = mock_info(DEPOSITOR, &coins(donation_amt, USDC));
    let deposit_msg = ExecuteMsg::Deposit(DepositMsg {
        id: CHARITY_ID,
        locked_percentage: Decimal::percent(50),
        liquid_percentage: Decimal::percent(50),
    });
    let _res = execute(deps.as_mut(), env.clone(), info, deposit_msg).unwrap();

    // Check the endowment state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.tokens_on_hand.liquid.native, coins(liquid_amt, USDC));

    // "spend_allowance" fails since the sender/caller does not have allowances
    let info = mock_info(&spender.to_string(), &[]);
    let spend_allowance_msg = ExecuteMsg::SpendAllowance {
        endowment_id: CHARITY_ID,
        asset: Asset {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(spend_amt),
        },
    };
    let err = execute(deps.as_mut(), env.clone(), info, spend_allowance_msg).unwrap_err();
    assert_eq!(err, ContractError::NoAllowance {});

    // "Add allowances" for the spender wallet
    let info = mock_info(CHARITY_ADDR, &[]);
    let _ = execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::Allowance {
            endowment_id: CHARITY_ID,
            action: "add".to_string(),
            spender: spender.to_string(),
            asset: Asset {
                info: AssetInfoBase::Native(USDC.to_string()),
                amount: Uint128::from(spend_amt),
            },
            expires: None,
        },
    )
    .unwrap();

    // Check the "allowances" state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Allowances {
            id: CHARITY_ID,
            spender: spender.to_string(),
        },
    )
    .unwrap();
    let allowances: Allowances = from_binary(&res).unwrap();
    assert_eq!(allowances.assets.len(), 1);
    assert_eq!(allowances.assets[0].amount, Uint128::from(spend_amt));
    assert_eq!(
        allowances.assets[0].info.to_string(),
        format!("native:{}", USDC)
    );

    // "spend_allowance" fails when zero amount
    let info = mock_info(&spender.to_string(), &[]);
    let spend_allowance_msg = ExecuteMsg::SpendAllowance {
        endowment_id: CHARITY_ID,
        asset: Asset {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::zero(),
        },
    };
    let err = execute(deps.as_mut(), env.clone(), info, spend_allowance_msg).unwrap_err();
    assert_eq!(err, ContractError::ZeroAmount {});

    // "spend_allowance" fails since the amount is too big
    let info = mock_info(&spender.to_string(), &[]);
    let spend_allowance_msg = ExecuteMsg::SpendAllowance {
        endowment_id: CHARITY_ID,
        asset: Asset {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(liquid_amt + 1),
        },
    };
    let _err = execute(deps.as_mut(), env.clone(), info, spend_allowance_msg).unwrap_err();

    // Succeed to "spend_allowance"
    let info = mock_info(&spender.to_string(), &[]);
    let spend_allowance_msg = ExecuteMsg::SpendAllowance {
        endowment_id: CHARITY_ID,
        asset: Asset {
            info: AssetInfoBase::Native(USDC.to_string()),
            amount: Uint128::from(spend_amt),
        },
    };
    let res = execute(deps.as_mut(), env.clone(), info, spend_allowance_msg).unwrap();
    assert_eq!(1, res.messages.len());

    // Check the "allowances" state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Allowances {
            id: CHARITY_ID,
            spender: spender.to_string(),
        },
    )
    .unwrap();
    let allowances: Allowances = from_binary(&res).unwrap();
    assert_eq!(allowances.assets.len(), 1);
    assert_eq!(
        allowances.assets[0].amount,
        Uint128::from(spend_amt - spend_amt)
    );
    assert_eq!(
        allowances.assets[0].info.to_string(),
        format!("native:{}", USDC)
    );

    // Check the endowment state
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::State { id: CHARITY_ID },
    )
    .unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.tokens_on_hand.liquid.native,
        coins(liquid_amt - spend_amt, USDC)
    );
}

#[test]
fn test_migrate() {
    let (mut deps, env, _acct_contract, _endow_details) = create_endowment();

    let err = migrate(deps.as_mut(), env, MigrateMsg {}).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        })
    );
}
