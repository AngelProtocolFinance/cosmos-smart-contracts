use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const CONFIG_KEY: &str = "config";
pub const ACCOUNTS_KEY: &str = "accounts";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Owner of the smart contract (the one who instantiate)
    pub owner: Addr,

    // Address of Locked Account smart contract
    pub locked_account: Option<Addr>,

    // Address of Index Fund smart contract
    pub index_fund: Option<Addr>,

    // Address of Investment Strategy smart contract
    pub investment_strategy: Option<Addr>,

    //
    // pub cw20_approved_coins: Vec<Addr>, // TODO: should it be there or could there be some kind of a link to another SC (Locked SC or more general)?
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SplitParameters {
    pub max: u8,
    pub min: u8,
    pub default: u8, // TODO: should there be a default value?
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Splits {
    // Split Parameters for Deposits
    pub deposit: SplitParameters,

    // Split Parameters for Interest
    pub interest: SplitParameters,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Account {
    // Arbiter is the AP team Addr, initially, & DANO in long-term
    // In charge of approving/whitelisting an endowment & closing the accounts
    pub arbiter: Addr, // TODO: Should it be there or in Config?

    // Once approved by the Arbiter, the accounts can accept incoming funds
    pub approved: bool,

    // Funds can be only be dispersed to the beneficiary of the endowment in the case of winding up
    // All other funds must flow to the attached Liquid Account in order to be withdrawn.
    pub beneficiary: Addr, // TODO: Should it be as separate Account parameter or beneficiary === originator

    // The Addr that setup the Liquid Account // TODO: should pass from Liquid Account?
    pub originator: Addr, // TODO: does neccessary parameter or originator == Locked Account

    // Split parameters for incoming deposits and interest payments
    // controls how much to send to Liquid vs Locked Account
    pub splits: Splits,

    // When end height set and block height exceeds this value, the account is expired.
    // Once an account is expired, it can be returned to the owner (via "refund").
    pub end_height: Option<u64>,

    // When end time (in seconds since epoch 00:00:00 UTC on 1 January 1970) is set and
    // block time exceeds this value, the account is expired.
    // Once an account is expired, it can be returned to the owner (via "refund").
    pub end_time: Option<u64>,
}

pub const ACCOUNTS: Map<String, Account> = Map::new(ACCOUNTS_KEY); // TODO: could it be Map<Addr, Account>?
// TODO: should ACCOUNTS constant be separately as in Locked Account, as in Liquid Account?
