use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No Balance for this account")]
    EmptyBalance {},

    #[error("There is already an account for the given address")]
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
}
