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
