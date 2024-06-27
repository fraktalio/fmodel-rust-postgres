use crate::framework::infrastructure::errors::ErrorMessage;
use pgrx::JsonB;
use serde::de::DeserializeOwned;

pub mod errors;
pub mod event_repository;
pub mod view_state_repository;

/// Converts a `JsonB` to the payload type.
pub fn to_payload<E: DeserializeOwned>(jsonb: JsonB) -> Result<E, ErrorMessage> {
    let value = jsonb.0.clone();
    serde_json::from_value(value).map_err(|err| ErrorMessage {
        message: "Failed to deserialize payload: ".to_string() + &err.to_string(),
    })
}
