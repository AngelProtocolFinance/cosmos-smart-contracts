use angel_core::structs::{AccountType, SwapOperation};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use terraswap::asset::AssetInfo;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub acct_type: AccountType,
    pub sibling_vault: Addr,
    pub registrar_contract: Addr,
    pub keeper: Addr,
    pub tax_collector: Addr,
    pub swap_router: Addr,

    // TOKENS
    pub native_token: AssetInfo, // the input token(and output back to Accounts) into Vault
    pub lp_token: Addr,          // loopswap pair liquidity token contract address(LP token)
    pub lp_pair_token0: AssetInfo, // loopswap pair token 1 of 2 in LP
    pub lp_pair_token1: AssetInfo, // loopswap pair token 2 of 2 in LP
    pub lp_reward_token: Addr,   // LOOP token address(Atm, LOOP is loopswap farming reward token)

    // ROUTES
    pub reward_to_native_route: Vec<SwapOperation>,
    pub native_to_lp0_route: Vec<SwapOperation>, // reverse the vec to go from LP0 >> native
    pub native_to_lp1_route: Vec<SwapOperation>, // reverse the vec to go from LP1 >> native

    // CORE CONTRACTS
    pub lp_factory_contract: Addr, // loopswap factory contract address
    pub lp_staking_contract: Addr, // loopswap farming contract address
    pub lp_pair_contract: Addr,    // loopswap pair contract address

    pub minimum_initial_deposit: Uint128, // Minimum deposit LP amount limit when `total_shares` = 0
    pub pending_owner: Option<Addr>, // Pending owner address which is used in 2-step `update_owner` process
    pub pending_owner_deadline: Option<u64>, // Block height until which the `pending_owner` is valid in `update_owner` process
}

#[cw_serde]
pub struct State {
    pub total_lp_amount: Uint128, // total amount of LP tokens in this `vault`
    pub total_shares: Uint128,    // total amount of minted vault tokens
}

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[cw_serde]
pub struct MinterData {
    pub minter: Addr,
    /// cap is how many more tokens can be issued by the minter
    pub cap: Option<Uint128>,
}

impl TokenInfo {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const BALANCES: Map<u32, Uint128> = Map::new("balance");
pub const APTAX: Item<Uint128> = Item::new("ap_treasury_tax_balance");
