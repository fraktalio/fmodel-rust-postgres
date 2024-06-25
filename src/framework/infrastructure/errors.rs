use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

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
