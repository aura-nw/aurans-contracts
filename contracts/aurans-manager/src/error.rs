use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Already registered")]
    UserRegistered {},

    #[error("Verification Error")]
    VerificationError,

    #[error("Invalid Signature")]
    InvalidSignature,

    #[error("Invalid Address")]
    InvalidAddress,

    #[error("Serde Error")]
    SerdeError,

    #[error("Insufficient Funds")]
    InsufficientFunds,

    #[error("Invalid Arguments")]
    InvalidArguments,

    #[error("Limit Year Register")]
    LimitYearRegister,

    #[error("Invalid Timestamp blocktime: {blocktime:?}")]
    InvalidTimestamp { blocktime: String },

    #[error("Name Has Registed: {name:?}")]
    NameRegisted { name: String },

    #[error("Name Not Registed: {name:?}")]
    NameNotRegisted { name: String },
}
