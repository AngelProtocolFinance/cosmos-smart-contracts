use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Nothing staked")]
    NothingStaked {},

    #[error("Nothing to distribute")]
    NothingToDistribute {},
}
