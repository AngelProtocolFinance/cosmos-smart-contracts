use astroport_lbp::asset::{AssetInfo, PairInfo};
use astroport_lbp::factory::QueryMsg;
use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, WasmQuery};

pub fn query_pair_info(deps: Deps, factory_contract: &Addr, asset_infos: [AssetInfo; 2]) -> StdResult<PairInfo> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: factory_contract.to_string(),
        msg: to_binary(&QueryMsg::Pair { asset_infos })?,
    }))
}
