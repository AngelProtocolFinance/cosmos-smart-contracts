use crate::executers;
use crate::queriers;
use crate::state::{Config, Endowment, OldConfig, State, CONFIG, ENDOWMENT, PROFILE, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw3_multisig::EndowmentInstantiateMsg as Cw3InstantiateMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::messages::subdao::InstantiateMsg as DaoInstantiateMsg;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::EndowmentType;
use angel_core::structs::{AcceptedTokens, BalanceInfo, RebalanceDetails, SettingsController};
use cosmwasm_std::{
    attr, entry_point, from_binary, from_slice, to_binary, to_vec, Binary, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    Uint128, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;
use cw4::Member;

use cw_asset::{Asset, AssetInfo, AssetInfoBase};

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // check that the "maturity_time" is not empty
    if msg.maturity_time.is_none() {
        return Err(ContractError::Std(StdError::NotFound {
            kind: "maturity_time".to_string(),
        }));
    }

    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.registrar_contract.clone(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&registrar_config.owner)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            accepted_tokens: AcceptedTokens::default(),
            deposit_approved: false,  // bool
            withdraw_approved: false, // bool
            pending_redemptions: None,
            next_account_id: 1_u32,
            max_general_category_id: 1_u8,
            settings_controller: match msg.settings_controller {
                Some(controller) => controller,
                None => SettingsController::default(),
            },
        },
    )?;

    ENDOWMENT.save(
        deps.storage,
        &Endowment {
            owner: deps.api.addr_validate(&msg.owner)?, // Addr
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,           // Option<u64>
            strategies: vec![],
            rebalance: RebalanceDetails::default(),
            dao: None,
            dao_token: None,
            whitelisted_beneficiaries: msg.whitelisted_beneficiaries, // Vec<String>
            whitelisted_contributors: msg.whitelisted_contributors,   // Vec<String>
            donation_match_active: false,
            donation_match_contract: match &msg.profile.endow_type {
                &EndowmentType::Charity => {
                    match &registrar_config.donation_match_charites_contract {
                        Some(match_contract) => Some(deps.api.addr_validate(&match_contract)?),
                        None => None,
                    }
                }
                _ => None,
            },
            earnings_fee: msg.earnings_fee,
            withdraw_fee: msg.withdraw_fee,
            deposit_fee: msg.deposit_fee,
            aum_fee: msg.aum_fee,
            parent: msg.parent,
            kyc_donors_only: false,
            maturity_whitelist: vec![],
        },
    )?;

    STATE.save(
        deps.storage,
        &State {
            donations_received: Uint128::zero(),
            balances: BalanceInfo::default(),
            closing_endowment: false,
            closing_beneficiary: None,
        },
    )?;

    PROFILE.save(deps.storage, &msg.profile)?;

    // initial default Response to add submessages to
    let mut res: Response = Response::new().add_attributes(vec![
        attr("endow_addr", env.contract.address.to_string()),
        attr("endow_name", msg.profile.name),
        attr("endow_type", msg.profile.endow_type.to_string()),
        attr(
            "endow_logo",
            msg.profile.logo.unwrap_or_else(|| "".to_string()),
        ),
        attr(
            "endow_image",
            msg.profile.image.unwrap_or_else(|| "".to_string()),
        ),
        attr(
            "endow_tier",
            msg.profile.tier.unwrap_or_else(|| 0).to_string(),
        ),
        attr(
            "endow_un_sdg",
            msg.profile.un_sdg.unwrap_or_else(|| 0).to_string(),
        ),
    ]);

    if registrar_config.cw3_code.eq(&None) || registrar_config.cw4_code.eq(&None) {
        return Err(ContractError::Std(StdError::generic_err(
            "cw3_code & cw4_code must exist",
        )));
    }

    res = res.add_submessage(SubMsg {
        id: 0,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: registrar_config.cw3_code.unwrap(),
            admin: None,
            label: "new endowment cw3 multisig".to_string(),
            msg: to_binary(&Cw3InstantiateMsg {
                // check if CW3/CW4 codes were passed to setup a multisig/group
                cw4_members: match msg.cw4_members.is_empty() {
                    true => vec![Member {
                        addr: msg.owner.to_string(),
                        weight: 1,
                    }],
                    false => msg.cw4_members,
                },
                cw4_code: registrar_config.cw4_code.unwrap(),
                threshold: msg.cw3_threshold,
                max_voting_period: msg.cw3_max_voting_period,
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    // check if a dao needs to be setup along with a dao token contract
    match (
        msg.dao,
        registrar_config.subdao_bonding_token_code,
        registrar_config.subdao_gov_code,
    ) {
        (Some(dao_setup), Some(_token_code), Some(gov_code)) => {
            res = res.add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: gov_code,
                    admin: None,
                    label: "new endowment dao contract".to_string(),
                    msg: to_binary(&DaoInstantiateMsg {
                        quorum: dao_setup.quorum,
                        threshold: dao_setup.threshold,
                        voting_period: dao_setup.voting_period,
                        timelock_period: dao_setup.timelock_period,
                        expiration_period: dao_setup.expiration_period,
                        proposal_deposit: dao_setup.proposal_deposit,
                        snapshot_period: dao_setup.snapshot_period,
                        endow_type: msg.profile.endow_type.clone(),
                        endow_owner: env.contract.address.to_string(),
                        registrar_contract: msg.registrar_contract.clone(),
                        token: dao_setup.token,
                    })?,
                    funds: vec![],
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            });
        }
        (Some(_dao_setup), None, _) | (Some(_dao_setup), _, None) => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "DAO settings are not yet configured on the Registrar contract".to_string(),
            }));
        }
        _ => (),
    }
    Ok(res)
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
        ExecuteMsg::Deposit(msg) => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfo::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::deposit(deps, env, info.clone(), info.sender, msg, native_fund)
        }
        ExecuteMsg::SwapToken {
            id,
            acct_type,
            amount,
            operations,
        } => executers::swap_token(deps, info, id, acct_type, amount, operations),
        ExecuteMsg::SwapReceipt {
            id,
            final_asset,
            acct_type,
        } => executers::swap_receipt(deps, id, info.sender, final_asset, acct_type),
        ExecuteMsg::VaultReceipt { id, acct_type } => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::vault_receipt(deps, env, id, acct_type, info.sender, native_fund)
        }
        ExecuteMsg::CreateEndowment(msg) => executers::create_endowment(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::DistributeToBeneficiary { id } => {
            executers::distribute_to_beneficiary(deps, env, info, id)
        }
        ExecuteMsg::ReinvestToLocked {
            id,
            amount,
            vault_addr,
        } => executers::reinvest_to_locked(deps, env, info, id, amount, vault_addr),
        ExecuteMsg::Withdraw {
            id,
            acct_type,
            beneficiary,
            assets,
        } => executers::withdraw(deps, env, info, id, acct_type, beneficiary, assets),
        ExecuteMsg::VaultsInvest {
            id,
            acct_type,
            vaults,
        } => executers::vaults_invest(deps, info, id, acct_type, vaults),
        ExecuteMsg::VaultsRedeem {
            id,
            acct_type,
            vaults,
        } => executers::vaults_redeem(deps, env, info, id, acct_type, vaults),
        ExecuteMsg::UpdateConfig {
            new_registrar,
            max_general_category_id,
        } => executers::update_config(deps, env, info, new_registrar, max_general_category_id),
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::UpdateStrategies {
            id,
            acct_type,
            strategies,
        } => executers::update_strategies(deps, env, info, id, acct_type, strategies),
        ExecuteMsg::CopycatStrategies {
            id,
            acct_type,
            id_to_copy,
        } => executers::copycat_strategies(deps, info, id, acct_type, id_to_copy),
        ExecuteMsg::CloseEndowment { id, beneficiary } => {
            executers::close_endowment(deps, env, info, id, beneficiary)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        ExecuteMsg::UpdateProfile(msg) => executers::update_profile(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentFees(msg) => {
            executers::update_endowment_fees(deps, env, info, msg)
        }
        ExecuteMsg::SetupDao(msg) => executers::setup_dao(deps, env, info, msg),
        ExecuteMsg::SetupDonationMatch { setup } => {
            executers::setup_donation_match(deps, env, info, setup)
        }
        // Allows the DANO/AP Team to harvest all active vaults
        ExecuteMsg::Harvest { vault_addr } => executers::harvest(deps, env, info, vault_addr),
        ExecuteMsg::HarvestAum {} => executers::harvest_aum(deps, env, info),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    let cw20_fund = Asset {
        info: AssetInfoBase::Cw20(deps.api.addr_validate(info.sender.as_str())?),
        amount: cw20_msg.amount,
    };
    match from_binary(&cw20_msg.msg) {
        Ok(ReceiveMsg::VaultReceipt { id, acct_type }) => executers::vault_receipt(
            deps,
            env,
            info,
            id,
            acct_type,
            api.addr_validate(&cw20_msg.sender)?,
            cw20_fund,
        ),
        Ok(ReceiveMsg::Deposit(msg)) => executers::deposit(
            deps,
            env,
            info,
            api.addr_validate(&cw20_msg.sender)?,
            msg,
            cw20_fund,
        ),
        _ => Err(ContractError::InvalidInputs {}),
    }
}

/// Replies back to the Endowment Account from various multisig contract calls (@ some passed code_id)
/// should be caught and handled to fire subsequent setup calls and ultimately the storing of the multisig
/// as the Accounts endowment owner
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        0 => executers::cw3_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::Balance { id } => to_binary(&queriers::query_endowment_balance(deps, id)?),
        QueryMsg::State { id } => to_binary(&queriers::query_state(deps, id)?),
        QueryMsg::EndowmentList {
            name,
            owner,
            status,
            tier,
            endow_type,
        } => to_binary(&queriers::query_endowment_list(
            deps, name, owner, status, tier, endow_type,
        )?),
        QueryMsg::Endowment { id } => to_binary(&queriers::query_endowment_details(deps, id)?),
        QueryMsg::GetProfile { id } => to_binary(&queriers::query_profile(deps, id)?),
        QueryMsg::TokenAmount {
            id,
            asset_info,
            acct_type,
        } => to_binary(&queriers::query_token_amount(
            deps, id, asset_info, acct_type,
        )?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
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

    const CONFIG_KEY: &[u8] = b"config";
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| ContractError::Std(StdError::not_found("config")))?;
    let old_config: OldConfig = from_slice(&data)?;

    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&Config {
            owner: old_config.owner,
            registrar_contract: old_config.registrar_contract,
            accepted_tokens: old_config.accepted_tokens,
            deposit_approved: old_config.deposit_approved,
            withdraw_approved: old_config.withdraw_approved,
            pending_redemptions: old_config.pending_redemptions,
            last_earnings_harvest: msg.last_earnings_harvest,
            last_harvest_fx: None,
            settings_controller: SettingsController::default(),
        })?,
    );

    Ok(Response::default())
}
