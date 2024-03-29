use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Threshold is invalid")]
    InvalidThreshold {},

    #[error("Required threshold cannot be zero")]
    ZeroThreshold {},

    #[error("Not possible to reach required (passing) threshold")]
    UnreachableThreshold {},

    #[error("Group contract invalid address '{addr}'")]
    InvalidGroup { addr: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Proposal is not open")]
    NotOpen {},

    #[error("Proposal voting period has expired")]
    Expired {},

    #[error("Proposal must expire before you can close it")]
    NotExpired {},

    #[error("Wrong expiration option")]
    WrongExpiration {},

    #[error("Already voted on this proposal")]
    AlreadyVoted {},

    #[error("Proposal must have passed and not yet been executed")]
    WrongExecuteStatus {},

    #[error("Cannot close completed or passed proposals")]
    WrongCloseStatus {},
}

impl From<cw_utils::ThresholdError> for ContractError {
    fn from(err: cw_utils::ThresholdError) -> Self {
        match err {
            cw_utils::ThresholdError::Std(std_error) => ContractError::Std(std_error),
            cw_utils::ThresholdError::InvalidThreshold {} => ContractError::InvalidThreshold {},
            cw_utils::ThresholdError::ZeroQuorumThreshold {} => ContractError::ZeroThreshold {},
            cw_utils::ThresholdError::UnreachableQuorumThreshold {} => {
                ContractError::UnreachableThreshold {}
            }
            cw_utils::ThresholdError::ZeroWeight {} => ContractError::UnreachableThreshold {},
            cw_utils::ThresholdError::UnreachableWeight {} => {
                ContractError::UnreachableThreshold {}
            }
        }
    }
}
