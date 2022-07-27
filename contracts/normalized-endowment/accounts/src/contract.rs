use crate::executers;
use crate::queriers;
use crate::state::{Config, Endowment, OldConfig, State, CONFIG, ENDOWMENT, PROFILE, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::messages::subdao::InstantiateMsg as DaoInstantiateMsg;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::EndowmentType;
use angel_core::structs::{
    AcceptedTokens, BalanceInfo, RebalanceDetails, SettingsController, StrategyComponent,
};
use cosmwasm_std::{
    attr, entry_point, from_binary, from_slice, to_binary, to_vec, Binary, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdError, StdResult,
    SubMsg, Uint128, WasmMsg, WasmQuery,
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

    // apply the initial configs passed
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            accepted_tokens: AcceptedTokens::default(),
            deposit_approved: false,  // bool
            withdraw_approved: false, // bool
            pending_redemptions: None,
            last_earnings_harvest: env.block.height,
            last_harvest_fx: None,
            settings_controller: match msg.settings_controller {
                Some(controller) => controller,
                None => SettingsController::default(),
            },
        },
    )?;

    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.registrar_contract.clone(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    let default_vault = match registrar_config.default_vault {
        Some(ref addr) => addr.to_string(),
        None => return Err(ContractError::ContractNotConfigured {}),
    };
    ENDOWMENT.save(
        deps.storage,
        &Endowment {
            owner: deps.api.addr_validate(&msg.owner)?, // Addr
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,           // Option<u64>
            strategies: vec![StrategyComponent {
                vault: deps.api.addr_validate(&default_vault)?.to_string(),
                percentage: Decimal::one(),
            }],
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
            transactions: vec![],
        },
    )?;

    PROFILE.save(deps.storage, &msg.profile)?;

    // initial default Response to add submessages to
    let mut res: Response = Response::new().add_attributes(vec![
        attr("endow_addr", env.contract.address.to_string()),
        attr("endow_name", msg.profile.name),
        attr("endow_owner", msg.owner.to_string()),
        attr("endow_type", msg.profile.endow_type.to_string()),
        attr(
            "endow_logo",
            msg.profile.logo.unwrap_or_else(|| "".to_string()),
        ),
        attr(
            "endow_image",
            msg.profile.image.unwrap_or_else(|| "".to_string()),
        ),
    ]);

    // check if CW3/CW4 codes were passed to setup a multisig/group
    let cw4_members = if msg.cw4_members.is_empty() {
        vec![Member {
            addr: msg.owner.to_string(),
            weight: 1,
        }]
    } else {
        msg.cw4_members
    };

    if registrar_config.cw3_code.eq(&None) || registrar_config.cw4_code.eq(&None) {
        return Err(ContractError::Std(StdError::generic_err(
            "cw3_code & cw4_code must exist",
        )));
    }
    res = res.add_submessage(SubMsg {
        id: 1,
        msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: registrar_config.cw4_code.unwrap(),
            admin: None,
            label: "new endowment cw4 group".to_string(),
            msg: to_binary(&angel_core::messages::cw4_group::InstantiateMsg {
                admin: None,
                members: cw4_members,
                cw3_code: registrar_config.cw3_code.unwrap(),
                cw3_threshold: msg.cw3_multisig_threshold,
                cw3_max_voting_period: msg.cw3_multisig_max_vote_period,
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    // check if a dao needs to be setup along with a dao token contract
    match (
        msg.dao,
        registrar_config.subdao_token_code,
        registrar_config.subdao_gov_code,
    ) {
        (Some(dao_setup), Some(_token_code), Some(gov_code)) => {
            res = res.add_submessage(SubMsg {
                id: 3,
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
                        donation_match: msg.donation_match,
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
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
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
        ExecuteMsg::Withdraw {
            sources,
            beneficiary,
            asset_info,
        } => executers::withdraw(deps, env, info, sources, beneficiary, asset_info),
        ExecuteMsg::WithdrawLiquid {
            liquid_amount,
            beneficiary,
            asset_info,
        } => executers::withdraw_liquid(deps, env, info, liquid_amount, beneficiary, asset_info),
        ExecuteMsg::VaultReceipt {} => {
            if info.funds.len() != 1 {
                return Err(ContractError::InvalidCoinsDeposited {});
            }
            let native_fund = Asset {
                info: AssetInfoBase::Native(info.funds[0].denom.to_string()),
                amount: info.funds[0].amount,
            };
            executers::vault_receipt(deps, env, info.clone(), info.sender, native_fund)
        }
        ExecuteMsg::UpdateRegistrar { new_registrar } => {
            executers::update_registrar(deps, env, info, new_registrar)
        }
        ExecuteMsg::UpdateOwner { new_owner } => {
            executers::update_owner(deps, env, info, new_owner)
        }
        ExecuteMsg::UpdateStrategies { strategies } => {
            executers::update_strategies(deps, env, info, strategies)
        }
        ExecuteMsg::CloseEndowment { beneficiary } => {
            executers::close_endowment(deps, env, info, beneficiary)
        }
        ExecuteMsg::UpdateConfig(msg) => executers::update_config(deps, env, info, msg),
        ExecuteMsg::UpdateProfile(msg) => executers::update_profile(deps, env, info, msg),
        ExecuteMsg::UpdateEndowmentFees(msg) => {
            executers::update_endowment_fees(deps, env, info, msg)
        }
        // Allows the DANO/AP Team to harvest all active vaults
        ExecuteMsg::Harvest { vault_addr } => executers::harvest(deps, env, info, vault_addr),
        ExecuteMsg::HarvestAum {} => executers::harvest_aum(deps, env, info),
        ExecuteMsg::SetupDao(msg) => executers::setup_dao(deps, env, info, msg),
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
        Ok(ReceiveMsg::VaultReceipt {}) => executers::vault_receipt(
            deps,
            env,
            info,
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
        1 => executers::cw4_group_reply(deps, env, msg.result),
        3 => executers::dao_reply(deps, env, msg.result),
        4 => executers::harvest_reply(deps, env, msg.result),
        _ => Err(ContractError::Std(StdError::GenericErr {
            msg: "Invalid Submessage Reply ID!".to_string(),
        })),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance {} => to_binary(&queriers::query_account_balance(deps, env)?),
        QueryMsg::Config {} => to_binary(&queriers::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queriers::query_state(deps)?),
        QueryMsg::Endowment {} => to_binary(&queriers::query_endowment_details(deps)?),
        QueryMsg::GetProfile {} => to_binary(&queriers::query_profile(deps)?),
        QueryMsg::GetTxRecords {
            sender,
            recipient,
            asset_info,
        } => to_binary(&queriers::query_transactions(
            deps, sender, recipient, asset_info,
        )?),
        QueryMsg::GetEndowmentFees {} => to_binary(&queriers::query_endowment_fees(deps)?),
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
