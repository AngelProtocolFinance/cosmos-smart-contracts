use angel_core::structs::{AcceptedTokens, SplitDetails, StrategyComponent};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, Coin, Decimal, Env, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin_addr: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub index_fund_contract: Addr,
    pub accepted_tokens: AcceptedTokens,
    pub deposit_approved: bool, // DANO has approved to receive donations & transact
    pub withdraw_approved: bool, // DANO has approved to withdraw funds
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub owner: Addr,         // address that originally setup the endowment account
    pub beneficiary: Addr, // address that funds are disbursed to for withdrawals & in a good-standing liquidation(winding up)
    pub name: String,      // name of the Charity Endowment
    pub description: String, // description of the Charity Endowment
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub split_to_liquid: SplitDetails, // set of max, min, and default Split paramenters to check user defined split input against
    pub strategies: Vec<StrategyComponent>,
}

impl Endowment {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(maturity_height) = self.maturity_height {
            if env.block.height > maturity_height {
                return true;
            }
        }
        if let Some(maturity_time) = self.maturity_time {
            if env.block.time > Timestamp::from_seconds(maturity_time) {
                return true;
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub ust_balance: Uint256,
    pub rebalance: RebalanceDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InvestmentHolding {
    pub denom: String,
    pub locked: Uint256,
    pub liquid: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RebalanceDetails {
    pub rebalance_liquid_invested_profits: bool, // should invested portions of the liquid account be rebalanced?
    pub locked_interests_to_liquid: bool, // should Locked acct interest earned be distributed to the Liquid Acct?
    pub interest_distribution: Decimal, // % of Locked acct interest earned to be distributed to the Liquid Acct
    pub locked_principle_to_liquid: bool, // should Locked acct principle be distributed to the Liquid Acct?
    pub principle_distribution: Decimal, // % of Locked acct principle to be distributed to the Liquid Acct
}

impl RebalanceDetails {
    pub fn default() -> Self {
        RebalanceDetails {
            rebalance_liquid_invested_profits: false,
            locked_interests_to_liquid: false,
            interest_distribution: Decimal::percent(20),
            locked_principle_to_liquid: false,
            principle_distribution: Decimal::zero(),
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ENDOWMENT: Item<Endowment> = Item::new("endowment");
pub const ACCOUNTS: Map<String, Account> = Map::new("account");
pub const INVESTMENTS: Map<String, InvestmentHolding> = Map::new("investment");
