use angel_core::structs::{IndexFund, SplitDetails};
use cosmwasm_std::{Timestamp, Decimal};

use crate::executers::{rotate_fund, calculate_split};

#[test]
fn rotate_funds() {
    let index_fund_1 = IndexFund {
        id: 1,
        name: "Fund #1".to_string(),
        description: "Fund number 1 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
        rotating_fund: Some(true),
    };
    let index_fund_2 = IndexFund {
        id: 2,
        name: "Fund #2".to_string(),
        description: "Fund number 2 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
        rotating_fund: Some(true),
    };

    let new_fund_1 = rotate_fund(
        vec![index_fund_1.clone()],
        1,
        10,
        Timestamp::from_seconds(100),
    );
    assert_eq!(new_fund_1, 1);
    let new_fund_2 = rotate_fund(
        vec![index_fund_1.clone(), index_fund_2.clone()],
        1,
        10,
        Timestamp::from_seconds(100),
    );
    assert_eq!(new_fund_2, 2);
    let new_fund_3 = rotate_fund(
        vec![index_fund_1, index_fund_2],
        2,
        10,
        Timestamp::from_seconds(100),
    );
    assert_eq!(new_fund_3, 1);
}

#[test]
fn rotate_funds_with_expired_funds() {
    let index_fund_1 = IndexFund {
        id: 1,
        name: "Fund #1".to_string(),
        description: "Fund number 1 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: None,
        rotating_fund: Some(true),
    };
    let index_fund_2 = IndexFund {
        id: 2,
        name: "Fund #2".to_string(),
        description: "Fund number 2 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: None,
        expiry_height: Some(10),
        rotating_fund: Some(false),
    };
    let index_fund_3 = IndexFund {
        id: 3,
        name: "Fund #3".to_string(),
        description: "Fund number 3 test rotation".to_string(),
        members: vec![],
        split_to_liquid: None,
        expiry_time: Some(1000),
        expiry_height: Some(1000),
        rotating_fund: Some(true),
    };

    let new_fund_1 = rotate_fund(
        vec![index_fund_1.clone()],
        1,
        100,
        Timestamp::from_seconds(10000),
    );
    assert_eq!(new_fund_1, 1);

    let new_fund_2 = rotate_fund(
        vec![index_fund_2.clone(), index_fund_1.clone()],
        1,
        100,
        Timestamp::from_seconds(10000),
    );
    assert_eq!(new_fund_2, 1);

    let new_fund_3 = rotate_fund(
        vec![index_fund_3, index_fund_1, index_fund_2],
        1,
        100,
        Timestamp::from_seconds(10000),
    );
    assert_eq!(new_fund_3, 1);
}

#[test]
fn test_tca_without_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(calculate_split(true, sc_split, None, None), Decimal::zero());
}
#[test]
fn test_tca_with_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(
        calculate_split(true, sc_split, None, Some(Decimal::percent(42))),
        Decimal::zero()
    );
}
#[test]
fn test_non_tca_with_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(
        calculate_split(false, sc_split, None, Some(Decimal::percent(23))),
        Decimal::percent(23)
    );
}
#[test]
fn test_non_tca_without_split() {
    let sc_split = SplitDetails::default();
    assert_eq!(
        calculate_split(false, sc_split.clone(), None, None),
        sc_split.default
    );
}
