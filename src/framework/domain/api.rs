use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A trait for identifying messages/events/commands
pub trait Identifier {
    fn identifier(&self) -> Uuid;
}

/// A trait for identifying the type/name of an event
pub trait EventType {
    fn event_type(&self) -> String;
}

/// A trait for identifying if an event is final
pub trait IsFinal {
    fn is_final(&self) -> bool;
}

/// A trait for identifying the type/name of a decider in the event.
pub trait DeciderType {
    fn decider_type(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum DomainError {
    RestaurantNotCreated(String),
    RestaurantMenuNotChanged(String),
    OrderNotPlaced(String),
    OrderNotCreated(String),
    OrderNotPrepared(String),
}

/// Implement Display for DomainError
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
