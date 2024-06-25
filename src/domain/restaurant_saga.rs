use fmodel_rust::saga::Saga;

use crate::domain::api::{OrderEvent, RestaurantCommand};

/// A convenient type alias for the Restaurant choreography saga
type RestaurantSaga<'a> = Saga<'a, OrderEvent, RestaurantCommand>;

/// The Restaurant choreography saga - represents the central point of control deciding what to execute next.
/// It is a function that takes an event and returns a list of commands.
/// This Saga is not doing much ;)
pub fn restaurant_saga<'a>() -> RestaurantSaga<'a> {
    Saga {
        react: Box::new(|_event| match _event {
            OrderEvent::Created(..) => {
                vec![]
            }
            OrderEvent::Prepared(..) => {
                vec![]
            }
        }),
    }
}
