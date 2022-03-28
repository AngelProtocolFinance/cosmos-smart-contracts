use angel_core::errors::core::PaymentError;
use cosmwasm_std::{Coin, MessageInfo, Uint128};

/// returns an error if any coins were sent
pub fn nonpayable(info: &MessageInfo) -> Result<(), PaymentError> {
    if info.funds.is_empty() {
        Ok(())
    } else {
        Err(PaymentError::NonPayable {})
    }
}

/// If exactly one coin was sent, returns it regardless of denom.
/// Returns error if 0 or 2+ coins were sent
pub fn one_coin(info: &MessageInfo) -> Result<Coin, PaymentError> {
    match info.funds.len() {
        0 => Err(PaymentError::NoFunds {}),
        1 => {
            let coin = &info.funds[0];
            if coin.amount.is_zero() {
                Err(PaymentError::NoFunds {})
            } else {
                Ok(coin.clone())
            }
        }
        _ => Err(PaymentError::MultipleDenoms {}),
    }
}

/// Requires exactly one denom sent, which matches the requested denom.
/// Returns the amount if only one denom and non-zero amount. Errors otherwise.
pub fn must_pay(info: &MessageInfo, denom: &str) -> Result<Uint128, PaymentError> {
    let coin = one_coin(info)?;
    if coin.denom != denom {
        Err(PaymentError::MissingDenom(denom.to_string()))
    } else {
        Ok(coin.amount)
    }
}

/// Similar to must_pay, but it any payment is optional. Returns an error if a different
/// denom was sent. Otherwise, returns the amount of `denom` sent, or 0 if nothing sent.
pub fn may_pay(info: &MessageInfo, denom: &str) -> Result<Uint128, PaymentError> {
    if info.funds.is_empty() {
        Ok(Uint128::zero())
    } else if info.funds.len() == 1 && info.funds[0].denom == denom {
        Ok(info.funds[0].amount)
    } else {
        // find first mis-match
        let wrong = info.funds.iter().find(|c| c.denom != denom).unwrap();
        Err(PaymentError::ExtraDenom(wrong.denom.to_string()))
    }
}
