use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_contract: String,
    pub keeper: String,

    pub loop_factory_contract: String,
    pub loop_farming_contract: String,
    pub loop_pair_contract: String,

    pub name: String,
    pub symbol: String,
    pub decimals: u8,

    pub harvest_to_liquid: Decimal,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
    Claim {},
    DistributeClaim {
        lp_token_bal_before: Uint128,
    },
    Withdraw(AccountWithdrawMsg),
    Harvest {},
    HarvestSwap {
        token1_denom_bal_before: Uint128,
        token2_denom_bal_before: Uint128,
    },
    DistributeHarvest {
        output_token_bal_before: Uint128,
    },
    AddLiquidity {
        endowment_id: u32,
        in_asset_info: terraswap::asset::AssetInfo,
        out_asset_info: terraswap::asset::AssetInfo,
        in_asset_bal_before: Uint128,
        out_asset_bal_before: Uint128,
    },
    RemoveLiquidity {
        lp_token_bal_before: Uint128,
        action: RemoveLiquidAction,
    },
    Stake {
        endowment_id: u32,
        lp_token_bal_before: Uint128,
    },
    SwapAndSendTo {
        token1_denom_bal_before: Uint128,
        token2_denom_bal_before: Uint128,
        beneficiary: Addr,
    },
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub loop_factory_contract: Option<String>,
    pub loop_farming_contract: Option<String>,
    pub loop_pair_contract: Option<String>,
    pub keeper: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountWithdrawMsg {
    pub endowment_id: u32,
    pub beneficiary: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoutesUpdateMsg {
    pub add: Vec<Addr>,
    pub remove: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Deposit { endowment_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the configuration of the contract
    /// Return type: ConfigResponse.
    Config {},
    /// Returns the current balance of the given "Endowment ID", 0 if unset.
    /// Return type: BalanceResponse.
    Balance { id: u32 },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
    /// Returns the total balance/total_share of the contract
    /// Return type: BalanceResponse.
    TotalBalance {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum RemoveLiquidAction {
    Harvest,
    Claim {},
    Withdraw { beneficiary: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoopFarmingExecuteMsg {
    Stake {},           // Farm action. Stake LP token.(param: amount in `send` msg)
    UnstakeAndClaim {}, // Unfarm action. Unstake farmed LP token & rewards.(param: amount in `send` msg)
    ClaimReward {},     // Claim the reward. Enabled just after `stake`
}
