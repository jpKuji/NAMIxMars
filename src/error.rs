use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("ICA Memo is unknown")]
    UnknownMemo {},

    #[error("Memo Format is unknown: {0}")]
    InvalidMemoFormat(String),

    #[error("No credit account found")]
    NoCreditAccount {},

    #[error("Outpost not found for destination: {destination}")]
    DestinationNotFound { destination: String },

    #[error("Ica Query Error: {error}")]
    IcaQueryError { error: String },

    #[error("Failed Parsing Amount")]
    InvalidAmount {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
