use crate::state::{Config, CONFIG};
use angel_core::errors::core::ContractError;
use angel_core::messages::donation_match::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};

use angel_core::messages::accounts::QueryMsg as AccountQueryMsg;
use angel_core::messages::dao_token::Cw20HookMsg as DaoTokenHookMsg;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::responses::accounts::EndowmentDetailsResponse;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfig, EndowmentDetailResponse,
};
use angel_core::structs::{EndowmentStatus, EndowmentType};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::QueryMsg as PairQueryMsg;
use terraswap::pair::SimulationResponse;

// version info for migration info
const CONTRACT_NAME: &str = "donation-match";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Validations
    let reserve_token = deps.api.addr_validate(msg.reserve_token.as_str())?;
    let lp_pair_contract = deps.api.addr_validate(msg.lp_pair.as_str())?;
    let registrar_contract = deps.api.addr_validate(msg.registrar_contract.as_str())?;

    // Save the "Config"
    CONFIG.save(
        deps.storage,
        &Config {
            reserve_token,
            lp_pair_contract,
            registrar_contract,
        },
    )?;

    Ok(Response::new().add_attributes(vec![attr("instantiate", "donation-match")]))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // TODO
        ExecuteMsg::DonorMatch {
            amount,
            donor,
            token,
        } => execute_donor_match(deps, env, info, amount, donor, token),
    }
}

fn execute_donor_match(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    donor: Addr,
    token: Addr,
) -> Result<Response, ContractError> {
    let uusd_amount = amount;
    let dao_token = token;
    let donor = donor.to_string();
    let endowment_contract = info.sender.to_string();

    let config = CONFIG.load(deps.storage)?;

    // Validation 1. Check if the tx sender is valid endowment contract
    let endow_detail: EndowmentDetailResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.clone(),
        &RegistrarQueryMsg::Endowment {
            endowment_addr: info.sender.to_string(),
        },
    )?;

    if endow_detail.endowment.status != EndowmentStatus::Approved {
        return Err(ContractError::Unauthorized {});
    }

    // Validation 2. Check if the correct endowment is calling this entry
    match endow_detail.endowment.endow_type {
        EndowmentType::Charity => {
            let registrar_config: RegistrarConfig = deps.querier.query_wasm_smart(
                config.registrar_contract.clone(),
                &RegistrarQueryMsg::Config {},
            )?;
            if env.contract.address.to_string()
                != registrar_config.donation_match_charites_contract.unwrap()
            {
                return Err(ContractError::Unauthorized {});
            }
        }
        EndowmentType::Normal => {
            let endow: EndowmentDetailsResponse = deps
                .querier
                .query_wasm_smart(config.registrar_contract, &AccountQueryMsg::Endowment {})?;
            if env.contract.address.to_string() != endow.donation_match_contract_addr {
                return Err(ContractError::Unauthorized {});
            }
        }
    };

    // Validation 2. Check if the correct amount of UST is sent.
    let received_uusd = match info.funds.into_iter().find(|coin| coin.denom == "uusd") {
        Some(c) => c.amount,
        None => return Err(ContractError::EmptyBalance {}),
    };
    if uusd_amount != received_uusd {
        return Err(ContractError::InsufficientFunds {});
    }

    // Query the "lp_pair" contract for "reserve_token" amount
    let swap_sim_resp: SimulationResponse = deps.querier.query_wasm_smart(
        config.lp_pair_contract,
        &PairQueryMsg::Simulation {
            offer_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: uusd_amount,
            },
        },
    )?;

    let reserve_token_amount = swap_sim_resp.return_amount;

    // Check if this contract has more than "reserve_token_amount" reserve_tokens
    let holding_reserve_token_bal: BalanceResponse = deps.querier.query_wasm_smart(
        config.reserve_token.clone(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    if holding_reserve_token_bal.balance < reserve_token_amount {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Insufficient reserve token amount".to_string(),
        }));
    }

    // send the "reserve_token" to CS/dao-token contract
    let msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.reserve_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: dao_token.to_string(),
            amount: reserve_token_amount,
            msg: to_binary(&DaoTokenHookMsg::DonorMatch {
                amount: reserve_token_amount,
                donor,
                endowment_contract,
            })?,
        })?,
        funds: vec![],
    })];

    Ok(Response::default().add_messages(msgs).add_attributes(vec![
        attr("method", "donor_match"),
        attr("reserve_token", config.reserve_token.to_string()),
        attr("dao_token", dao_token.to_string()),
        attr("reserve_token_amt", reserve_token_amount.to_string()),
    ]))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        reserve_token: config.reserve_token.to_string(),
        lp_pair: config.lp_pair_contract.to_string(),
        registrar_contract: config.registrar_contract.to_string(),
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
