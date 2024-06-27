use fmodel_rust::decider::Decider;
use pgrx::error;

use crate::domain::api::{
    OrderCommand, OrderCreated, OrderEvent, OrderId, OrderLineItem, OrderPrepared, OrderStatus,
    RestaurantId,
};

/// The state of the Order is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug)]
pub struct Order {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub status: OrderStatus,
    pub line_items: Vec<OrderLineItem>,
}

/// A convenient type alias for the Order decider
pub type OrderDecider<'a> = Decider<'a, OrderCommand, Option<Order>, OrderEvent>;

/// Decider is a datatype/struct that represents the main decision-making algorithm. It belongs to the Domain layer.
pub fn order_decider<'a>() -> OrderDecider<'a> {
    Decider {
        // Decide new events based on the current state and the command
        // Exhaustive pattern matching on the command
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(command) => {
                if state.is_some() {
                    error!("Failed to create the Order. Order already exists!")
                } else {
                    vec![OrderEvent::Created(OrderCreated {
                        identifier: command.identifier.to_owned(),
                        restaurant_identifier: command.restaurant_identifier.to_owned(),
                        status: OrderStatus::Created,
                        line_items: command.line_items.to_owned(),
                        r#final: false,
                    })]
                }
            }
            OrderCommand::MarkAsPrepared(command) => {
                if state
                    .clone()
                    .is_some_and(|s| OrderStatus::Created == s.status)
                {
                    vec![OrderEvent::Prepared(OrderPrepared {
                        identifier: command.identifier.to_owned(),
                        status: OrderStatus::Prepared,
                        r#final: true,
                    })]
                } else {
                    error!("Failed to mark the order as prepared. Order does not exist or is not in the correct state!");
                }
            }
        }),
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            OrderEvent::Created(event) => Some(Order {
                identifier: event.identifier.to_owned(),
                restaurant_identifier: event.restaurant_identifier.to_owned(),
                status: event.status.to_owned(),
                line_items: event.line_items.to_owned(),
            }),
            OrderEvent::Prepared(event) => state.clone().map(|s| Order {
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
