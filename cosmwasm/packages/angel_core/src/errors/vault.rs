use cosmwasm_std::StdError;
use cw20_base::ContractError as cw20ContractError;
use cw_asset::AssetError;
use thiserror::Error;

#[derive(Error, Debug)]
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

impl From<cw20ContractError> for ContractError {
    fn from(error: cw20ContractError) -> Self {
        match error {
            cw20ContractError::CannotExceedCap {} => ContractError::CannotExceedCap {},
            cw20ContractError::CannotSetOwnAccount {} => ContractError::CannotSetOwnAccount {},
            cw20ContractError::DuplicateInitialBalanceAddresses {} => {
                ContractError::Std(StdError::GenericErr {
                    msg: "Duplicate initial balance address".to_string(),
                })
            }
            cw20ContractError::Expired {} => ContractError::Std(StdError::GenericErr {
                msg: "Expired".to_string(),
            }),
            cw20ContractError::InvalidPngHeader {} => ContractError::Std(StdError::GenericErr {
                msg: "Invalid png header".to_string(),
            }),
            cw20ContractError::InvalidXmlPreamble {} => ContractError::Std(StdError::GenericErr {
                msg: "Invalid xml preamble".to_string(),
            }),
            cw20ContractError::InvalidZeroAmount {} => ContractError::Std(StdError::GenericErr {
                msg: "Invalid zero amount".to_string(),
            }),
            cw20ContractError::LogoTooBig {} => ContractError::Std(StdError::GenericErr {
                msg: "Logo Too Big".to_string(),
            }),
            cw20ContractError::NoAllowance {} => ContractError::Std(StdError::GenericErr {
                msg: "No allowance".to_string(),
            }),
            cw20ContractError::Std(e) => ContractError::Std(e),
            cw20ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw20ContractError::InvalidExpiration {} => todo!(),
        }
    }
}

impl From<AssetError> for ContractError {
    fn from(_error: AssetError) -> Self {
        ContractError::Std(StdError::GenericErr {
            msg: "An asset error occured".to_string(),
        })
    }
}
