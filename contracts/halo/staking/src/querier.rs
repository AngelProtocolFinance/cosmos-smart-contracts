use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use cw20::{Cw20QueryMsg, MinterResponse};

/// Query asset price igonoring price age
pub fn query_halo_minter(querier: &QuerierWrapper, halo_token: Addr) -> StdResult<String> {
    let res: MinterResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: halo_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::Minter {})?,
    }))?;

    Ok(res.minter)
}
