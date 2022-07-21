use crate::config;
use crate::executers;
use crate::msg::{InitMsg, MigrateMsg};
use crate::queriers;
use crate::wasmswap;
use crate::wasmswap::InfoResponse;
use angel_core::errors::vault::ContractError;
use angel_core::messages::vault::{ExecuteMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Denom;
use cw_asset::{Asset, AssetInfoBase};

// version info for future migration info
const CONTRACT_NAME: &str = "junoswap_vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let swap_pool_addr = deps.api.addr_validate(&msg.swap_pool_addr)?;
    let swap_pool_info: InfoResponse = deps
        .querier
        .query_wasm_smart(swap_pool_addr.to_string(), &wasmswap::QueryMsg::Info {})?;

    let token1_denom_string = match swap_pool_info.token1_denom {
        Denom::Native(denom) => denom.to_string(),
        Denom::Cw20(addr) => addr.to_string(),
    };
    let token2_denom_string = match swap_pool_info.token2_denom {
        Denom::Native(denom) => denom.to_string(),
        Denom::Cw20(addr) => addr.to_string(),
    };

    let config = config::Config {
        owner: info.sender,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,

        target: swap_pool_addr,
        input_denoms: vec![token1_denom_string, token2_denom_string],
        yield_token: deps.api.addr_validate(&swap_pool_info.lp_token_address)?,
        routes: vec![],

        total_assets: Uint128::zero(),
        total_shares: Uint128::zero(),

        next_pending_id: 0,
        last_harvest: env.block.height,
        last_harvest_fx: None,
        harvest_to_liquid: msg.harvest_to_liquid,
    };

    config::store(deps.storage, &config)?;

    // store token info
    let token_info = config::TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        mint: None,
        total_supply: Uint128::zero(),
    };
    config::TOKEN_INFO.save(deps.storage, &token_info)?;

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
        ExecuteMsg::UpdateOwner { new_owner } => executers::update_owner(deps, info, new_owner),
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        // -Input token(eg. USDC) (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Deposit {} => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::deposit(deps, env, info.clone(), native_fund)
        }
        // Redeem is only called by the SC when setting up new strategies.
        // Pulls all existing strategy amounts back to Account in UST.
        // Then re-Deposits according to the Strategies set.
        // -Deposit Token/Yield Token (Vault) --> +UST (Account) --> -UST (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Redeem { account_addr } => {
            executers::redeem_stable(deps, env, info, account_addr)
        } // -Deposit Token/Yield Token (Account) --> +UST (outside beneficiary)
        ExecuteMsg::Withdraw(msg) => executers::withdraw_stable(deps, env, info, msg), // DP (Account Locked) -> DP (Account Liquid + Treasury Tax)
        ExecuteMsg::Harvest {
            collector_address,
            collector_share,
        } => executers::harvest(deps, env, info, collector_address, collector_share), // DP -> DP shuffle (taxes collected)
        ExecuteMsg::AddLiquidity {
            in_asset,
            out_asset,
            in_asset_bal_before,
            out_asset_bal_before,
        } => executers::add_liquidity(
            deps,
            env,
            info,
            in_asset,
            out_asset,
            in_asset_bal_before,
            out_asset_bal_before,
        ),
    }
}

/// Replies back to the Vault from the Junoswap pool contract:
/// SubMsg IDs are matched back with the PENDING storage to match the
/// incoming and outgoing funds and any further processing steps performed
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    executers::process_junoswap_pool_reply(deps, env, msg.id, msg.result)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)),
        QueryMsg::Balance { address } => to_binary(&queriers::query_balance(deps, address)),
        QueryMsg::TokenInfo {} => to_binary(&queriers::query_token_info(deps)),
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
