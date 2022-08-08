use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ReceiveMsg, Denom};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg, WasmSwapQueryMsg,
};
use angel_core::responses::vault::InfoResponse;

use crate::executers;
use crate::queriers;
use crate::state::{Config, CONFIG};

// version info for future migration info
const CONTRACT_NAME: &str = "junoswap_vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let swap_pool_addr = deps.api.addr_validate(&msg.swap_pool_addr)?;
    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(swap_pool_addr.to_string(), &WasmSwapQueryMsg::Info {})?;

    if swap_pool_info.token1_denom != msg.output_token_denom
        && swap_pool_info.token2_denom != msg.output_token_denom
    {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: format!(
                "Invalid output_token_denom: {:?}",
                msg.output_token_denom.clone()
            ),
        }));
    }

    let config = Config {
        owner: info.sender,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        keeper: deps.api.addr_validate(&msg.keeper)?,

        pool_addr: swap_pool_addr,
        input_denoms: vec![swap_pool_info.token1_denom, swap_pool_info.token2_denom],
        pool_lp_token_addr: deps.api.addr_validate(&swap_pool_info.lp_token_address)?,
        staking_addr: deps.api.addr_validate(&msg.staking_addr)?,
        routes: vec![],
        output_token_denom: msg.output_token_denom,

        total_assets: Uint128::zero(),
        total_shares: Uint128::zero(),

        last_harvest: env.block.height,
        last_harvest_fx: None,
        harvest_to_liquid: msg.harvest_to_liquid,
    };

    CONFIG.save(deps.storage, &config)?;

    // store token info
    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &token_info)?;

    Ok(Response::new().add_attribute("register_vault", token_info.symbol))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateOwner { new_owner } => executers::update_owner(deps, info, new_owner),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        // -Input token(eg. USDC) (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Deposit {} => {
            if info.funds.len() != 1 {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "Invalid: Multiple coins sent. Only accepts a single token as input."
                        .to_string(),
                }));
            }
            let depositor = info.sender.to_string();
            let deposit_denom = Denom::Native(info.funds[0].denom.to_string());
            let deposit_amount = info.funds[0].amount;
            executers::deposit(deps, env, info, depositor, deposit_denom, deposit_amount)
        }
        // Claim is only called by the SC when setting up new strategies.
        // Pulls all existing amounts back to Account in USDC or [input_denom].
        // -Deposit Token/Yield Token (Vault) --> +USDC (Account)
        ExecuteMsg::Claim { beneficiary } => executers::claim(deps, env, info, beneficiary),
        // -Deposit Token/Yield Token (Account) --> +UST (outside beneficiary)
        ExecuteMsg::Withdraw(msg) => executers::withdraw(deps, env, info, msg),
        ExecuteMsg::Harvest {} => executers::harvest(deps, env, info),
        ExecuteMsg::HarvestSwap {
            token1_denom_bal_before,
            token2_denom_bal_before,
        } => executers::harvest_swap(
            deps,
            env,
            info,
            token1_denom_bal_before,
            token2_denom_bal_before,
        ),
        ExecuteMsg::DistributeHarvest {
            output_token_bal_before,
        } => executers::distribute_harvest(deps, env, info, output_token_bal_before),
        ExecuteMsg::AddLiquidity {
            depositor,
            in_denom,
            out_denom,
            in_denom_bal_before,
            out_denom_bal_before,
        } => executers::add_liquidity(
            deps,
            env,
            info,
            depositor,
            in_denom,
            out_denom,
            in_denom_bal_before,
            out_denom_bal_before,
        ),
        ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before,
            action,
        } => executers::remove_liquidity(deps, env, info, lp_token_bal_before, action),
        ExecuteMsg::Stake {
            depositor,
            lp_token_bal_before,
        } => executers::stake_lp_token(deps, env, info, depositor, lp_token_bal_before),
        ExecuteMsg::SwapAndSendTo {
            token1_denom_bal_before,
            token2_denom_bal_before,
            beneficiary,
        } => executers::swap_and_send(
            deps,
            env,
            info,
            token1_denom_bal_before,
            token2_denom_bal_before,
            beneficiary,
        ),

        // Cw20_base entries
        ExecuteMsg::Transfer { recipient, amount } => {
            cw20_base::contract::execute_transfer(deps, env, info, recipient, amount)
                .map_err(|e| e.into())
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => cw20_base::contract::execute_send(deps, env, info, contract, amount, msg)
            .map_err(|e| e.into()),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )
        .map_err(|e| e.into()),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_base::allowances::execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )
        .map_err(|e| e.into()),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            cw20_base::allowances::execute_transfer_from(deps, env, info, owner, recipient, amount)
                .map_err(|e| e.into())
        }
        ExecuteMsg::BurnFrom { owner, amount } => {
            cw20_base::allowances::execute_burn_from(deps, env, info, owner, amount)
                .map_err(|e| e.into())
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => {
            cw20_base::allowances::execute_send_from(deps, env, info, owner, contract, amount, msg)
                .map_err(|e| e.into())
        }
    }
}

fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(ReceiveMsg::Deposit {}) => {
            let depositor = cw20_msg.sender;
            let deposit_denom = Denom::Cw20(info.sender.clone());
            let deposit_amount = cw20_msg.amount;
            executers::deposit(deps, env, info, depositor, deposit_denom, deposit_amount)
        }
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: "Invalid call".to_string(),
        })),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)),
        QueryMsg::Balance { address } => to_binary(&queriers::query_balance(deps, address)),
        QueryMsg::TokenInfo {} => to_binary(&queriers::query_token_info(deps)),
        QueryMsg::TotalBalance {} => to_binary(&queriers::query_total_balance(deps)),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Can only upgrade from same type".to_string(),
        }));
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Cannot upgrade from a newer version".to_string(),
        }));
    }

    // set the new version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
