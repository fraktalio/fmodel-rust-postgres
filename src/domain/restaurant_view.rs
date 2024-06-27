use fmodel_rust::view::View;
use pgrx::PostgresType;
use serde::{Deserialize, Serialize};

use crate::domain::api::{RestaurantEvent, RestaurantId, RestaurantMenu, RestaurantName};

/// The state of the Restaurant View is represented by this struct. It belongs to the Domain layer.
#[derive(PostgresType, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RestaurantViewState {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
}

/// A convenient type alias for the Restaurant view
pub type RestaurantView<'a> = View<'a, Option<RestaurantViewState>, RestaurantEvent>;

/// View represents the event handling algorithm. It belongs to the Domain layer.
pub fn restaurant_view<'a>() -> RestaurantView<'a> {
    View {
        // Evolve the state based on the current state and the event
        // Exhaustive pattern matching on the event
        evolve: Box::new(|state, event| match event {
            RestaurantEvent::Created(event) => Some(RestaurantViewState {
                identifier: event.identifier.to_owned(),
                name: event.name.to_owned(),
                menu: event.menu.to_owned(),
            }),

            RestaurantEvent::MenuChanged(event) => state.clone().map(|s| RestaurantViewState {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: event.menu.to_owned(),
            }),

            RestaurantEvent::OrderPlaced(event) => state.clone().map(|s| RestaurantViewState {
                identifier: event.identifier.to_owned(),
                name: s.name,
                menu: s.menu,
            }),
        }),

        // The initial state of the decider
        initial_state: Box::new(|| None),
    }
}
