use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Unauthorized sender: {sender:?}")]
    Unauthorized { sender: String },

    #[error("Already registered")]
    UserRegistered {},

    // Wrapper cw721 error
    #[error("{0}")]
    CW721Base(#[from] cw721_base::ContractError),

    #[error("Batch too long")]
    BatchTooLong {},

    #[error("Invalid token id")]
    InvalidTokenId {},
}
