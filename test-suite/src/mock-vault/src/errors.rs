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

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Vault not created")]
    VaultNotCreated {},

    #[error("Only accept one coin type per deposit")]
    InvalidCoinsDeposited {},

    #[error("No Balance found")]
    EmptyBalance {},
}
