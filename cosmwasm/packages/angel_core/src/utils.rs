use crate::errors::core::{ContractError, PaymentError};
use crate::msgs::registrar::{
    ConfigResponse as RegistrarConfigResponse, QueryMsg as RegistrarQuerier,
};
use crate::msgs::vault::QueryMsg as VaultQuerier;
use crate::structs::{GenericBalance, SplitDetails};
use cosmwasm_std::{
    to_binary, to_vec, Addr, BankMsg, Coin, ContractResult, Decimal, Deps, DepsMut, Empty, Event,
    MessageInfo, QueryRequest, StdError, StdResult, SubMsg, SystemError, SystemResult, Uint128,
    WasmMsg, WasmQuery,
};
use cw20::{Balance, Cw20CoinVerified, Cw20ExecuteMsg, Denom};
use cw_asset::{Asset, AssetInfoBase};

/// Determine if a reply event contains a specific key-value pair
pub fn event_contains_attr(event: &Event, key: &str, value: &str) -> bool {
    event
        .attributes
        .iter()
        .any(|attr| attr.key == key && attr.value == value)
}

/// The following `calc_range_<???>` functions will set the first key after the provided key, by appending a 1 byte
pub fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
pub fn calc_range_end(end_before: Option<u64>) -> Option<Vec<u8>> {
    end_before.map(|id| id.to_be_bytes().to_vec())
}

// this will set the first key after the provided key, by appending a 1 byte
pub fn calc_range_start_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_bytes().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
pub fn calc_range_end_addr(end_before: Option<Addr>) -> Option<Vec<u8>> {
    end_before.map(|addr| addr.as_bytes().to_vec())
}

pub fn percentage_checks(val: Decimal) -> Result<Decimal, ContractError> {
    // percentage decimals need to be checked that they are all between zero and one (inclusive)
    if val > Decimal::one() {
        return Err(ContractError::InvalidInputs {});
    }
    Ok(val)
}

pub fn split_checks(
    max: Decimal,
    min: Decimal,
    default: Decimal,
) -> Result<SplitDetails, ContractError> {
    // max musst be less than min
    // min must be less than max
    // default must be somewhere between max & min
    if max < min || default > max || default < min {
        return Err(ContractError::InvalidInputs {});
    }

    Ok(SplitDetails { max, min, default })
}

pub fn ratio_adjusted_balance(balance: Balance, portion: Uint128, total: Uint128) -> Balance {
    let adjusted_balance: Balance = match balance {
        Balance::Native(coins) => {
            let coins: Vec<Coin> = coins
                .0
                .into_iter()
                .map(|mut c: Coin| {
                    c.amount = c.amount.multiply_ratio(portion, total);
                    c
                })
                .collect();
            Balance::from(coins)
        }
        Balance::Cw20(coin) => Balance::Cw20(Cw20CoinVerified {
            address: coin.address,
            amount: coin.amount.multiply_ratio(portion, total),
        }),
    };
    adjusted_balance
}

pub fn check_splits(
    splits: SplitDetails,
    user_locked: Decimal,
    user_liquid: Decimal,
    user_override: bool,
) -> (Decimal, Decimal) {
    // check that the split provided by a user if within the max/min bounds
    if user_liquid > splits.max || user_liquid < splits.min || user_override {
        (Decimal::one() - splits.default, splits.default)
    } else {
        (user_locked, user_liquid)
    }
}

pub fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<SubMsg>> {
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

pub fn vault_endowment_balance(deps: Deps, vault_address: String, endowment_id: u32) -> Uint128 {
    // get an account's balance held with a vault
    deps.querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vault_address,
            msg: to_binary(&VaultQuerier::Balance { endowment_id }).unwrap(),
        }))
        .unwrap()
}

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

/// Check if the given address is contract or not.
pub fn check_is_contract(deps: Deps, address: Addr) -> Result<bool, ContractError> {
    let raw = QueryRequest::<Empty>::Wasm(WasmQuery::ContractInfo {
        contract_addr: address.to_string(),
    });
    match deps.querier.raw_query(&to_vec(&raw)?) {
        SystemResult::Err(SystemError::NoSuchContract { .. }) => Ok(false),
        SystemResult::Err(system_err) => Err(ContractError::Std(StdError::GenericErr {
            msg: format!("Querier system error: {}", system_err),
        })),
        SystemResult::Ok(ContractResult::Err(contract_err))
            if contract_err.contains("not found") =>
        {
            Ok(false)
        }
        SystemResult::Ok(ContractResult::Err(contract_err)) => {
            Err(ContractError::Std(StdError::GenericErr {
                msg: format!("Querier contract error: {}", contract_err),
            }))
        }
        SystemResult::Ok(ContractResult::Ok(_)) => Ok(true),
    }
}

/// Check if the given "token"(denom or contract address) is in "accepted_tokens" list.  
///     "token":              native token denom or cw20 token contract address   
///     "registrar_contract": address of `registrar` contract  
pub fn is_accepted_token(deps: Deps, token: &str, registrar_contract: &str) -> StdResult<bool> {
    let config_response: RegistrarConfigResponse = deps
        .querier
        .query_wasm_smart(registrar_contract.to_string(), &RegistrarQuerier::Config {})?;

    Ok(config_response
        .accepted_tokens
        .native_valid(token.to_string())
        || config_response
            .accepted_tokens
            .cw20_valid(token.to_string()))
}

pub fn validate_deposit_fund(
    deps: Deps,
    registrar_contract: &str,
    fund: Asset,
) -> Result<Asset, ContractError> {
    let token = match fund.info {
        AssetInfoBase::Native(ref denom) => denom.to_string(),
        AssetInfoBase::Cw20(ref contract_addr) => contract_addr.to_string(),
        _ => unreachable!(),
    };

    if !is_accepted_token(deps, &token, registrar_contract)? {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!("Not accepted token: {}", token),
        }));
    }

    // Cannot deposit zero amount
    if fund.amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }

    Ok(fund)
}

/// Returns a `Denom` balance for a specific account.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **account_addr** is an object of type [`String`].
///
/// * **denom** is an object of type [`Denom`] used to specify the denomination used to return the balance.
pub fn query_denom_balance(deps: &DepsMut, denom: &Denom, account_addr: String) -> Uint128 {
    match denom {
        Denom::Native(denom) => {
            query_balance(deps.as_ref(), account_addr, denom.to_string()).unwrap_or(Uint128::zero())
        }
        Denom::Cw20(contract_addr) => {
            query_token_balance(deps.as_ref(), contract_addr.to_string(), account_addr)
                .unwrap_or(Uint128::zero())
        }
    }
}

/// Returns a native token's balance for a specific account.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **account_addr** is an object of type [`String`].
///
/// * **denom** is an object of type [`String`] used to specify the denomination used to return the balance (e.g uluna).
pub fn query_balance(deps: Deps, account_addr: String, denom: String) -> StdResult<Uint128> {
    Ok(deps
        .querier
        .query_balance(account_addr, denom)
        .map(|c| c.amount)
        .unwrap_or(Uint128::zero()))
}

/// Returns a token balance for an account.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **contract_addr** is an object of type [`String`]. This is the token contract for which we return a balance.
///
/// * **account_addr** is an object of type [`String`] for which we query the token balance for.
pub fn query_token_balance(
    deps: Deps,
    contract_addr: String,
    account_addr: String,
) -> StdResult<Uint128> {
    // load balance from the token contract
    let res: cw20::BalanceResponse = deps.querier.query_wasm_smart(
        contract_addr,
        &cw20::Cw20QueryMsg::Balance {
            address: account_addr,
        },
    )?;
    Ok(res.balance)
}
