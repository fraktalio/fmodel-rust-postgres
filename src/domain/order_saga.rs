use fmodel_rust::saga::Saga;

use crate::domain::{
    api::{CreateOrder, OrderCommand},
    Event,
};

/// A convenient type alias for the Order choreography saga
type OrderSaga<'a> = Saga<'a, Event, OrderCommand>;

/// The Order choreography saga - represents the central point of control deciding what to execute next.
/// It is a function that takes an event and returns a list of commands.
pub fn order_saga<'a>() -> OrderSaga<'a> {
    Saga {
        react: Box::new(|event| match event {
            Event::RestaurantCreated(..) => {
                vec![]
            }
            Event::OrderPrepared(..) => {
                vec![]
            }
            Event::OrderPlaced(event) => {
                vec![OrderCommand::Create(CreateOrder {
                    identifier: event.order_identifier.to_owned(),
                    restaurant_identifier: event.identifier.to_owned(),
                    line_items: event.line_items.to_owned(),
                })]
            }
            Event::RestaurantMenuChanged(..) => {
                vec![]
            }
            Event::OrderCreated(..) => {
                vec![]
            }
        }),
    }
}
