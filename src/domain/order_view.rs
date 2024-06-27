use fmodel_rust::view::View;
use serde::{Deserialize, Serialize};

use crate::domain::api::{OrderEvent, OrderId, OrderLineItem, OrderStatus, RestaurantId};

/// The state of the Order is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OrderViewState {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub status: OrderStatus,
    pub line_items: Vec<OrderLineItem>,
}

/// A convenient type alias for the Order view
pub type OrderView<'a> = View<'a, Option<OrderViewState>, OrderEvent>;

/// View represents the event handling algorithm. It belongs to the Domain layer.
pub fn order_view<'a>() -> OrderView<'a> {
    View {
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            OrderEvent::Created(event) => Some(OrderViewState {
                identifier: event.identifier.to_owned(),
                restaurant_identifier: event.restaurant_identifier.to_owned(),
                status: event.status.to_owned(),
                line_items: event.line_items.to_owned(),
            }),

            OrderEvent::Prepared(event) => state.clone().map(|s| OrderViewState {
                identifier: event.identifier.to_owned(),
                restaurant_identifier: s.restaurant_identifier,
                status: event.status.to_owned(),
                line_items: s.line_items,
            }),
        }),

        // The initial state of the decider
        initial_state: Box::new(|| None),
    }
}
