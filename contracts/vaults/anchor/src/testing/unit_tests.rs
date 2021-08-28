use crate::contract::instantiate;
use crate::msg::InitMsg;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_msg = InitMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
        moneymarket: "anchorprotocolmoneymrk".to_string(),
        registrar_contract: "angelprotocolteamdano".to_string(),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(1, res.messages.len());
}
