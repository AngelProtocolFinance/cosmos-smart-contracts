use crate::contract::instantiate;
use crate::msg::InitMsg;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Decimal;

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();
    let instantiate_msg = InitMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 6,
        junoswap_pool: "junoswap-pool".to_string(),
        registrar_contract: "angelprotocolteamdano".to_string(),
        harvest_to_liquid: Decimal::percent(75),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}
