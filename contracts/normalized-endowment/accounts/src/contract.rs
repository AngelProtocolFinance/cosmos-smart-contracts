use crate::executers;
use crate::executers::setup_dao_token_messages;
use crate::queriers;
use crate::state::{
    Config, Cw3MultiSigConfig, Endowment, OldConfig, State, CONFIG, CW3MULTISIGCONFIG, ENDOWMENT,
    PROFILE, STATE,
};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw4_group::InstantiateMsg as Cw4GroupInstantiateMsg;
use angel_core::messages::donation_match::InstantiateMsg as DonationMatchInstantiateMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
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
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
            owner: deps.api.addr_validate(&msg.owner_sc)?,
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
            name: msg.name.clone(),
            description: msg.description.clone(),
            withdraw_before_maturity: msg.withdraw_before_maturity, // bool
            maturity_time: msg.maturity_time,                       // Option<u64>
            strategies: vec![StrategyComponent {
                vault: deps.api.addr_validate(&default_vault)?.to_string(),
                percentage: Decimal::one(),
            }],
            rebalance: RebalanceDetails::default(),
            dao: None,
            dao_token: None,
            donation_match: msg.donation_match,
            whitelisted_beneficiaries: msg.whitelisted_beneficiaries, // Vec<String>
            whitelisted_contributors: msg.whitelisted_contributors,   // Vec<String>
            donation_matching_contract: None,
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

    CW3MULTISIGCONFIG.save(
        deps.storage,
        &Cw3MultiSigConfig {
            threshold: msg.cw3_multisig_threshold,
            max_voting_period: msg.cw3_multisig_max_vote_period,
        },
    )?;

    // initial default Response to add submessages to
    let mut res: Response = Response::new().add_attributes(vec![
        attr("endow_name", msg.name),
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
            msg: to_binary(&Cw4GroupInstantiateMsg {
                admin: Some(info.sender.to_string()),
                members: cw4_members,
            })?,
            funds: vec![],
        }),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    // check if a dao needs to be setup along with subdao token contract
    if msg.dao {
        let endowment_owner = deps.api.addr_validate(&msg.owner)?;
        let submsgs = setup_dao_token_messages(
            deps.branch(),
            msg.dao_setup_option,
            &registrar_config,
            endowment_owner,
        )?;
        res = res.add_submessages(submsgs);
    }

    // check if donation_matching_contract needs to be instantiated
    // `donation_match_setup_option`: Field to determine the various way of setting up "donation_match" contract
    // Possible values:
    //   0 => Endowment doesn't want a DAO. No dao, dao token, or donation matching contract are need (obviously!! )
    //   1 => Endowment wants to have a DAO and they choose to use $HALO as reserve token for their bonding curve from the Endowment Launchpad UI.
    //   2 => Endowment wants to have a DAO but they want to use an existing Token (other than HALO) as the reserve token for their bonding curve.
    //        They would then have to supply several contract addresses during their endowment creation:
    //           - the Token contract address (CW20)
    //           - a Token / UST LP Pair contract ( this attribute would be updatable should they move supply to a new pool, etc)
    //   3 =>  Endowment wants to have a DAO but they want to use an brand new CW20 Token that will not be attached to a bonding curve. (coming later)
    if msg.profile.endow_type == EndowmentType::Normal && msg.donation_match_setup_option != 0 {
        // setup the donation_matching contract
        let donation_match_code = match registrar_config.donation_match_code {
            Some(id) => id,
            None => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "No code id for donation matching contract".to_string(),
                }))
            }
        };

        let reserve_token = match msg.donation_match_setup_option {
            1 => match registrar_config.halo_token {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "No reserve token(halo_token) is available".to_string(),
                    }))
                }
            },
            2 => match msg.user_reserve_token {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "No reserve token address is given".to_string(),
                    }))
                }
            },
            _ => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "Invalid donation_match setup option".to_string(),
                }));
            }
        };

        let reserve_token_ust_lp_pair = match msg.donation_match_setup_option {
            1 => match msg.halo_ust_lp_pair_contract {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "No halo-ust lp pair contract is given".to_string(),
                    }))
                }
            },
            2 => match msg.user_reserve_ust_lp_pair_contract {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::Std(StdError::GenericErr {
                        msg: "No reserve token - ust lp pair contract is given".to_string(),
                    }))
                }
            },
            _ => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "Invalid donation_match setup option".to_string(),
                }));
            }
        };

        res = res.add_submessage(SubMsg {
            id: 4,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: donation_match_code,
                admin: None,
                label: "new donation match contract".to_string(),
                msg: to_binary(&DonationMatchInstantiateMsg {
                    reserve_token,
                    lp_pair: reserve_token_ust_lp_pair,
                    registrar_contract: msg.registrar_contract.clone(),
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        })
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

        ExecuteMsg::SetupDaoToken { option } => executers::setup_dao_token(deps, env, info, option),
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
            info.clone(),
            api.addr_validate(&cw20_msg.sender)?,
            cw20_fund,
        ),
        Ok(ReceiveMsg::Deposit(msg)) => executers::deposit(
            deps,
            env,
            info.clone(),
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
        1 => executers::new_cw4_group_reply(deps, env, msg.result),
        2 => executers::new_cw3_multisig_reply(deps, env, msg.result),
        3 => executers::new_dao_token_reply(deps, env, msg.result),
        4 => executers::new_donation_match_reply(deps, env, msg.result),
        5 => executers::harvest_reply(deps, env, msg.result),
        6 => executers::new_dao_cw20_token_reply(deps, env, msg.result),
        _ => Err(ContractError::Unauthorized {}),
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
