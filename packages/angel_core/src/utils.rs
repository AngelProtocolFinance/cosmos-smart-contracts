use crate::errors::core::ContractError;
use crate::messages::registrar::QueryMsg as RegistrarQuerier;
use crate::messages::vault::AccountWithdrawMsg;
use crate::responses::registrar::{ConfigResponse as RegistrarConfigResponse, VaultDetailResponse};
use crate::responses::vault::ExchangeRateResponse;
use crate::structs::{FundingSource, GenericBalance, SplitDetails, StrategyComponent, YieldVault};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Decimal256, Deps, DepsMut, QueryRequest,
    StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::{Balance, BalanceResponse, Cw20CoinVerified, Cw20ExecuteMsg, Denom};
use cw_asset::{Asset, AssetInfoBase};

/// The following `calc_range_<???>` functions will set the first key after the provided key, by appending a 1 byte
pub fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

pub fn calc_range_end(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| id.to_be_bytes().to_vec())
}

pub fn calc_range_start_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_bytes().to_vec();
        v.push(1);
        v
    })
}

pub fn calc_range_end_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| addr.as_bytes().to_vec())
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
    if !(max >= min && default <= max && default >= min) {
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
            // msg: to_binary(&crate::messages::vault::QueryMsg::ExchangeRate {
            //     input_denom: Denom::Native("uluna".to_string()),
            // })
            // .unwrap(),
            msg: to_binary("TODO!!!!").unwrap(),
        }))
        .unwrap();
    exchange_rate.exchange_rate
}

pub fn vault_endowment_balance(deps: Deps, vault_address: String, endowment_id: u32) -> Uint128 {
    // get an endowment's balance held with a vault
    let endow_bal_resp: BalanceResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vault_address,
            msg: to_binary(&crate::messages::vault::QueryMsg::Balance { id: endowment_id })
                .unwrap(),
        }))
        .unwrap();
    endow_bal_resp.balance
}

pub fn redeem_from_vaults(
    deps: Deps,
    endowment_id: u32,
    registrar_contract: String,
    strategies: &Vec<StrategyComponent>,
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
            msg: to_binary(&crate::messages::vault::ExecuteMsg::Claim {}).unwrap(),
            funds: vec![],
        })));
        // The "vault" contract now renamed the `redeem` entry to `claim`.
        // New logic is applied for `claim` entry.
        // Hence, this part should be updated after the `vault` implement completes.
    }
    Ok(redeem_messages)
}

pub fn withdraw_from_vaults(
    deps: Deps,
    registrar_contract: String,
    endowment_id: u32,
    beneficiary: &Addr,
    sources: Vec<FundingSource>,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut withdraw_messages = vec![];

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

            // create a withdraw message for X Vault, noting amounts for Locked / Liquid
            withdraw_messages.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: yield_vault.address.to_string(),
                msg: to_binary(&crate::messages::vault::ExecuteMsg::Withdraw(
                    AccountWithdrawMsg {
                        endowment_id: endowment_id.clone(),
                        beneficiary: beneficiary.clone(),
                        amount: source.amount,
                    },
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
    endowment_id: u32,
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
                    msg: to_binary(&crate::messages::vault::ExecuteMsg::Deposit {
                        endowment_id: endowment_id.clone(),
                    })
                    .unwrap(),
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
                        msg: to_binary(&crate::messages::vault::ExecuteMsg::Deposit {
                            endowment_id: endowment_id.clone(),
                        })
                        .unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                })));
            }
            AssetInfoBase::Cw1155(_, _) => unimplemented!(),
        }
    }
    Ok(deposit_messages)
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
        AssetInfoBase::Cw1155(_, _) => unimplemented!(),
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
            query_balance(&deps, account_addr, denom.to_string()).unwrap_or(Uint128::zero())
        }
        Denom::Cw20(contract_addr) => {
            query_token_balance(&deps, contract_addr.to_string(), account_addr)
                .unwrap_or(Uint128::zero())
        }
    }
}

/// Returns a native token's balance for a specific account.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **account_addr** is an object of type [`String`].
///
/// * **denom** is an object of type [`String`] used to specify the denomination used to return the balance (e.g uluna).
pub fn query_balance(deps: &DepsMut, account_addr: String, denom: String) -> StdResult<Uint128> {
    Ok(deps
        .querier
        .query_balance(account_addr, denom)
        .map(|c| c.amount)
        .unwrap_or(Uint128::zero()))
}

/// Returns a token balance for an account.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **contract_addr** is an object of type [`String`]. This is the token contract for which we return a balance.
///
/// * **account_addr** is an object of type [`String`] for which we query the token balance for.
pub fn query_token_balance(
    deps: &DepsMut,
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
