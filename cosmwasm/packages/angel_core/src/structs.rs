use crate::errors::core::ContractError;
use crate::msgs::subdao_bonding_token::CurveType;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, SubMsg, Timestamp, Uint128};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

// OLD Struct for purposes of supporting migrations of Endowments (remove in next major version)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OneOffVaults {
    pub locked: Vec<Addr>,
    pub liquid: Vec<Addr>,
}

// OLD Struct for purposes of supporting migrations of Endowments (remove in next major version)
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

// OLD Struct for purposes of supporting migrations of Endowments (remove in next major version)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StrategyComponent {
    pub vault: String,       // Vault SC Address
    pub percentage: Decimal, // percentage of funds to invest
}

#[cw_serde]
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

#[derive(Default)]
#[cw_serde]
pub struct Allowances {
    pub assets: Vec<Asset>,
    pub expires: Vec<Expiration>,
}

#[cw_serde]
pub struct SettingsPermissions {
    owner_controlled: bool,
    gov_controlled: bool,
    modifiable: bool,
    delegate: Option<Delegate>,
}

impl SettingsPermissions {
    pub fn default() -> Self {
        SettingsPermissions {
            owner_controlled: true,
            gov_controlled: false,
            modifiable: true,
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
        if self.modifiable
            && (self.owner_controlled && sender == owner
                || self.gov_controlled && gov.is_some() && sender == gov.unwrap()
                || self.delegate.is_some()
                    && self
                        .delegate
                        .clone()
                        .unwrap()
                        .can_take_action(sender, env_time))
        {
            return true;
        }
        false
    }
}

#[cw_serde]
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

#[cw_serde]
pub struct EndowmentSettings {
    pub dao: Option<Addr>,                     // subdao governance contract address
    pub dao_token: Option<Addr>,               // dao gov token contract address
    pub donation_match_active: bool, // donation matching contract address (None set for Charity Endowments as they just phone home to Registrar to get the addr)
    pub donation_match_contract: Option<Addr>, // contract for donation matching
    pub beneficiaries_allowlist: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub contributors_allowlist: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub maturity_allowlist: Vec<Addr>, // list of addresses, which can withdraw after maturity date is reached (if any)
    pub earnings_fee: Option<EndowmentFee>, // Earnings Fee
    pub withdraw_fee: Option<EndowmentFee>, // Withdraw Fee
    pub deposit_fee: Option<EndowmentFee>, // Deposit Fee
    pub aum_fee: Option<EndowmentFee>, // AUM(Assets Under Management) Fee
    pub parent: Option<u32>,
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
            beneficiaries_allowlist: vec![],
            contributors_allowlist: vec![],
            maturity_allowlist: vec![],
            earnings_fee: None,
            withdraw_fee: None,
            deposit_fee: None,
            aum_fee: None,
            parent: None,
            split_to_liquid: None,
            ignore_user_splits: false,
        }
    }
}

#[cw_serde]
pub struct EndowmentController {
    pub endowment_controller: SettingsPermissions,
    pub strategies: SettingsPermissions,
    pub beneficiaries_allowlist: SettingsPermissions,
    pub contributors_allowlist: SettingsPermissions,
    pub maturity_allowlist: SettingsPermissions,
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
    pub ignore_user_splits: SettingsPermissions,
    pub split_to_liquid: SettingsPermissions,
}

impl EndowmentController {
    pub fn default() -> Self {
        EndowmentController {
            endowment_controller: SettingsPermissions::default(),
            strategies: SettingsPermissions::default(),
            beneficiaries_allowlist: SettingsPermissions::default(),
            contributors_allowlist: SettingsPermissions::default(),
            maturity_allowlist: SettingsPermissions::default(),
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
            ignore_user_splits: SettingsPermissions::default(),
            split_to_liquid: SettingsPermissions::default(),
        }
    }

