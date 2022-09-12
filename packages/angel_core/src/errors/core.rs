use cosmwasm_std::{OverflowError, StdError, Uint128};
use cw20_base::ContractError as Cw20ContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Base(#[from] Cw20ContractError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Invalid inputs")]
    InvalidInputs {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("Updates are not allowed after endowment has been closed")]
    UpdatesAfterClosed {},

    #[error("Balance for this account is insufficient")]
    BalanceTooSmall {},

    #[error("No Balance for this account")]
    EmptyBalance {},

    #[error("There is already an account for the given ID")]
    AlreadyInUse {},

    #[error("Token was not found in approved coins")]
    NotInApprovedCoins {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Cannot accept coins. Account is not approved yet.")]
    AccountNotApproved {},

    #[error("Cannot alter this account. It has been closed.")]
    AccountClosed {},

    #[error("Account creation error")]
    AccountNotCreated {},

    #[error("Account is already approved")]
    AccountAlreadyApproved {},

    #[error("Account does not exist")]
    AccountDoesNotExist {},

    #[error("Contract is not properly configured")]
    ContractNotConfigured {},

    #[error("Index Fund already exists with given ID")]
    IndexFundAlreadyExists {},

    #[error("Invalid strategy allocation")]
    InvalidStrategyAllocation {},

    #[error("Strategy components should be unique")]
    StrategyComponentsNotUnique {},

    #[error("Invalid deposit split provided")]
    InvalidSplit {},

    #[error("Only accept one coin type per deposit")]
    InvalidCoinsDeposited {},

    #[error("Too many token types returned")]
    TokenTypes {},

    #[error("Cannot withdraw from Locked balances")]
    InaccessableLockedBalance {},

    #[error("Vault redemptions already in progress.")]
    RedemptionInProgress {},

    #[error("Index Fund has expired")]
    IndexFundExpired {},

    #[error("Vault already exists at given address")]
    VaultAlreadyExists {},

    #[error("Index Fund has no members in it")]
    IndexFundEmpty {},

    #[error("Index Fund members limit exceeded")]
    IndexFundMembershipExceeded {},

    #[error("No claims that can be released currently")]
    NothingToClaim {},
}

#[derive(Error, Debug, PartialEq)]
pub enum PaymentError {
    #[error("Must send reserve token '{0}'")]
    MissingDenom(String),

    #[error("Received unsupported denom '{0}'")]
    ExtraDenom(String),

    #[error("Sent more than one denomination")]
    MultipleDenoms {},

    #[error("No funds sent")]
    NoFunds {},

    #[error("This message does no accept funds")]
    NonPayable {},
    #[error("Must provide operations!")]
    MustProvideOperations {},

    #[error("Assertion failed; minimum receive amount: {receive}, swap amount: {amount}")]
    AssertionMinimumReceive { receive: Uint128, amount: Uint128 },
}

impl From<OverflowError> for ContractError {
    fn from(o: OverflowError) -> Self {
        StdError::from(o).into()
    }
}
