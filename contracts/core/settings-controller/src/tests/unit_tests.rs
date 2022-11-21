use super::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::contract::{execute, instantiate, query};
use angel_core::errors::core::*;

use angel_core::messages::accounts::UpdateConfigMsg;
use angel_core::messages::accounts::{
    CreateEndowmentMsg, DepositMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Strategy,
    UpdateEndowmentSettingsMsg, UpdateEndowmentStatusMsg, UpdateMaturityWhitelist,
};
use angel_core::responses::accounts::{ConfigResponse, EndowmentDetailsResponse, StateResponse};
use angel_core::structs::{
    AccountType, Beneficiary, Categories, EndowmentType, SplitDetails, StrategyComponent,
    SwapOperation,
};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, coins, from_binary, to_binary, Addr, Coin, Decimal, Env, OwnedDeps, StdError, Uint128,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetUnchecked};
use cw_utils::{Expiration, Threshold};

const AP_TEAM: &str = "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly";
const CHARITY_ID: u32 = 1;
const CHARITY_ADDR: &str = "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v";
const REGISTRAR_CONTRACT: &str = "terra18wtp5c32zfde3vsjwvne8ylce5thgku99a2hyt";
const PLEB: &str = "terra17nqw240gyed27q8y4aj2ukg68evy3ml8n00dnh";
const DEPOSITOR: &str = "depositor";