    pub fn get_permissions(&self, name: String) -> Result<SettingsPermissions, ContractError> {
        match name.as_str() {
            "endowment_controller" => Ok(self.endowment_controller.clone()),
            "strategies" => Ok(self.strategies.clone()),
            "beneficiaries_allowlist" => Ok(self.beneficiaries_allowlist.clone()),
            "contributors_allowlist" => Ok(self.contributors_allowlist.clone()),
            "maturity_allowlist" => Ok(self.maturity_allowlist.clone()),
            "split_to_liquid" => Ok(self.split_to_liquid.clone()),
            "ignore_user_splits" => Ok(self.ignore_user_splits.clone()),
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
            "strategies" => {
                self.strategies = permissions;
                Ok(())
            }
            "beneficiaries_allowlist" => {
                self.beneficiaries_allowlist = permissions;
                Ok(())
            }
            "contributors_allowlist" => {
                self.contributors_allowlist = permissions;
                Ok(())
            }
            "maturity_allowlist" => {
                self.maturity_allowlist = permissions;
                Ok(())
            }
            "split_to_liquid" => {
                self.split_to_liquid = permissions;
                Ok(())
            }
            "ignore_user_splits" => {
                self.ignore_user_splits = permissions;
                Ok(())
            }
            "earnings_fee" => {
                self.earnings_fee = permissions;
                Ok(())
            }
            "withdraw_fee" => {
                self.withdraw_fee = permissions;
                Ok(())
            }
            "deposit_fee" => {
                self.deposit_fee = permissions;
                Ok(())
            }
            "aum_fee" => {
                self.aum_fee = permissions;
                Ok(())
            }
            "kyc_donors_only" => {
                self.kyc_donors_only = permissions;
                Ok(())
            }
            "name" => {
                self.name = permissions;
                Ok(())
            }
            "image" => {
                self.image = permissions;
                Ok(())
            }
            "logo" => {
                self.logo = permissions;
                Ok(())
            }
            "categories" => {
                self.categories = permissions;
                Ok(())
            }
            _ => Err(ContractError::InvalidInputs {}),
        }
    }
}

#[cw_serde]
pub struct Pair {
    pub assets: [AssetInfo; 2],
    pub contract_address: Addr,
}

#[cw_serde]
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

#[cw_serde]
pub enum StrategyApprovalState {
    NotApproved,
    Approved,
    WithdrawOnly,
    Deprecated,
}

// The "locale" of a given Strategy will drive:
// 1. Encoding of the payload (IBC vs EVM)
// 2. Should the Router pass the deposit msg off to a Gateway (IBC/EVM) or a Vault(s) directly (Native)
// 3. Chain hardcoded vs Gateway driven lookup
#[cw_serde]
pub enum StrategyLocale {
    Native,
    Ibc,
    Evm,
}

#[cw_serde]
pub struct StrategyParams {
    pub approval_state: StrategyApprovalState,
    pub locale: StrategyLocale,
    pub chain: String,             // links back to a Network Connection struct
    pub input_denom: String, // should this be in terms of the originating chain where the Accounts need to check sufficient balance on hand or the destination chain?
    pub locked_addr: Option<Addr>, // for EVM Registrars can just hold a 0x00000 for Non-Native?
    pub liquid_addr: Option<Addr>, // for EVM Registrars can just hold a 0x00000 for Non-Native?
}

#[cw_serde]
pub struct StrategyInvestment {
    pub strategy_key: String,
    pub locked_amount: Uint128,
    pub liquid_amount: Uint128,
}

// string keys pointing to StrategyParams for Locked and Liquid
#[cw_serde]
pub struct Investments {
    pub locked: Vec<String>,
    pub liquid: Vec<String>,
}

impl Investments {
    pub fn default() -> Self {
        Investments {
            locked: vec![],
            liquid: vec![],
        }
    }

    pub fn get(&self, acct_type: AccountType) -> Vec<String> {
        match acct_type {
            AccountType::Locked => self.locked.clone(),
            AccountType::Liquid => self.liquid.clone(),
        }
    }
}

#[derive(Default)]
#[cw_serde]
pub struct RedeemResults {
    pub messages: Vec<SubMsg>,
    pub total: Uint128,
}

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
pub enum Beneficiary {
    Endowment { id: u32 },
    IndexFund { id: u64 },
    Wallet { address: String },
}

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
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

#[derive(Default)]
#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
pub enum DonationMatch {
    // Endowment uses HALO Token for their matching reserve (no inputs needed, as we store this info in Registrar)
    HaloTokenReserve {},
    // Endowment uses a different CW20 Token for their mtching reserve
    Cw20TokenReserve {
        reserve_addr: String, // Address of CW20 token, which user wants to use as reserve token in donation_matching
        lp_addr: String, // Address of LP contract (assumes to be a wasmswap/junoswap esque contract)
    },
}

#[cw_serde]
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

#[cw_serde]
pub struct EndowmentFee {
    pub payout_address: Addr,
    pub fee_percentage: Decimal,
    pub active: bool,
}

#[cw_serde]
pub struct NetworkInfo {
    pub router_contract: Option<String>, // router must exist if vaults exist on that chain
    pub accounts_contract: Option<String>, // accounts contract may exist if endowments are on that chain
}

#[cw_serde]
pub struct DonationsReceived {
    pub locked: Uint128,
    pub liquid: Uint128,
}
