use fmodel_rust::decider::Decider;

use crate::domain::api::{
    OrderNotPlaced, OrderPlaced, Reason, RestaurantCommand, RestaurantCreated, RestaurantEvent,
    RestaurantId, RestaurantMenu, RestaurantMenuChanged, RestaurantMenuNotChanged, RestaurantName,
    RestaurantNotCreated,
};

/// The state of the Restaurant is represented by this struct. It belongs to the Domain layer.
#[derive(Clone, PartialEq, Debug)]
pub struct Restaurant {
    identifier: RestaurantId,
    name: RestaurantName,
    menu: RestaurantMenu,
}

/// A convenient type alias for the Restaurant decider
pub type RestaurantDecider<'a> =
    Decider<'a, RestaurantCommand, Option<Restaurant>, RestaurantEvent>;

/// Decider is a datatype/struct that represents the main decision-making algorithm. It belongs to the Domain layer.
pub fn restaurant_decider<'a>() -> RestaurantDecider<'a> {
    Decider {
        // Decide new events based on the current state and the command
        // Exhaustive pattern matching on the command
        decide: Box::new(|command, state| match command {
            RestaurantCommand::CreateRestaurant(command) => {
                if state.is_some() {
                    vec![RestaurantEvent::NotCreated(RestaurantNotCreated {
                        identifier: command.identifier.to_owned(),
                        name: command.name.to_owned(),
                        menu: command.menu.to_owned(),
                        reason: Reason("Restaurant already exists".to_string()),
                        r#final: false,
                    })]
                } else {
                    vec![RestaurantEvent::Created(RestaurantCreated {
                        identifier: command.identifier.to_owned(),
                        name: command.name.to_owned(),
                        menu: command.menu.to_owned(),
                        r#final: false,
                    })]
                }
            }
            RestaurantCommand::ChangeMenu(command) => {
                if state.is_some() {
                    vec![RestaurantEvent::MenuChanged(RestaurantMenuChanged {
                        identifier: command.identifier.to_owned(),
                        menu: command.menu.to_owned(),
                        r#final: false,
                    })]
                } else {
                    vec![RestaurantEvent::MenuNotChanged(RestaurantMenuNotChanged {
                        identifier: command.identifier.to_owned(),
                        menu: command.menu.to_owned(),
                        reason: Reason("Restaurant does not exist".to_string()),
                        r#final: false,
                    })]
                }
            }
            RestaurantCommand::PlaceOrder(command) => {
                if state.is_some() {
                    vec![RestaurantEvent::OrderPlaced(OrderPlaced {
                        identifier: command.identifier.to_owned(),
                        order_identifier: command.order_identifier.to_owned(),
                        line_items: command.line_items.to_owned(),
                        r#final: false,
                    })]
                } else {
                    vec![RestaurantEvent::OrderNotPlaced(OrderNotPlaced {
                        identifier: command.identifier.to_owned(),
                        order_identifier: command.order_identifier.to_owned(),
                        line_items: command.line_items.to_owned(),
                        reason: Reason("Restaurant does not exist".to_string()),
                        r#final: false,
                    })]
                }
            }
        }),
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            RestaurantEvent::Created(event) => Some(Restaurant {
                identifier: event.identifier.to_owned(),
                name: event.name.to_owned(),
                menu: event.menu.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the Restaurant, for example.
            RestaurantEvent::NotCreated(..) => state.clone(),

            RestaurantEvent::MenuChanged(event) => state.clone().map(|s| Restaurant {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: event.menu.to_owned(),
            }),
            // On error event we choose NOT TO change the state of the Restaurant, for example.
            RestaurantEvent::MenuNotChanged(..) => state.clone(),

            RestaurantEvent::OrderPlaced(event) => state.clone().map(|s| Restaurant {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: s.menu,
            }),
            // On error event we choose NOT TO change the state of the Restaurant, for example.
            RestaurantEvent::OrderNotPlaced(..) => state.clone(),
        }),

        // The initial state of the decider
        initial_state: Box::new(|| None),
    }
}
