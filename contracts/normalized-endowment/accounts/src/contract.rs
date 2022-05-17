use crate::executers;
use crate::queriers;
use crate::state::OldConfig;
use crate::state::PROFILE;
use crate::state::{Config, Endowment, State, CONFIG, ENDOWMENT, STATE};
use angel_core::errors::core::ContractError;
use angel_core::messages::accounts::*;
use angel_core::messages::cw4_group::InstantiateMsg as Cw4GroupInstantiateMsg;
use angel_core::messages::dao_token::InstantiateMsg as DaoTokenInstantiateMsg;
use angel_core::messages::donation_match::InstantiateMsg as DonationMatchInstantiateMsg;
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::ConfigResponse;
use angel_core::structs::EndowmentType;
use angel_core::structs::{AcceptedTokens, BalanceInfo, RebalanceDetails, StrategyComponent};
use cosmwasm_std::{
    attr, entry_point, from_slice, to_binary, to_vec, Binary, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, QueryRequest, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg, WasmQuery,
};

use cw2::set_contract_version;

// version info for future migration info
const CONTRACT_NAME: &str = "accounts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        },
    )?;

    let registrar_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.registrar_contract.clone(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    let default_vault = match registrar_config.default_vault {
        Some(addr) => addr,
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
            maturity_height: msg.maturity_height,                   // Option<u64>
            strategies: vec![StrategyComponent {
                vault: deps.api.addr_validate(&default_vault)?,
                locked_percentage: Decimal::one(),
                liquid_percentage: Decimal::one(),
            }],
            rebalance: RebalanceDetails::default(),
            dao: None,
            dao_token: None,
            donation_match: msg.donation_match,
            whitelisted_beneficiaries: msg.whitelisted_beneficiaries, // Vec<String>
            whitelisted_contributors: msg.whitelisted_contributors,   // Vec<String>
            locked_endowment_configs: msg.locked_endowment_configs,   // vec<String>
            donation_matching_contract: None,
            earnings_fee: msg.earnings_fee,
            withdraw_fee: msg.withdraw_fee,
            deposit_fee: msg.deposit_fee,
            aum_fee: msg.aum_fee,
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
        attr("endow_name", msg.name),
        attr("endow_owner", msg.owner),
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
    if msg.cw4_members.ne(&vec![])
        && (registrar_config.cw3_code.ne(&None) && registrar_config.cw4_code.ne(&None))
    {
        res = res.add_submessage(SubMsg {
            id: 1,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: registrar_config.cw4_code.unwrap(),
                admin: None,
                label: "new endowment cw4 group".to_string(),
                msg: to_binary(&Cw4GroupInstantiateMsg {
                    admin: Some(info.sender.to_string()),
                    members: msg.cw4_members,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        })
    }

    // check if a dao needs to be setup along with subdao token contract
    if msg.dao && msg.curve_type.ne(&None) {
        // setup DAO token contract
        let halo_token = match registrar_config.halo_token.clone() {
            Some(addr) => addr,
            None => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: "HALO token address is empty".to_string(),
                }))
            }
        };
        res = res.add_submessage(SubMsg {
            id: 3,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: registrar_config.subdao_token_code.unwrap(),
                admin: None,
                label: "new endowment dao token contract".to_string(),
                msg: to_binary(&DaoTokenInstantiateMsg {
                    name: "AP Endowment Dao Token".to_string(), // need dynamic name
                    symbol: "APEDT".to_string(),                // need dynamic symbol
                    decimals: 6,
                    reserve_denom: halo_token.to_string(),
                    reserve_decimals: 6,
                    curve_type: msg.curve_type.unwrap(),
                    halo_token,
                    unbonding_period: 7,
                })?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        })
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
        ExecuteMsg::UpdateEndowmentSettings(msg) => {
            executers::update_endowment_settings(deps, env, info, msg)
        }
        ExecuteMsg::UpdateEndowmentStatus(msg) => {
            executers::update_endowment_status(deps, env, info, msg)
        }
        ExecuteMsg::Deposit(msg) => executers::deposit(deps, env, info.clone(), info.sender, msg),
        ExecuteMsg::Withdraw {
            sources,
            beneficiary,
        } => executers::withdraw(deps, env, info, sources, beneficiary),
        ExecuteMsg::VaultReceipt(msg) => {
            executers::vault_receipt(deps, env, info.clone(), info.sender, msg)
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
            denom,
        } => to_binary(&queriers::query_transactions(
            deps, sender, recipient, denom,
        )?),
        QueryMsg::GetEndowmentFees {} => to_binary(&queriers::query_endowment_fees(deps)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
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
        })?,
    );

    Ok(Response::default())
}
