use crate::errors::ContractError;
use crate::executers;
use crate::msg::{ExchangeRateResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::queriers;
use crate::state::{Config, TokenInfo, CONFIG, TOKEN_INFO};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal256, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, Uint256,
};
use cw2::set_contract_version;
use cw20::Denom;

// version info for future migration info
const CONTRACT_NAME: &str = "mock-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            acct_type: msg.acct_type,
            sibling_vault: env.contract.address,
            input_denom: msg.input_denom,
            next_pending_id: 0,
            last_harvest: env.block.height,
            harvest_to_liquid: msg.harvest_to_liquid,
        },
    )?;

    TOKEN_INFO.save(
        deps.storage,
        &TokenInfo {
            name: msg.name,
            symbol: msg.symbol.clone(),
            decimals: msg.decimals,
            mint: None,
            total_supply: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_attribute("register_vault", msg.symbol))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { new_owner } => executers::update_owner(deps, info, new_owner),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        // -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Deposit { endowment_id } => {
            if info.funds.len() != 1 {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "Invalid: Multiple coins sent. Only accepts a single token as input."
                        .to_string(),
                }));
            }
            let deposit_denom = Denom::Native(info.funds[0].denom.to_string());
            let deposit_amount = info.funds[0].amount;
            let msg_sender = info.sender.to_string();
            executers::deposit(
                deps,
                env,
                info,
                msg_sender,
                endowment_id,
                deposit_denom,
                deposit_amount,
            )
        }
        // Redeem is only called by the SC when setting up new strategies.
        // Pulls all existing strategy amounts back to Account in UST.
        // Then re-Deposits according to the Strategies set.
        // -Deposit Token/Yield Token (Vault) --> +UST (Account) --> -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Redeem {
            endowment_id,
            amount,
        } => executers::redeem(deps, env, info, endowment_id, amount), // -Deposit Token/Yield Token (Account) --> +UST (outside beneficiary)
        _ => unimplemented!(),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VaultConfig {} => to_binary(&queriers::query_vault_config(deps)),
        QueryMsg::Balance { endowment_id } => {
            to_binary(&queriers::query_balance(deps, endowment_id))
        }
        QueryMsg::TokenInfo {} => to_binary(&queriers::query_token_info(deps)),
        // ANCHOR-SPECIFIC QUERIES BELOW THIS POINT!
        QueryMsg::ExchangeRate { input_denom: _ } => to_binary(&ExchangeRateResponse {
            exchange_rate: Decimal256::one(),
            yield_token_supply: Uint256::zero(),
        }),
        _ => unimplemented!(),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
