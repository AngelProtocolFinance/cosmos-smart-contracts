use cosmwasm_std::{Addr, Coin, Decimal, Env, SubMsg, Timestamp, Uint128};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceResponse {
    pub locked_native: Vec<Coin>,
    pub locked_cw20: Vec<Cw20Coin>,
    pub liquid_native: Vec<Coin>,
    pub liquid_cw20: Vec<Cw20Coin>,
}

impl BalanceResponse {
    pub fn default() -> Self {
        BalanceResponse {
            locked_native: vec![],
            locked_cw20: vec![],
            liquid_native: vec![],
            liquid_cw20: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct YieldVault {
    pub address: Addr,
    pub input_denom: String,
    pub yield_token: Addr,
    pub approved: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StrategyComponent {
    pub vault: Addr,                // Vault SC Address
    pub locked_percentage: Decimal, // percentage of funds to invest
    pub liquid_percentage: Decimal, // percentage of funds to invest
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundingSource {
    pub vault: String,
    pub locked: Uint128,
    pub liquid: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct RedeemResults {
    pub messages: Vec<SubMsg>,
    pub total: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SplitDetails {
    pub max: Decimal,
    pub min: Decimal,
    pub default: Decimal, // for when a split parameter is not provided
}

impl SplitDetails {
    pub fn default() -> Self {
        SplitDetails {
            min: Decimal::zero(),
            max: Decimal::one(),
            default: Decimal::percent(50),
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EndowmentEntry {
    pub address: Addr,
    pub status: EndowmentStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum EndowmentStatus {
    Inactive = 0, // Default state when new Endowment is created
    // Statuses below are set by DANO or AP Team
    Approved = 1, // Allowed to receive donations and process withdrawals
    Frozen = 2,   // Temp. hold is placed on withdraw from an Endowment
    Closed = 3,   // Status for final Liquidations(good-standing) or Terminations(poor-standing)
}

impl fmt::Display for EndowmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EndowmentStatus::Inactive => "0",
                EndowmentStatus::Approved => "1",
                EndowmentStatus::Frozen => "2",
                EndowmentStatus::Closed => "3",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IndexFund {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub members: Vec<Addr>,
    // Fund Specific: over-riding SC level setting to handle a fixed split value
    // Defines the % to split off into liquid account, and if defined overrides all other splits
    pub split_to_liquid: Option<Decimal>,
    // Used for one-off funds that have an end date (ex. disaster recovery funds)
    pub expiry_time: Option<u64>,   // datetime int of index fund expiry
    pub expiry_height: Option<u64>, // block equiv of the expiry_datetime
}

impl IndexFund {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(expiry_height) = self.expiry_height {
            if env.block.height > expiry_height {
                return true;
            }
        }
        if let Some(expiry_time) = self.expiry_time {
            if env.block.time > Timestamp::from_seconds(expiry_time) {
                return true;
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AcceptedTokens {
    pub native: Vec<String>,
    pub cw20: Vec<String>,
}

impl AcceptedTokens {
    pub fn default() -> Self {
        AcceptedTokens {
            native: vec!["uust".to_string()],
            cw20: vec![],
        }
    }
    pub fn native_valid(&self, denom: String) -> bool {
        matches!(self.native.iter().position(|d| *d == denom), Some(_i))
    }
    pub fn cw20_valid(&self, addr: String) -> bool {
        matches!(self.cw20.iter().position(|a| *a == addr), Some(_i))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BalanceInfo {
    pub locked_balance: GenericBalance,
    pub liquid_balance: GenericBalance,
}

impl BalanceInfo {
    pub fn default() -> Self {
        BalanceInfo {
            locked_balance: GenericBalance::default(),
            liquid_balance: GenericBalance::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

impl GenericBalance {
    pub fn default() -> Self {
        GenericBalance {
            cw20: vec![],
            native: vec![],
        }
    }
    pub fn set_token_balances(&mut self, tokens: Balance) {
        match tokens {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount = token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount = token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
    pub fn get_ust(&self) -> Coin {
        match self.native.iter().find(|t| t.denom == "uusd".to_string()) {
            Some(coin) => coin.clone(),
            None => Coin {
                amount: Uint128::zero(),
                denom: "uusd".to_string(),
            },
        }
    }
    pub fn cw20_list(&self) -> Vec<Cw20Coin> {
        self.cw20
            .clone()
            .into_iter()
            .map(|token| Cw20Coin {
                address: token.address.into(),
                amount: token.amount,
            })
            .collect()
    }
    pub fn get_token_amount(&self, token_address: Addr) -> Uint128 {
        self.cw20_list()
            .iter()
            .filter(|token| token.address == token_address)
            .next()
            .unwrap_or(&Cw20Coin {
                amount: Uint128::zero(),
                address: token_address.to_string(),
            })
            .clone()
            .amount
    }
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
    pub fn deduct_tokens(&mut self, deduct: Balance) {
        match deduct {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount -= token.amount,
                        None => (),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount -= token.amount,
                    None => (),
                }
            }
        };
    }
}
