use crate::domain::{Command, Event};
use crate::framework::infrastructure::event_repository::EventOrchestratingRepository;

/// An event repository for the restaurant and order domain(s).
pub struct OrderAndRestaurantEventRepository {}

/// Implementation of the event orchestrating repository for the restaurant and order domain(s).
impl EventOrchestratingRepository<Command, Event> for OrderAndRestaurantEventRepository {}

impl OrderAndRestaurantEventRepository {
    /// Creates a new restaurant and order event repository.
    pub fn new() -> Self {
        OrderAndRestaurantEventRepository {}
    }
}
