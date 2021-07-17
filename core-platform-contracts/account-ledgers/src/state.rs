use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

use cw20::{Balance, Cw20CoinVerified};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // DANO Address
    pub charity_endowment_sc: Option<Addr>, // Address of Charity Endowment SC
    pub index_fund_sc: Option<Addr>, // Address of Index Fund SC
    // List of all possible contracts that we can accept Cw20 tokens from
    // that are accepted by the account during a top-up. This is required to avoid a DoS attack by topping-up
    // with an invalid cw20 contract. See https://github.com/CosmWasm/cosmwasm-plus/issues/19
    pub cw20_approved_coins: Vec<Addr>,
}

impl Config {
    // Convert a Vec of Addr to a Vec of Human-readable Strings
    pub fn human_approved_coins(&self) -> Vec<String> {
        self.cw20_approved_coins
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub balance: GenericBalance,
    pub strategy: Strategy,
    pub split_deposit: SplitDetails,
    pub split_interest: SplitDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SplitDetails {
    pub max: u8,
    pub min: u8,
    pub default: u8, // for when a split parameter is not provided
}

impl SplitDetails {
    pub fn default() -> Self {
        SplitDetails {
            min: 0,
            max: 100,
            default: 50,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Strategy {
    pub invested: Vec<StrategyComponent>,
    pub cash: u8, // possibly use impl function to calculate remainder from invested strategies instead?
}

impl Strategy {
    pub fn default() -> Self {
        Strategy {
            invested: vec![],
            cash: 100,
        }
    }
}
// TO DO: Add impl function to check strategy percentages + cash remaining all sums to 100%

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StrategyComponent {
    pub address: Addr, // Asset Vault SC Address
    pub percentage: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

impl GenericBalance {
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetVault {
    pub name: String,
    pub description: String,
    pub approved: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ACCOUNTS: Map<String, Account> = Map::new("account");
pub const VAULTS: Map<String, AssetVault> = Map::new("vault");
