use crate::errors::core::{ContractError, PaymentError};
use crate::messages::registrar::QueryMsg as RegistrarQuerier;
use crate::messages::vault::AccountWithdrawMsg;
use crate::responses::registrar::{ConfigResponse as RegistrarConfigResponse, VaultDetailResponse};
use crate::responses::vault::ExchangeRateResponse;
use crate::structs::{FundingSource, GenericBalance, SplitDetails, StrategyComponent, YieldVault};
use cosmwasm_std::{
    to_binary, to_vec, Addr, BankMsg, Coin, ContractResult, CosmosMsg, Decimal, Decimal256, Deps,
    Empty, MessageInfo, QueryRequest, StdError, StdResult, SubMsg, SystemError, SystemResult,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, BalanceResponse, Cw20CoinVerified, Cw20ExecuteMsg};
use cw_asset::{Asset, AssetInfoBase};

// this will set the first key after the provided key, by appending a 1 byte
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
    registrar_splits: SplitDetails,
    user_locked: Decimal,
    user_liquid: Decimal,
) -> (Decimal, Decimal) {
    // check that the split provided by a non-TCA address meets the default
    // requirements for splits that is set in the Registrar contract
    if user_liquid > registrar_splits.max || user_liquid < registrar_splits.min {
        (
            Decimal::one() - registrar_splits.default,
            registrar_splits.default,
        )
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

pub fn vault_fx_rate(deps: Deps, vault_address: String) -> Decimal256 {
    // get the vault exchange rate
    let exchange_rate: ExchangeRateResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vault_address,
            msg: to_binary(&crate::messages::vault::QueryMsg::ExchangeRate {
                input_denom: "uusd".to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    exchange_rate.exchange_rate
}

pub fn vault_account_balance(
    deps: Deps,
    vault_address: String,
    account_address: String,
) -> Uint128 {
    // get an account's balance held with a vault
    let account_balance: BalanceResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vault_address,
            msg: to_binary(&crate::messages::vault::QueryMsg::Balance {
                address: account_address,
            })
            .unwrap(),
        }))
        .unwrap();
    account_balance.balance
}

pub fn redeem_from_vaults(
    deps: Deps,
    account_addr: Addr,
    registrar_contract: String,
    strategies: Vec<StrategyComponent>,
) -> Result<Vec<SubMsg>, ContractError> {
    // redeem all amounts from existing strategies
    let mut redeem_messages = vec![];
    for source in strategies.iter() {
        // check source vault is in registrar vaults list and is approved
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: source.vault.to_string(),
                })?,
            }))?;
        let yield_vault: YieldVault = vault_config.vault;
        if !yield_vault.approved {
            return Err(ContractError::InvalidInputs {});
        }
        // create a withdraw message for X Vault, noting amounts for Locked / Liquid
        redeem_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: yield_vault.address.to_string(),
            msg: to_binary(&crate::messages::vault::ExecuteMsg::Redeem {
                account_addr: account_addr.clone(),
            })
            .unwrap(),
            funds: vec![],
        })));
    }
    Ok(redeem_messages)
}

pub fn withdraw_from_vaults(
    deps: Deps,
    registrar_contract: String,
    beneficiary: &Addr,
    sources: Vec<FundingSource>,
    asset_info: AssetInfoBase<Addr>,
) -> Result<(Vec<SubMsg>, Uint128), ContractError> {
    let mut withdraw_messages = vec![];
    let mut tx_amounts = Uint128::zero();

    // redeem amounts from sources listed
    for source in sources.iter() {
        if source.amount > Uint128::zero() {
            // check source vault is in registrar vaults list and is approved
            let vault_config: VaultDetailResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: registrar_contract.to_string(),
                    msg: to_binary(&RegistrarQuerier::Vault {
                        vault_addr: source.vault.to_string(),
                    })?,
                }))?;
            let yield_vault: YieldVault = vault_config.vault;
            if !yield_vault.approved {
                return Err(ContractError::InvalidInputs {});
            }
            let withdraw_msg = AccountWithdrawMsg {
                beneficiary: beneficiary.clone(),
                amount: source.amount,
            };

            tx_amounts += source.amount;

            // create a withdraw message for X Vault, noting amounts for Locked / Liquid
            withdraw_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: yield_vault.address.to_string(),
                msg: to_binary(&crate::messages::vault::ExecuteMsg::Withdraw(
                    withdraw_msg.clone(),
                ))
                .unwrap(),
                funds: vec![],
            })));
        }
    }
    Ok((withdraw_messages, tx_amounts))
}

pub fn deposit_to_vaults(
    deps: Deps,
    registrar_contract: String,
    fund: Asset,
    strategies: &[StrategyComponent],
) -> Result<Vec<SubMsg>, ContractError> {
    let mut deposit_messages = vec![];
    // deposit to the strategies set
    for strategy in strategies.iter() {
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: registrar_contract.clone(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: strategy.vault.to_string(),
                })?,
            }))?;
        let yield_vault: YieldVault = vault_config.vault;

        // create a deposit message for X Vault, noting amounts for Locked / Liquid
        // funds payload contains both amounts for locked and liquid accounts
        match fund.info {
            AssetInfoBase::Native(ref denom) => {
                deposit_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: yield_vault.address.to_string(),
                    msg: to_binary(&crate::messages::vault::ExecuteMsg::Deposit {}).unwrap(),
                    funds: vec![Coin {
                        denom: denom.clone(),
                        amount: fund.amount * strategy.percentage,
                    }],
                })));
            }
            AssetInfoBase::Cw20(ref contract_addr) => {
                deposit_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Send {
                        contract: yield_vault.address.to_string(),
                        amount: fund.amount * strategy.percentage,
                        msg: to_binary(&crate::messages::vault::ExecuteMsg::Deposit {}).unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                })));
            }
        }
    }
    Ok(deposit_messages)
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
    };

    if !is_accepted_token(deps, &token, registrar_contract)? {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!("Not accepted token: {}", token),
        }));
    }

    // Cannot deposit zero amount
    if fund.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    Ok(fund)
}
