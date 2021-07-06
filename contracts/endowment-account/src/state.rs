use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Env, Timestamp};
use cw_storage_plus::{Item, Map};

use cw20::{Balance, Cw20CoinVerified};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Account {
    // Arbiter is the AP team Addr, initially, & DANO in long-term
    // In charge of approving/whitelisting an endowment & closing the accounts
    pub arbiter: Addr,
    // Once approved by the Arbiter, the accounts can accept incoming funds
    pub approved: bool,
    // Funds can be only be dispersed to the beneficiary of the endowment in the case of winding up
    // All other funds must flow to the attached Liquid Account in order to be withdrawn.
    pub beneficiary: Addr,
    // The Addr that setup the Endowment account
    pub originator: Addr,
    // Endowment Acct Balance, in Native and/or Cw20 tokens
    pub balance: GenericBalance,
    // Liquid Account connected to the endowment account (created upon Approval by Arbiter)
    pub liquid_acct: Option<String>,
    // When end height set and block height exceeds this value, the account is expired.
    // Once an account is expired, it can be returned to the owner (via "refund").
    pub end_height: Option<u64>,
    // When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    // block time exceeds this value, the account is expired.
    // Once an account is expired, it can be returned to the owner (via "refund").
    pub end_time: Option<u64>,
}

impl Account {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(end_height) = self.end_height {
            if env.block.height > end_height {
                return true;
            }
        }
        if let Some(end_time) = self.end_time {
            if env.block.time > Timestamp::from_seconds(end_time) {
                return true;
            }
        }
        false
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ACCOUNTS: Map<String, Account> = Map::new("account");
