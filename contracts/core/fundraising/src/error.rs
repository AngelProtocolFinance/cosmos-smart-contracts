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

    #[error("Contributions have already been refunded for this campaign.")]
    AlreadyRefunded {},

    #[error(
        "Only accepts {token_type} tokens that have been whitelisted. Supplied '{given}' is not approved."
    )]
    NotInWhitelist { token_type: String, given: String },

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
