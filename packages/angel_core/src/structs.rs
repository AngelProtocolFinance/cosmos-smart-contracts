use cosmwasm_std::{Addr, Coin, Decimal, Decimal256, SubMsg, Timestamp, Uint128};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Locked = 0,
    Liquid = 1,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AccountType::Locked => "locked",
                AccountType::Liquid => "liquid",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pair {
    pub assets: [AssetInfo; 2],
    pub contract_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapOperation {
    JunoSwap {
        offer_asset_info: AssetInfo,
        ask_asset_info: AssetInfo,
    },
    Loop {
        offer_asset_info: AssetInfo,
        ask_asset_info: AssetInfo,
    },
}

impl SwapOperation {
    pub fn get_offer_asset_info(&self) -> AssetInfo {
        match self {
            SwapOperation::JunoSwap {
                offer_asset_info, ..
            } => offer_asset_info.clone(),
            SwapOperation::Loop {
                offer_asset_info, ..
            } => offer_asset_info.clone(),
        }
    }

    pub fn get_ask_asset_info(&self) -> AssetInfo {
        match self {
            SwapOperation::JunoSwap { ask_asset_info, .. } => ask_asset_info.clone(),
            SwapOperation::Loop { ask_asset_info, .. } => ask_asset_info.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct YieldVault {
    pub address: String,
    pub network: String, // Points to key in NetworkConnections storage map
    pub input_denom: String,
    pub yield_token: String,
    pub approved: bool,
    pub restricted_from: Vec<EndowmentType>,
    pub acct_type: AccountType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VaultRate {
    pub vault_addr: String,
    pub fx_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OneOffVaults {
    pub locked: Vec<Addr>,
    pub liquid: Vec<Addr>,
}

impl OneOffVaults {
    pub fn default() -> Self {
        OneOffVaults {
            locked: vec![],
            liquid: vec![],
        }
    }

    pub fn get(&self, acct_type: AccountType) -> Vec<Addr> {
        match acct_type {
            AccountType::Locked => self.locked.clone(),
            AccountType::Liquid => self.liquid.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountStrategies {
    pub locked: Vec<StrategyComponent>,
    pub liquid: Vec<StrategyComponent>,
}

impl AccountStrategies {
    pub fn default() -> Self {
        AccountStrategies {
            locked: vec![],
            liquid: vec![],
        }
    }

    pub fn get(&self, acct_type: AccountType) -> Vec<StrategyComponent> {
        match acct_type {
            AccountType::Locked => self.locked.clone(),
            AccountType::Liquid => self.liquid.clone(),
        }
    }

    pub fn set(&mut self, acct_type: AccountType, strategy: Vec<StrategyComponent>) {
        match acct_type {
            AccountType::Locked => self.locked = strategy,
            AccountType::Liquid => self.liquid = strategy,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StrategyComponent {
    pub vault: String,       // Vault SC Address
    pub percentage: Decimal, // percentage of funds to invest
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
    pub id: u32,
    pub owner: String,
    pub status: EndowmentStatus,
    pub endow_type: EndowmentType,
    pub name: Option<String>,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub tier: Option<Tier>,
    pub categories: Categories,
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
pub enum Tier {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Tier::Level1 => "1",
                Tier::Level2 => "2",
                Tier::Level3 => "3",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Beneficiary {
    Endowment { id: u32 },
    IndexFund { id: u64 },
    Wallet { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum EndowmentType {
    Charity,
    Normal,
}

impl fmt::Display for EndowmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EndowmentType::Charity => "charity",
                EndowmentType::Normal => "normal",
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
    pub members: Vec<u32>,
    pub rotating_fund: Option<bool>, // set a fund as a rotating fund
    // Fund Specific: over-riding SC level setting to handle a fixed split value
    // Defines the % to split off into liquid account, and if defined overrides all other splits
    pub split_to_liquid: Option<Decimal>,
    // Used for one-off funds that have an end date (ex. disaster recovery funds)
    pub expiry_time: Option<u64>,   // datetime int of index fund expiry
    pub expiry_height: Option<u64>, // block equiv of the expiry_datetime
}

impl IndexFund {
    pub fn is_expired(&self, env_height: u64, env_time: Timestamp) -> bool {
        if (self.expiry_height != None && env_height >= self.expiry_height.unwrap())
            || (self.expiry_time != None
                && env_time >= Timestamp::from_seconds(self.expiry_time.unwrap()))
        {
            return true;
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
            native: vec![
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".to_string(),
            ],
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
pub struct EndowmentBalanceResponse {
    pub tokens_on_hand: BalanceInfo,
    pub oneoff_locked: Vec<(String, Uint128)>,
    pub oneoff_liquid: Vec<(String, Uint128)>,
    pub strategies_locked: Vec<(String, Uint128)>,
    pub strategies_liquid: Vec<(String, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct VaultsBalanceInfo {
    locked: Vec<(String, Uint128)>,
    liquid: Vec<(String, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BalanceInfo {
    pub locked: GenericBalance,
    pub liquid: GenericBalance,
}

impl BalanceInfo {
    pub fn default() -> Self {
        BalanceInfo {
            locked: GenericBalance::default(),
            liquid: GenericBalance::default(),
        }
    }

    pub fn get(&self, acct_type: &AccountType) -> GenericBalance {
        match *acct_type {
            AccountType::Locked => self.locked.clone(),
            AccountType::Liquid => self.liquid.clone(),
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SocialMedialUrls {
    pub facebook: Option<String>,
    pub twitter: Option<String>,
    pub linkedin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllianceMember {
    pub name: String,
    pub logo: Option<String>,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TransactionRecord {
    pub block: u64,
    pub sender: Addr,
    pub recipient: Option<Addr>,
    pub assets: GenericBalance,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Categories {
    pub sdgs: Vec<u8>, // u8 maps one of the 17 UN SDG
    pub general: Vec<u8>,
}

impl Categories {
    fn default() -> Self {
        Categories {
            sdgs: vec![],
            general: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Profile {
    pub name: String, // name of the Charity Endowment
    pub overview: String,
    pub categories: Categories, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub tier: Option<u8>, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub logo: Option<String>,
    pub image: Option<String>,
    pub url: Option<String>,
    pub registration_number: Option<String>,
    pub country_of_origin: Option<String>,
    pub street_address: Option<String>,
    pub contact_email: Option<String>,
    pub social_media_urls: SocialMedialUrls,
    pub number_of_employees: Option<u16>,
    pub average_annual_budget: Option<String>,
    pub annual_revenue: Option<String>,
    pub charity_navigator_rating: Option<String>,
    pub endow_type: EndowmentType,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            name: "".to_string(),
            overview: "".to_string(),
            categories: Categories::default(),
            tier: None,
            logo: None,
            image: None,
            url: None,
            registration_number: None,
            country_of_origin: None,
            street_address: None,
            contact_email: None,
            social_media_urls: SocialMedialUrls {
                facebook: None,
                twitter: None,
                linkedin: None,
            },
            number_of_employees: None,
            average_annual_budget: None,
            annual_revenue: None,
            charity_navigator_rating: None,
            endow_type: EndowmentType::Charity,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NetworkInfo {
    pub name: String,
    pub chain_id: String,
    pub ibc_channel: Option<String>,
    pub ica_address: Option<Addr>,
    pub gas_limit: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DonationsReceived {
    pub locked: Uint128,
    pub liquid: Uint128,
}
