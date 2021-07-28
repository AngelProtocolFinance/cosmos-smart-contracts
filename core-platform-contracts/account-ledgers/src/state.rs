use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};

use cw20::{Balance, Cw20CoinVerified};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner_addr: Addr,                 // DANO Address
    pub charity_endowment_contract: Addr, // Address of Charity Endowment SC
    pub index_fund_contract: Addr,        // Address of Index Fund SC
    // List of all possible CW20 Token demoninations that we can accept
    // This is required to avoid a DoS attack with an invalid cw20 contract. See https://github.com/CosmWasm/cosmwasm-plus/issues/19
    pub approved_coins: Vec<Addr>,
}

impl Config {
    pub fn human_approved_coins(&self) -> Vec<String> {
        self.approved_coins.iter().map(|a| a.to_string()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub balance: GenericBalance,
    pub strategy: Strategy,
    pub rebalance: RebalanceDetails,
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
