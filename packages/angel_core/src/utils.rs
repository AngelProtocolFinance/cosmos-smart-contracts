use crate::errors::core::ContractError;
use crate::messages::registrar::QueryMsg as RegistrarQuerier;
use crate::messages::vault::{AccountTransferMsg, AccountWithdrawMsg};
use crate::responses::registrar::VaultDetailResponse;
use crate::responses::vault::ExchangeRateResponse;
use crate::structs::{FundingSource, GenericBalance, SplitDetails, StrategyComponent, YieldVault};
use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, QueryRequest, StdResult, SubMsg,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, BalanceResponse, Cw20CoinVerified, Cw20ExecuteMsg};
use terra_cosmwasm::TerraQuerier;

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

pub fn compute_tax(deps: Deps, coin: &Coin) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
    let tax_cap: Uint128 = (terra_querier.query_tax_cap(coin.denom.to_string())?).cap;
    const DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);
    Ok(std::cmp::min(
        (coin.amount.checked_sub(coin.amount.multiply_ratio(
            DECIMAL_FRACTION,
            DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
        )))?,
        tax_cap,
    ))
}

pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: (coin.amount.checked_sub(tax_amount))?,
    })
}

pub fn check_splits(
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
    account_addr: Addr,
    registrar_contract: String,
    beneficiary: &Addr,
    sources: Vec<FundingSource>,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut withdraw_messages = vec![];

    // redeem amounts from sources listed
    for source in sources.iter() {
        if source.locked > Uint128::zero() || source.liquid > Uint128::zero() {
            // check source vault is in registrar vaults list and is approved
            let vault_config: VaultDetailResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: registrar_contract.to_string(),
                    msg: to_binary(&RegistrarQuerier::Vault {
                        vault_addr: source.vault.to_string(),
                    })?,
                }))?;
            let yield_vault: YieldVault = vault_config.vault;

            let withdraw_msg = AccountWithdrawMsg {
                account_addr: account_addr.clone(),
                beneficiary: beneficiary.clone(),
                locked: source.locked,
                liquid: source.liquid,
            };

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
    Ok(withdraw_messages)
}

pub fn deposit_to_vaults(
    deps: Deps,
    registrar_contract: String,
    locked_ust: Coin,
    liquid_ust: Coin,
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

        let transfer_msg = AccountTransferMsg {
            locked: deduct_tax(
                deps,
                Coin {
                    denom: "uusd".to_string(),
                    amount: locked_ust.amount * strategy.locked_percentage,
                },
            )
            .unwrap()
            .amount,
            liquid: deduct_tax(
                deps,
                Coin {
                    denom: "uusd".to_string(),
                    amount: liquid_ust.amount * strategy.liquid_percentage,
                },
            )
            .unwrap()
            .amount,
        };

        // create a deposit message for X Vault, noting amounts for Locked / Liquid
        // funds payload contains both amounts for locked and liquid accounts
        deposit_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: yield_vault.address.to_string(),
            msg: to_binary(&crate::messages::vault::ExecuteMsg::Deposit(
                transfer_msg.clone(),
            ))
            .unwrap(),
            funds: vec![Coin {
                amount: transfer_msg.locked + transfer_msg.liquid,
                denom: "uusd".to_string(),
            }],
        })));
    }
    Ok(deposit_messages)
}
