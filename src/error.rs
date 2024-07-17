use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("InvalidInput: {0}")]
    InvalidInput(String),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid Funds")]
    InvalidFunds(),

}
