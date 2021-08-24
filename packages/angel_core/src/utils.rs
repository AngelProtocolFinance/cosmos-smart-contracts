use crate::structs::{GenericBalance, SplitDetails};
use cosmwasm_std::{to_binary, Addr, BankMsg, Decimal, StdResult, SubMsg, WasmMsg};
use cw20::Cw20ExecuteMsg;

fn check_splits(
    endowment_splits: SplitDetails,
    user_locked: Decimal,
    user_liquid: Decimal,
) -> (Decimal, Decimal) {
    // check that the split provided by a non-TCA address meets the default
    // split requirements set by the Endowment Account
    if user_liquid > endowment_splits.max || user_liquid < endowment_splits.min {
        (
            Decimal::one() - endowment_splits.default,
            endowment_splits.default,
        )
    } else {
        (user_locked, user_liquid)
    }
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<SubMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}
