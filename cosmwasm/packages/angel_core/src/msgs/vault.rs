use crate::structs::{AccountType, SwapOperation};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
#[allow(unused_imports)]
use cw20::{BalanceResponse, Cw20ReceiveMsg, TokenInfoResponse};
use cw_asset::AssetInfo as CwAssetInfo;
use terraswap::asset::Asset;

#[cw_serde]
pub struct InstantiateMsg {
    pub acct_type: AccountType,
    pub sibling_vault: Option<String>,
    pub registrar_contract: String,
    pub keeper: String,
    pub tax_collector: String,
    pub swap_router: String,

    pub lp_factory_contract: String,
    pub lp_staking_contract: String,
    pub pair_contract: String,
    pub lp_reward_token: String,
    pub native_token: CwAssetInfo,

    pub reward_to_native_route: Vec<SwapOperation>,
    pub native_to_lp0_route: Vec<SwapOperation>,
    pub native_to_lp1_route: Vec<SwapOperation>,

    pub minimum_initial_deposit: Uint128,

    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

/// We currently take no arguments for migrations
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateOwner {
        new_owner: String,
    },
    UpdateRegistrar {
        new_registrar: Addr,
    },
    UpdateConfig(UpdateConfigMsg),
    Deposit {
        endowment_id: u32,
    },
    Redeem {
        endowment_id: u32,
        amount: Uint128, // vault tokens to be burned
    },
    /// reinvest vault assets from self (if AccountType::Liquid)
    /// over to it's AccountType::Locked (sibling) vault
    ReinvestToLocked {
        endowment_id: u32,
        amount: Uint128,
    },
    Harvest {},
    RestakeClaimReward {
        reward_token_bal_before: Uint128,
    },
    AddLiquidity {
        endowment_id: Option<u32>,
        lp_pair_token0_bal_before: Uint128,
        lp_pair_token1_bal_before: Uint128,
    },
    RemoveLiquidity {
        lp_token_bal_before: Uint128,
        beneficiary: Addr,
        id: Option<u32>,
    },
    Stake {
        endowment_id: Option<u32>,
        lp_token_bal_before: Uint128,
    },
    SwapBack {
        lp_pair_token0_bal_before: Uint128,
        lp_pair_token1_bal_before: Uint128,
    },
    SendAsset {
        beneficiary: Addr,
        id: Option<u32>,
        native_token_bal_before: Uint128,
    },
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub sibling_vault: Option<String>,
    pub keeper: Option<String>,
    pub tax_collector: Option<String>,

    pub native_token: Option<CwAssetInfo>,
    pub reward_to_native_route: Option<Vec<SwapOperation>>,
    pub native_to_lp0_route: Option<Vec<SwapOperation>>,
    pub native_to_lp1_route: Option<Vec<SwapOperation>>,

    pub minimum_initial_deposit: Option<Uint128>,
}

#[cw_serde]
pub struct AccountWithdrawMsg {
    pub endowment_id: u32,
    pub beneficiary: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct RoutesUpdateMsg {
    pub add: Vec<Addr>,
    pub remove: Vec<Addr>,
}

#[cw_serde]
pub enum ReceiveMsg {
    Deposit {
        endowment_id: u32,
    },
    ReinvestToLocked {
        endowment_id: u32,
        amount: Uint128,
    },
    /// send the harvest portion from self (if AccountType::Locked)
    /// over to it's AccountType::Liquid (sibling) vault
    HarvestToLiquid {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the configuration of the contract
    #[returns(ConfigResponse)]
    Config {},
    /// Returns the state of the contract
    #[returns(StateResponse)]
    State {},
    /// Returns the current balance of the given "Endowment ID", 0 if unset.
    #[returns(BalanceResponse)]
    Balance { endowment_id: u32 },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    /// Returns the total balance/total_share of the contract
    #[returns(BalanceResponse)]
    TotalBalance {},
    /// Returns the APTAX balance of the contract
    #[returns(BalanceResponse)]
    ApTaxBalance {},
}

#[cw_serde]
pub enum LoopFarmingExecuteMsg {
    Stake {},           // Farm action. Stake LP token.(param: amount in `send` msg)
    UnstakeAndClaim {}, // Unfarm action. Unstake farmed LP token & rewards.(param: amount in `send` msg)
    ClaimReward {},     // Claim the reward. Enabled just after `stake`
}

#[cw_serde]
pub enum LoopPairExecuteMsg {
    Swap {
        offer_asset: Asset,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    },
    ProvideLiquidity {
        assets: [Asset; 2],
    },
    WithdrawLiquidity {},
}

#[cw_serde]
pub enum LoopFarmingQueryMsg {
    QueryFlpTokenFromPoolAddress { pool_address: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub acct_type: AccountType,
    pub sibling_vault: String,
    pub registrar_contract: String,
    pub keeper: String,
    pub tax_collector: String,

    pub native_token: String,
    pub lp_token: String,
    pub lp_pair_token0: String,
    pub lp_pair_token1: String,
    pub lp_reward_token: String,

    pub reward_to_native_rotue: Vec<SwapOperation>,
    pub native_to_lp0_route: Vec<SwapOperation>,
    pub native_to_lp1_route: Vec<SwapOperation>,

    pub lp_factory_contract: String,
    pub lp_staking_contract: String,
    pub lp_pair_contract: String,

    pub minimum_initial_deposit: String,
    pub pending_owner: String,
    pub pending_owner_deadline: u64,
}

#[cw_serde]
pub struct StateResponse {
    pub total_lp_amount: String,
    pub total_shares: String,
}
