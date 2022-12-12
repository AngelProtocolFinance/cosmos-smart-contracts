use crate::errors::core::ContractError;
use crate::messages::subdao_bonding_token::CurveType;
use cosmwasm_std::{Addr, Coin, Decimal, SubMsg, Timestamp, Uint128};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegate {
    address: Addr,
    expires: Option<u64>, // datetime int of delegation expiry
}

impl Delegate {
    pub fn can_take_action(&self, sender: &Addr, env_time: Timestamp) -> bool {
        sender == &self.address
            && (self.expires.is_none()
                || env_time >= Timestamp::from_seconds(self.expires.unwrap()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SettingsPermissions {
    owner_controlled: bool,
    gov_controlled: bool,
    modifiable_after_init: bool,
    delegate: Option<Delegate>,
}

impl SettingsPermissions {
    pub fn default() -> Self {
        SettingsPermissions {
            owner_controlled: true,
            gov_controlled: false,
            modifiable_after_init: true,
            delegate: None,
        }
    }

    pub fn set_delegate(
        &mut self,
        sender: &Addr,
        owner: &Addr,
        gov: Option<&Addr>,
        delegate_addr: Addr,
        delegate_expiry: Option<u64>,
    ) {
        if sender == owner && self.owner_controlled
            || gov.is_some() && self.gov_controlled && sender == gov.unwrap()
        {
            self.delegate = Some(Delegate {
                address: delegate_addr,
                expires: delegate_expiry,
            })
        }
    }

    pub fn revoke_delegate(
        &mut self,
        sender: &Addr,
        owner: &Addr,
        gov: Option<&Addr>,
        env_time: Timestamp,
    ) {
        if sender == owner && self.owner_controlled
            || gov.is_some() && self.gov_controlled && sender == gov.unwrap()
            || self.delegate.is_some()
                && self
                    .delegate
                    .clone()
                    .unwrap()
                    .can_take_action(sender, env_time)
        {
            self.delegate = None
        }
    }

    pub fn can_change(
        &self,
        sender: &Addr,
        owner: &Addr,
        gov: Option<&Addr>,
        env_time: Timestamp,
    ) -> bool {
        if sender == owner && self.owner_controlled
            || gov.is_some() && self.gov_controlled && sender == gov.unwrap()
            || self.delegate.is_some()
                && self
                    .delegate
                    .clone()
                    .unwrap()
                    .can_take_action(sender, env_time)
        {
            if self.modifiable_after_init {
                return true;
            }
        }
        return false;
    }
}

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
pub struct EndowmentSettings {
    pub dao: Option<Addr>,                     // subdao governance contract address
    pub dao_token: Option<Addr>,               // dao gov token contract address
    pub donation_match_active: bool, // donation matching contract address (None set for Charity Endowments as they just phone home to Registrar to get the addr)
    pub donation_match_contract: Option<Addr>, // contract for donation matching
    pub whitelisted_beneficiaries: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub maturity_whitelist: Vec<Addr>, // list of addresses, which can withdraw after maturity date is reached (if any)
    pub earnings_fee: Option<EndowmentFee>, // Earnings Fee
    pub withdraw_fee: Option<EndowmentFee>, // Withdraw Fee
    pub deposit_fee: Option<EndowmentFee>, // Deposit Fee
    pub aum_fee: Option<EndowmentFee>, // AUM(Assets Under Management) Fee
    pub settings_controller: SettingsController,
    pub parent: Option<u64>,
    pub split_to_liquid: Option<SplitDetails>, // set of max, min, and default Split paramenters to check user defined split input against
    pub ignore_user_splits: bool, // ignore user-submitted splits in favor of the default splits
}

impl EndowmentSettings {
    pub fn default() -> Self {
        EndowmentSettings {
            dao: None,
            dao_token: None,
            donation_match_active: false,
            donation_match_contract: None,
            whitelisted_beneficiaries: vec![],
            whitelisted_contributors: vec![],
            maturity_whitelist: vec![],
            earnings_fee: None,
            withdraw_fee: None,
            deposit_fee: None,
            aum_fee: None,
            settings_controller: SettingsController::default(),
            parent: None,
            split_to_liquid: None,
            ignore_user_splits: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SettingsController {
    pub settings_controller: SettingsPermissions,
    pub strategies: SettingsPermissions,
    pub whitelisted_beneficiaries: SettingsPermissions,
    pub whitelisted_contributors: SettingsPermissions,
    pub maturity_time: SettingsPermissions,
    pub profile: SettingsPermissions,
    pub earnings_fee: SettingsPermissions,
    pub withdraw_fee: SettingsPermissions,
    pub deposit_fee: SettingsPermissions,
    pub aum_fee: SettingsPermissions,
    pub kyc_donors_only: SettingsPermissions,
    pub name: SettingsPermissions,
    pub image: SettingsPermissions,
    pub logo: SettingsPermissions,
    pub categories: SettingsPermissions,
}

impl SettingsController {
    pub fn default() -> Self {
        SettingsController {
            settings_controller: SettingsPermissions::default(),
            strategies: SettingsPermissions::default(),
            whitelisted_beneficiaries: SettingsPermissions::default(),
            whitelisted_contributors: SettingsPermissions::default(),
            maturity_time: SettingsPermissions::default(),
            profile: SettingsPermissions::default(),
            earnings_fee: SettingsPermissions::default(),
            withdraw_fee: SettingsPermissions::default(),
            deposit_fee: SettingsPermissions::default(),
            aum_fee: SettingsPermissions::default(),
            kyc_donors_only: SettingsPermissions::default(),
            name: SettingsPermissions::default(),
            image: SettingsPermissions::default(),
            logo: SettingsPermissions::default(),
            categories: SettingsPermissions::default(),
        }
    }

    pub fn get_permissions(&self, name: String) -> Result<SettingsPermissions, ContractError> {
        match name.as_str() {
            "settings_controller" => Ok(self.settings_controller.clone()),
            "strategies" => Ok(self.strategies.clone()),
            "whitelisted_beneficiaries" => Ok(self.whitelisted_beneficiaries.clone()),
            "whitelisted_contributors" => Ok(self.whitelisted_contributors.clone()),
            "maturity_time" => Ok(self.maturity_time.clone()),
            "profile" => Ok(self.profile.clone()),
            "earnings_fee" => Ok(self.earnings_fee.clone()),
            "withdraw_fee" => Ok(self.withdraw_fee.clone()),
            "deposit_fee" => Ok(self.deposit_fee.clone()),
            "aum_fee" => Ok(self.aum_fee.clone()),
            "kyc_donors_only" => Ok(self.kyc_donors_only.clone()),
            "name" => Ok(self.name.clone()),
            "image" => Ok(self.image.clone()),
            "logo" => Ok(self.logo.clone()),
            "categories" => Ok(self.categories.clone()),
            _ => Err(ContractError::InvalidInputs {}),
        }
    }

    pub fn set_permissions(
        &mut self,
        name: String,
        permissions: SettingsPermissions,
    ) -> Result<(), ContractError> {
        match name.as_str() {
            "strategies" => Ok(self.strategies = permissions),
            "whitelisted_beneficiaries" => Ok(self.whitelisted_beneficiaries = permissions),
            "whitelisted_contributors" => Ok(self.whitelisted_contributors = permissions),
            "maturity_time" => Ok(self.maturity_time = permissions),
            "profile" => Ok(self.profile = permissions),
            "earnings_fee" => Ok(self.earnings_fee = permissions),
            "withdraw_fee" => Ok(self.withdraw_fee = permissions),
            "deposit_fee" => Ok(self.deposit_fee = permissions),
            "aum_fee" => Ok(self.aum_fee = permissions),
            "kyc_donors_only" => Ok(self.kyc_donors_only = permissions),
            "name" => Ok(self.name = permissions),
            "image" => Ok(self.image = permissions),
            "logo" => Ok(self.logo = permissions),
            "categories" => Ok(self.categories = permissions),
            _ => Err(ContractError::InvalidInputs {}),
        }
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

    pub fn reverse_operation(&self) -> Self {
        match self {
            SwapOperation::JunoSwap {
                offer_asset_info,
                ask_asset_info,
            } => SwapOperation::JunoSwap {
                offer_asset_info: ask_asset_info.clone(),
                ask_asset_info: offer_asset_info.clone(),
            },
            SwapOperation::Loop {
                offer_asset_info,
                ask_asset_info,
            } => SwapOperation::Loop {
                offer_asset_info: ask_asset_info.clone(),
                ask_asset_info: offer_asset_info.clone(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VaultType {
    Native,              // Juno native Vault contract
    Ibc { ica: String }, // the address of the Vault contract on it's Cosmos(non-Juno) chain
    Evm,                 // the address of the Vault contract on it's EVM chain
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct YieldVault {
    pub address: String, // vault's contract address on chain where the Registrar lives
    pub network: String, // Points to key in NetworkConnections storage map
    pub input_denom: String,
    pub yield_token: String,
    pub approved: bool,
    pub restricted_from: Vec<EndowmentType>,
    pub acct_type: AccountType,
    pub vault_type: VaultType,
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
        if (self.expiry_height.is_some() && env_height >= self.expiry_height.unwrap())
            || (self.expiry_time.is_some()
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
    pub invested_locked: Vec<(String, Uint128)>,
    pub invested_liquid: Vec<(String, Uint128)>,
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
pub struct AllianceMember {
    pub name: String,
    pub logo: Option<String>,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DaoSetup {
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
    pub token: DaoToken,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DaoToken {
    // Option1: Existing cw20 contract
    ExistingCw20(String),
    // Create new cw20 with initial-supply
    NewCw20 {
        initial_supply: Uint128,
        name: String,
        symbol: String,
    },
    // Option3: Create new "bonding-curve"
    BondingCurve {
        curve_type: CurveType,
        name: String,
        symbol: String,
        decimals: u8,
        reserve_denom: String,
        reserve_decimals: u8,
        unbonding_period: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DonationMatch {
    // Endowment uses HALO Token for their matching reserve (no inputs needed, as we store this info in Registrar)
    HaloTokenReserve {},
    // Endowment uses a different CW20 Token for their mtching reserve
    Cw20TokenReserve {
        reserve_addr: String, // Address of CW20 token, which user wants to use as reserve token in donation_matching
        lp_addr: String, // Address of LP contract (assumes to be a wasmswap/junoswap esque contract)
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
    pub fn default() -> Self {
        Categories {
            sdgs: vec![],
            general: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EndowmentFee {
    pub payout_address: Addr,
    pub fee_percentage: Decimal,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NetworkInfo {
    pub name: String,
    pub chain_id: String,
    pub ibc_channel: Option<String>,
    pub transfer_channel: Option<String>,
    pub ibc_host_contract: Option<Addr>,
    pub gas_limit: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DonationsReceived {
    pub locked: Uint128,
    pub liquid: Uint128,
}
