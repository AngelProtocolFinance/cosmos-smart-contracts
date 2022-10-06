use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;
use cw_asset::AssetInfoBase as CwAssetInfoBase;

use angel_core::errors::vault::ContractError;

use crate::executers;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg};
use crate::queriers;
use crate::state::{Config, MinterData, State, TokenInfo, APTAX, CONFIG, STATE, TOKEN_INFO};
use crate::structs::{
    asset::{AssetInfo, PairInfo},
    pair::QueryMsg as AstroportPairQueryMsg,
};

// version info for future migration info
const CONTRACT_NAME: &str = "loopswap_vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Store the configuration
    let sibling_vault = match msg.sibling_vault {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => env.contract.address.clone(), // can set later with update_config
    };
    let pair_contract = deps.api.addr_validate(&msg.pair_contract)?;
    let pair_info: PairInfo = deps
        .querier
        .query_wasm_smart(pair_contract.to_string(), &AstroportPairQueryMsg::Pair {})?;

    let config = Config {
        owner: info.sender,
        acct_type: msg.acct_type,
        sibling_vault,
        registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
        keeper: deps.api.addr_validate(&msg.keeper)?,
        tax_collector: deps.api.addr_validate(&msg.tax_collector)?,
        swap_router: deps.api.addr_validate(&msg.swap_router)?,

        ibc_relayer: deps.api.addr_validate(&msg.ibc_relayer)?,
        ibc_sender: deps.api.addr_validate(&msg.ibc_sender)?,

        lp_token: pair_info.liquidity_token,
        lp_pair_token0: pair_info.asset_infos[0].clone(),
        lp_pair_token1: pair_info.asset_infos[1].clone(),
        lp_reward_token: deps.api.addr_validate(&msg.lp_reward_token)?,

        native_token: match msg.native_token {
            CwAssetInfoBase::Native(denom) => AssetInfo::NativeToken { denom },
            CwAssetInfoBase::Cw20(contract_addr) => AssetInfo::Token { contract_addr },
            _ => unreachable!(),
        },
        reward_to_native_route: msg.reward_to_native_route,
        native_to_lp0_route: msg.native_to_lp0_route,
        native_to_lp1_route: msg.native_to_lp1_route,

        lp_factory_contract: deps.api.addr_validate(&msg.lp_factory_contract)?,
        lp_staking_contract: deps.api.addr_validate(&msg.lp_staking_contract)?,
        lp_pair_contract: pair_contract,
    };
    CONFIG.save(deps.storage, &config)?;

    // Initialize the contract state
    let state = State {
        total_lp_amount: Uint128::zero(),
        total_shares: Uint128::zero(),
    };
    STATE.save(deps.storage, &state)?;

    APTAX.save(deps.storage, &Uint128::zero())?;

    // Store vault token information
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
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        // -Input token(eg. USDC) (Account) --> +Deposit Token/Yield Token (Vault)
        ExecuteMsg::Deposit { endowment_id } => {
            if info.funds.len() != 1 {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "Invalid: Multiple coins sent. Only accepts a single token as input."
                        .to_string(),
                }));
            }
            let deposit_asset_info = AssetInfo::NativeToken {
                denom: info.funds[0].denom.to_string(),
            };
            let deposit_amount = info.funds[0].amount;
            let msg_sender = info.sender.to_string();
            executers::deposit(
                deps,
                env,
                info,
                msg_sender,
                endowment_id,
                deposit_asset_info,
                deposit_amount,
            )
        }
        // Harvest from "lp_staking" contract `lp_reward`(LOOP) token --> more LPs
        ExecuteMsg::Harvest {} => executers::harvest(deps, env, info),
        // -Deposit Token/Yield Token (Vault) --> + tokens of lp pair --> `accounts_contract`
        ExecuteMsg::Redeem {
            endowment_id,
            amount, // vault tokens to be burned
        } => executers::redeem(deps, env, info, endowment_id, amount),
        // -Deposit Token/Yield Token(Liquid Vault) --> +Deposit Token/Yield Token(Locked Vault)
        ExecuteMsg::ReinvestToLocked {
            endowment_id,
            amount,
        } => executers::reinvest_to_locked_execute(deps, env, info, endowment_id, amount),

        /* --- INTERNAL ENTRIES --- */
        ExecuteMsg::RestakeClaimReward {
            reward_token_bal_before,
        } => executers::restake_claim_reward(deps, env, info, reward_token_bal_before),
        ExecuteMsg::AddLiquidity {
            endowment_id,
            lp_pair_token0_bal_before,
            lp_pair_token1_bal_before,
        } => executers::add_liquidity(
            deps,
            env,
            info,
            endowment_id,
            lp_pair_token0_bal_before,
            lp_pair_token1_bal_before,
        ),
        ExecuteMsg::RemoveLiquidity {
            lp_token_bal_before,
            beneficiary,
            id,
        } => executers::remove_liquidity(deps, env, info, lp_token_bal_before, beneficiary, id),
        ExecuteMsg::Stake {
            endowment_id,
            lp_token_bal_before,
        } => executers::stake_lp_token(deps, env, info, endowment_id, lp_token_bal_before),
        ExecuteMsg::SendAsset {
            beneficiary,
            id,
            native_token_bal_before,
        } => executers::send_asset(deps, env, info, beneficiary, id, native_token_bal_before),
        ExecuteMsg::SwapBack {
            lp_pair_token0_bal_before,
            lp_pair_token1_bal_before,
        } => executers::swap_back(
            deps,
            env,
            info,
            lp_pair_token0_bal_before,
            lp_pair_token1_bal_before,
        ),
    }
}

fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(ReceiveMsg::Deposit { endowment_id }) => {
            let msg_sender = cw20_msg.sender;
            let deposit_asset_info = AssetInfo::Token {
                contract_addr: info.sender.clone(),
            };
            let deposit_amount = cw20_msg.amount;
            executers::deposit(
                deps,
                env,
                info,
                msg_sender,
                endowment_id,
                deposit_asset_info,
                deposit_amount,
            )
        }
        Ok(ReceiveMsg::ReinvestToLocked {
            endowment_id,
            amount,
        }) => {
            executers::reinvest_to_locked_recieve(deps, env, info, endowment_id, amount, cw20_msg)
        }
        Ok(ReceiveMsg::HarvestToLiquid {}) => {
            executers::harvest_to_liquid(deps, env, info, cw20_msg)
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
        QueryMsg::State {} => to_binary(&queriers::query_state(deps)),
        QueryMsg::Balance { endowment_id } => {
            to_binary(&queriers::query_balance(deps, endowment_id))
        }
        QueryMsg::TokenInfo {} => to_binary(&queriers::query_token_info(deps)),
        QueryMsg::TotalBalance {} => to_binary(&queriers::query_total_balance(deps)),
        QueryMsg::ApTaxBalance {} => to_binary(&queriers::query_ap_tax_balance(deps)),
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
