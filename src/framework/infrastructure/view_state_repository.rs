use crate::framework::infrastructure::errors::ErrorMessage;

/// A trait for a view state repository / the query side of the CQRS pattern.
pub trait ViewStateRepository<E, S> {
    /// Fetches current state, based on the event.
    fn fetch_state(&self, event: &E) -> Result<Option<S>, ErrorMessage>;
    /// Saves the new state.
    fn save(&self, state: &S) -> Result<S, ErrorMessage>;
}
