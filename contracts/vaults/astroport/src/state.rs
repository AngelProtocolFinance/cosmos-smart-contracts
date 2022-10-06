use angel_core::structs::AccountType;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::astro_core_structs::{asset::AssetInfo, router::SwapOperation};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub acct_type: AccountType,
    pub sibling_vault: Addr,
    pub registrar_contract: Addr,
    pub keeper: Addr,
    pub tax_collector: Addr,
    pub swap_router: Addr,

    pub ibc_relayer: Addr,
    pub ibc_sender: Addr,
    pub ap_tax_rate: Decimal, // Same as `registrar::config.tax_rate`
    pub interest_distribution: Decimal, // Same as `registrar::config.rebalance.interest_distribution`

    // TOKENS
    pub native_token: AssetInfo, // the input token(and output back to Accounts) into Vault
    pub lp_token: Addr,          // astroport pair liquidity token contract address(LP token)
    pub lp_pair_token0: AssetInfo, // astroport pair token 1 of 2 in LP
    pub lp_pair_token1: AssetInfo, // astroport pair token 2 of 2 in LP
    pub lp_reward_token: Addr, // ASTRO token address(Atm, ASTRO is astroport generator reward token)

    // ROUTES
    pub reward_to_native_route: Vec<SwapOperation>,
    pub native_to_lp0_route: Vec<SwapOperation>, // reverse the vec to go from LP0 >> native
    pub native_to_lp1_route: Vec<SwapOperation>, // reverse the vec to go from LP1 >> native

    // CORE CONTRACTS
    pub lp_factory_contract: Addr, // astroport factory contract address
    pub lp_staking_contract: Addr, // astroport generator contract address
    pub lp_pair_contract: Addr,    // astroport pair contract address
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_lp_amount: Uint128, // total amount of LP tokens in this `vault`
    pub total_shares: Uint128,    // total amount of minted vault tokens
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
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
