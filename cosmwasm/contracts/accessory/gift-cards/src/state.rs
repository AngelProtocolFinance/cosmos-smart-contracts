use cosmwasm_std::{Addr, Coin, Uint128};
use cw_asset::Asset;
use cw_storage_plus::{Item, Map};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw_asset::{AssetInfoBase};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // AP TEAM MULTISIG
    pub keeper: Addr,
    pub registrar_contract: Addr,
    pub next_deposit: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Deposit {
    pub sender: Addr,
    pub token: Asset,
    pub claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
    pub fn get_denom_amount(&self, denom: String) -> Asset {
        let coin = match self.native.iter().find(|t| t.denom == *denom) {
            Some(coin) => coin.clone(),
            None => Coin {
                amount: Uint128::zero(),
                denom: denom.to_string(),
            },
        };
        Asset {
            info: AssetInfoBase::Native(coin.denom),
            amount: coin.amount,
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
    pub fn get_token_amount(&self, token_address: Addr) -> Asset {
        let amount = self
            .cw20_list()
            .iter()
            .find(|token| token.address == token_address)
            .unwrap_or(&Cw20Coin {
                amount: Uint128::zero(),
                address: token_address.to_string(),
            })
            .clone()
            .amount;

        Asset {
            info: AssetInfoBase::Cw20(token_address),
            amount,
        }
    }
    pub fn receive_generic_balance(&mut self, tokens: GenericBalance) {
        for token in tokens.native.iter() {
            let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                if exist.denom == token.denom {
                    Some(i)
                } else {
                    None
                }
            });
            match index {
                Some(idx) => self.native[idx].amount += token.amount,
                None => self.native.push(token.clone()),
            }
        }
        for token in tokens.cw20.iter() {
            let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                if exist.address == token.address {
                    Some(i)
                } else {
                    None
                }
            });
            match index {
                Some(idx) => self.cw20[idx].amount += token.amount,
                None => self.cw20.push(token.clone()),
            }
        }
    }
    pub fn split_balance(&mut self, split_factor: Uint128) -> GenericBalance {
        let mut split_bal = self.clone();
        split_bal.native = split_bal
            .native
            .iter()
            .map(|token| Coin {
                denom: token.denom.clone(),
                amount: token.amount.checked_div(split_factor).unwrap(),
            })
            .collect();
        split_bal.cw20 = split_bal
            .cw20
            .iter()
            .enumerate()
            .map(|(_i, token)| Cw20CoinVerified {
                address: token.address.clone(),
                amount: token.amount.checked_div(split_factor).unwrap(),
            })
            .collect();
        split_bal
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
                    if let Some(idx) = index {
                        self.native[idx].amount -= token.amount
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
                if let Some(idx) = index {
                    self.cw20[idx].amount -= token.amount
                }
            }
        };
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DEPOSITS: Map<u64, Deposit> = Map::new("deposit");
pub const BALANCES: Map<Addr, GenericBalance> = Map::new("balance");
