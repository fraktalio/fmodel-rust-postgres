use pgrx::prelude::*;
use pgrx::TryFromDatumError;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

/// Error message to be returned to the client
#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
}

/// Implement Display for ErrorMessage
impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Implement Debug for ErrorMessage
impl fmt::Debug for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ErrorMessage: {}", self.message)
    }
}

/// Implement Error for ErrorMessage
impl Error for ErrorMessage {}

#[derive(thiserror::Error, Debug)]
pub enum TriggerError {
    #[error("Null Trigger Tuple found")]
    NullTriggerTuple,
    #[error("PgHeapTuple error: {0}")]
    PgHeapTuple(#[from] PgHeapTupleError),
    #[error("TryFromDatumError error: {0}")]
    TryFromDatum(#[from] TryFromDatumError),
    #[error("TryFromInt error: {0}")]
    TryFromInt(#[from] TryFromIntError),
    #[error("Event Handling Error: {0}")]
    EventHandlingError(String),
}
