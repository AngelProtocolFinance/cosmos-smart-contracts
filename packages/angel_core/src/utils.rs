use crate::messages::registrar::QueryMsg as RegistrarQuerier;
use crate::messages::vault::AccountTransferMsg;
use crate::responses::registrar::VaultDetailResponse;
use crate::responses::vault::ExchangeRateResponse;
use crate::structs::{FundingSource, GenericBalance, RedeemResults, SplitDetails, YieldVault};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, CosmosMsg, Decimal, Deps, QueryRequest, ReplyOn, StdResult, SubMsg,
    Uint128, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg};

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
    registrar_contract: String,
    sources: Vec<FundingSource>,
) -> StdResult<RedeemResults> {
    let mut redeem = RedeemResults {
        messages: vec![],
        total: Uint128::zero(),
    };

    // redeem amounts from sources listed
    for source in sources.iter() {
        // check source vault is in registrar vaults list and is approved
        let vault_config: VaultDetailResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: registrar_contract.to_string(),
                msg: to_binary(&RegistrarQuerier::Vault {
                    vault_addr: source.vault.to_string(),
                })?,
            }))?;
        let yield_vault: YieldVault = vault_config.vault;

        // get the vault exchange rate
        let exchange_rate: Decimal256 = vault_fx_rate(deps, yield_vault.address.to_string());
        redeem.total += source.locked + source.liquid;
        let transfer_msg = AccountTransferMsg {
            locked: Uint256::from(source.locked * Decimal::from(exchange_rate)),
            liquid: Uint256::from(source.liquid * Decimal::from(exchange_rate)),
        };

        // create a withdraw message for X Vault, noting amounts for Locked / Liquid
        redeem.messages.push(SubMsg {
            id: 42,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: yield_vault.address.to_string(),
                msg: to_binary(&transfer_msg).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        });
    }
    Ok(redeem)
}
