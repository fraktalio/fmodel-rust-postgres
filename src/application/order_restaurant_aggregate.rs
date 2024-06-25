use crate::domain::order_decider::Order;
use crate::framework::application::event_sourced_aggregate::EventSourcedOrchestratingAggregate;

use crate::domain::restaurant_decider::Restaurant;
use crate::domain::{Command, Event};
use crate::infrastructure::order_restaurant_event_repository::OrderAndRestaurantEventRepository;

/// A convenient type alias for the order and restaurant aggregate.
pub type OrderAndRestaurantAggregate<'a> = EventSourcedOrchestratingAggregate<
    'a,
    Command,
    (Option<Restaurant>, Option<Order>),
    Event,
    OrderAndRestaurantEventRepository,
>;
