use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Inputs are invalid")]
    InvalidInputs {},

    #[error("Rewards cannot be claimed. Campaign is still open.")]
    CampaignIsOpen {},

    #[error("Rewards have already been claimed for this campaign.")]
    CannotClaimRewards {},

    #[error("Only accepts tokens in the cw20_whitelist")]
    NotInWhitelist {},

    #[error("Campaign is expired")]
    Expired {},

    #[error("Cannot close: Campaign is not expired")]
    NotExpired {},

    #[error("Send some coins to create a Campaign")]
    EmptyBalance {},

    #[error("Campaign id already in use")]
    AlreadyInUse {},

    #[error("Recipient is not set")]
    RecipientNotSet {},
}
