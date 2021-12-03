#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use crate::error::ContractError;
use crate::querier::query_pair_info;
use crate::response::MsgInstantiateContractResponse;
use crate::state::{
    pair_key, read_pair, read_pairs, Config, TmpPairInfo, CONFIG, PAIRS, TMP_PAIR_INFO,
};

use halo_amm::asset::{AssetInfo, PairInfo};
use halo_amm::factory::{
    ConfigResponse, ExecuteMsg, FactoryPairInfo, InstantiateMsg, MigrateMsg, PairsResponse,
    QueryMsg,
};
use halo_amm::pair::ExecuteMsg::UpdateConfig as PairUpdateConfig;
use halo_amm::pair::InstantiateMsg as PairInstantiateMsg;
use protobuf::Message;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        token_code_id: msg.token_code_id,
        pair_code_id: msg.pair_code_id,
        collector_addr: deps.api.addr_validate(&msg.collector_addr)?,
        commission_rate: msg.commission_rate,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            token_code_id,
            pair_code_id,
            pair_contract,
            collector_addr,
            commission_rate,
        } => execute_update_config(
            deps,
            env,
            info,
            owner,
            token_code_id,
            pair_code_id,
            pair_contract,
            collector_addr,
            commission_rate,
        ),
        ExecuteMsg::CreatePair { asset_infos } => execute_create_pair(deps, env, info, asset_infos),
    }
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    token_code_id: Option<u64>,
    pair_code_id: Option<u64>,
    pair_contract: String,
    collector_addr: Option<String>,
    commission_rate: Option<String>,
) -> StdResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    let mut is_pair_update = false;

    // permission check
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(owner) = owner {
        // validate address format
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(token_code_id) = token_code_id {
        config.token_code_id = token_code_id;
    }

    if let Some(pair_code_id) = pair_code_id {
        config.pair_code_id = pair_code_id;
    }

    if let Some(collector_addr) = collector_addr.clone() {
        config.collector_addr = deps.api.addr_validate(&collector_addr)?;
        is_pair_update = true;
    }
    if let Some(commission_rate) = commission_rate.clone() {
        config.commission_rate = commission_rate;
        is_pair_update = true;
    }

    CONFIG.save(deps.storage, &config)?;

    if is_pair_update {
        // Update pair contract config
        let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_contract,
            msg: to_binary(&PairUpdateConfig {
                collector_addr,
                commission_rate,
            })
            .unwrap(),
            funds: vec![],
        });
        Ok(Response::new()
            .add_attribute("action", "update_config")
            .add_submessage(SubMsg {
                id: 0,
                gas_limit: None,
                msg: wasm_msg,
                reply_on: ReplyOn::Never,
            }))
    } else {
        Ok(Response::new().add_attribute("action", "update_config"))
    }
}

// Anyone can execute it to create swap pair
pub fn execute_create_pair(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset_infos: [AssetInfo; 2],
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    if read_pair(deps.as_ref(), &asset_infos).is_ok() {
        return Err(StdError::generic_err("Pair already exists"));
    }
    let pair_key = pair_key(&asset_infos);

    TMP_PAIR_INFO.save(
        deps.storage,
        &TmpPairInfo {
            pair_key,
            owner: info.sender,
        },
    )?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_pair"),
            ("pair", &format!("{}-{}", asset_infos[0], asset_infos[1])),
        ])
        .add_submessage(SubMsg {
            id: 0,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: config.pair_code_id,
                funds: vec![],
                admin: Some(config.owner.to_string()),
                label: "HALO pair".to_string(),
                msg: to_binary(&PairInstantiateMsg {
                    asset_infos,
                    token_code_id: config.token_code_id,
                    collector_addr: config.collector_addr.to_string(),
                    commission_rate: config.commission_rate,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let tmp = TMP_PAIR_INFO.load(deps.storage)?;
    if PAIRS.may_load(deps.storage, &tmp.pair_key)?.is_some() {
        return Err(ContractError::PairWasRegistered {});
    }

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(msg.result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let pair_contract = deps.api.addr_validate(res.get_contract_address())?;

    PAIRS.save(
        deps.storage,
        &tmp.pair_key,
        &FactoryPairInfo {
            contract_addr: pair_contract.clone(),
        },
    )?;

    Ok(Response::new().add_attributes(vec![("pair_contract_addr", pair_contract)]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pair { asset_infos } => to_binary(&query_pair(deps, asset_infos)?),
        QueryMsg::Pairs { start_after, limit } => {
            to_binary(&query_pairs(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: state.owner.to_string(),
        token_code_id: state.token_code_id,
        pair_code_id: state.pair_code_id,
        collector_addr: state.collector_addr.to_string(),
        commission_rate: state.commission_rate,
    };

    Ok(resp)
}

pub fn query_pair(deps: Deps, asset_infos: [AssetInfo; 2]) -> StdResult<PairInfo> {
    let pair_addr = PAIRS
        .load(deps.storage, &pair_key(&asset_infos))?
        .contract_addr;
    let pair_info = query_pair_info(deps, &pair_addr)?;
    Ok(pair_info)
}

pub fn query_pairs(
    deps: Deps,
    start_after: Option<[AssetInfo; 2]>,
    limit: Option<u32>,
) -> StdResult<PairsResponse> {
    let start_after =
        start_after.map(|start_after| [start_after[0].clone(), start_after[1].clone()]);

    let pairs: Vec<PairInfo> = read_pairs(deps, start_after, limit)
        .iter()
        .map(|pair| query_pair_info(deps, &pair.contract_addr).unwrap())
        .collect();

    Ok(PairsResponse { pairs })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
