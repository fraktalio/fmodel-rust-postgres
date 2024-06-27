use crate::application::order_restaurant_aggregate::OrderAndRestaurantAggregate;
use crate::application::restaurant_materialized_view::RestaurantMeterializedView;
use crate::domain::restaurant_view::restaurant_view;
use crate::domain::{
    event_to_restaurant_event, order_restaurant_decider, order_restaurant_saga, Command, Event,
};
use crate::framework::infrastructure::errors::{ErrorMessage, TriggerError};
use crate::framework::infrastructure::to_payload;
use crate::infrastructure::order_restaurant_event_repository::OrderAndRestaurantEventRepository;
use crate::infrastructure::restaurant_view_state_repository::RestaurantViewStateRepository;
use pgrx::prelude::*;
use pgrx::JsonB;

mod application;
mod domain;
mod framework;
mod infrastructure;

pg_module_magic!();

// Declare SQL (from a file) to be included in generated extension script.
// Defines the `event_sourcing` table(s) and indexes.
extension_sql_file!(
    "../sql/event_sourcing.sql",
    name = "event_sourcing",
    bootstrap // Communicates that this is SQL intended to go before all other generated SQL.
);

/// Command handler for the whole domain / orders and restaurants combined.
/// It handles a single command and returns a list of events that were generated and persisted.
#[pg_extern]
fn handle(command: Command) -> Result<Vec<Event>, ErrorMessage> {
    let repository = OrderAndRestaurantEventRepository::new();
    let aggregate = OrderAndRestaurantAggregate::new(
        repository,
        order_restaurant_decider(),
        order_restaurant_saga(),
    );
    aggregate
        .handle(&command)
        .map(|res| res.into_iter().map(|(e, _)| e.clone()).collect())
}

/// Compound command handler for the domain / orders and restaurants combined
/// It handles a list of commands and returns a list of events that were generated and persisted.
/// All commands are executed in a single transaction, and the effects/events of the previous commands are visible to the subsequent commands.
/// If any of the commands fail, the transaction is rolled back, and no events are persisted.
/// This is useful when you need to ensure that all commands are executed or none.
#[pg_extern]
fn handle_all(commands: Vec<Command>) -> Result<Vec<Event>, ErrorMessage> {
    let repository = OrderAndRestaurantEventRepository::new();
    let aggregate = OrderAndRestaurantAggregate::new(
        repository,
        order_restaurant_decider(),
        order_restaurant_saga(),
    );
    aggregate
        .handle_all(&commands)
        .map(|res| res.into_iter().map(|(e, _)| e.clone()).collect())
}

/// Event handler for Restaurant events / Trigger function that handles events and updates the materialized view.
#[pg_trigger]
fn handle_restaurant_events<'a>(
    trigger: &'a PgTrigger<'a>,
) -> Result<Option<PgHeapTuple<'a, impl WhoAllocated>>, TriggerError> {
    let new = trigger
        .new()
        .ok_or(TriggerError::NullTriggerTuple)?
        .into_owned();
    let event: JsonB = new
        .get_by_name::<JsonB>("data")?
        .ok_or(TriggerError::NullTriggerTuple)?;
    let materialized_view =
        RestaurantMeterializedView::new(RestaurantViewStateRepository::new(), restaurant_view());

    match event_to_restaurant_event(
        &to_payload::<Event>(event)
            .map_err(|err| TriggerError::EventHandlingError(err.to_string()))?,
    ) {
        // If the event is not a Restaurant event, we do nothing
        None => return Ok(Some(new)),
        // If the event is a Restaurant event, we handle it
        Some(e) => {
            materialized_view
                .handle(&e)
                .map_err(|err| TriggerError::EventHandlingError(err.message))?;
        }
    }
    Ok(Some(new))
}

// Materialized view / Table for the Restaurant query side model
// This table is updated by the trigger function / event handler `handle_restaurant_events`
extension_sql!(
    r#"
    CREATE TABLE IF NOT EXISTS restaurants (
                                           id UUID PRIMARY KEY,
                                           data JSONB
    );

    CREATE TRIGGER restaurant_event_handler_trigger AFTER INSERT ON events FOR EACH ROW EXECUTE PROCEDURE handle_restaurant_events();
    "#,
    name = "restaurant_event_handler_trigger",
    requires = [handle_restaurant_events]
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    // Test data: RestaurantCreated
    extension_sql!(
        r#"
    INSERT INTO events (event, event_id, decider, decider_id, data, command_id, previous_id, final)
    VALUES ('RestaurantCreated', '5f8bdf95-c95b-4e4b-8535-d2ac4663bea9', 'Restaurant', 'e48d4d9e-403e-453f-b1ba-328e0ce23737', '{"type": "RestaurantCreated","identifier": "e48d4d9e-403e-453f-b1ba-328e0ce23737", "name": "Pljeska", "menu": {"menu_id": "02f09a3f-1624-3b1d-8409-44eff7708210", "items": [{"id": "02f09a3f-1624-3b1d-8409-44eff7708210","name": "supa","price": 10},{"id": "02f09a3f-1624-3b1d-8409-44eff7708210","name": "sarma","price": 20 }],"cuisine": "Vietnamese"}, "final": false }', 'e48d4d9e-403e-453f-b1ba-328e0ce23737', NULL, FALSE);
    "#,
        name = "data_insert",
        requires = ["restaurant_event_handler_trigger"]
    );
    use crate::domain::api::{
        ChangeRestaurantMenu, CreateRestaurant, OrderCreated, OrderLineItem, OrderPlaced,
        PlaceOrder, RestaurantCreated, RestaurantMenuChanged,
    };
    use crate::domain::api::{
        MenuId, MenuItem, MenuItemId, MenuItemName, Money, OrderId, OrderLineItemId,
        OrderLineItemQuantity, OrderStatus, RestaurantId, RestaurantMenu, RestaurantMenuCuisine,
        RestaurantName,
    };
    use crate::domain::{Command, Event};
    use pgrx::prelude::*;
    use uuid::Uuid;

    #[pg_test]
    fn create_restaurant_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let restaurant_name = RestaurantName("Test Restaurant".to_string());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_id = MenuId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_items = vec![MenuItem {
            id: menu_item_id,
            name: MenuItemName("Item 1".to_string()),
            price: Money(100u64),
        }];

        let create_restaurant_command = Command::CreateRestaurant(CreateRestaurant {
            identifier: restaurant_identifier.clone(),
            name: restaurant_name.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });

        let restaurant_created_event = Event::RestaurantCreated(RestaurantCreated {
            identifier: restaurant_identifier.clone(),
            name: restaurant_name.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
            r#final: false,
        });

        assert_eq!(
            Some(restaurant_created_event.clone()),
            crate::handle(create_restaurant_command)
                .unwrap()
                .into_iter()
                .next()
        );
    }

    #[pg_test(error = "Failed to create the Restaurant. Restaurant already exists!")]
    fn create_restaurant_error_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("e48d4d9e-403e-453f-b1ba-328e0ce23737").unwrap());
        let restaurant_name = RestaurantName("Test Restaurant".to_string());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_id = MenuId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_items = vec![MenuItem {
            id: menu_item_id,
            name: MenuItemName("Item 1".to_string()),
            price: Money(100u64),
        }];

        let create_restaurant_command = Command::CreateRestaurant(CreateRestaurant {
            identifier: restaurant_identifier.clone(),
            name: restaurant_name.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });

        let _ = crate::handle(create_restaurant_command);
    }

    #[pg_test]
    fn change_menu_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("e48d4d9e-403e-453f-b1ba-328e0ce23737").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_id = MenuId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_items = vec![MenuItem {
            id: menu_item_id,
            name: MenuItemName("Item 1".to_string()),
            price: Money(100u64),
        }];

        let change_restaurant_menu = Command::ChangeRestaurantMenu(ChangeRestaurantMenu {
            identifier: restaurant_identifier.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });

        let restaurant_menu_changed_event = Event::RestaurantMenuChanged(RestaurantMenuChanged {
            identifier: restaurant_identifier.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
            r#final: false,
        });

        assert_eq!(
            Some(restaurant_menu_changed_event.clone()),
            crate::handle(change_restaurant_menu)
                .unwrap()
                .into_iter()
                .next()
        );
    }

    #[pg_test(error = "Failed to change the menu. Restaurant does not exist!")]
    fn change_menu_error_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_id = MenuId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_items = vec![MenuItem {
            id: menu_item_id,
            name: MenuItemName("Item 1".to_string()),
            price: Money(100u64),
        }];

        let change_restaurant_menu = Command::ChangeRestaurantMenu(ChangeRestaurantMenu {
            identifier: restaurant_identifier.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });

        let _ = crate::handle(change_restaurant_menu);
    }

    #[pg_test]
    fn place_order_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("e48d4d9e-403e-453f-b1ba-328e0ce23737").unwrap());
        let order_identifier =
            OrderId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let line_items = vec![OrderLineItem {
            id: OrderLineItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap()),
            quantity: OrderLineItemQuantity(1),
            menu_item_id: menu_item_id.clone(),
            name: MenuItemName("Item 1".to_string()),
        }];

        let place_order = Command::PlaceOrder(PlaceOrder {
            identifier: restaurant_identifier.clone(),
            order_identifier: order_identifier.clone(),
            line_items: line_items.clone(),
        });

        let order_placed_event = Event::OrderPlaced(OrderPlaced {
            identifier: restaurant_identifier.clone(),
            order_identifier: order_identifier.clone(),
            line_items: line_items.clone(),
            r#final: false,
        });

        let order_created_event = Event::OrderCreated(OrderCreated {
            identifier: order_identifier.clone(),
            restaurant_identifier: restaurant_identifier.clone(),
            status: OrderStatus::Created,
            line_items: line_items.clone(),
            r#final: false,
        });

        let mut result = crate::handle(place_order).unwrap().into_iter();
        assert_eq!(Some(order_placed_event), result.next(),);
        assert_eq!(Some(order_created_event), result.next(),);
    }

    #[pg_test(error = "Failed to place the order. Restaurant does not exist!")]
    fn place_order_error_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let order_identifier =
            OrderId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let line_items = vec![OrderLineItem {
            id: OrderLineItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap()),
            quantity: OrderLineItemQuantity(1),
            menu_item_id: menu_item_id.clone(),
            name: MenuItemName("Item 1".to_string()),
        }];

        let place_order = Command::PlaceOrder(PlaceOrder {
            identifier: restaurant_identifier.clone(),
            order_identifier: order_identifier.clone(),
            line_items: line_items.clone(),
        });

        let _ = crate::handle(place_order);
    }

    #[pg_test]
    fn create_restaurant_and_place_order_test() {
        let restaurant_identifier =
            RestaurantId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708208").unwrap());
        let order_identifier =
            OrderId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let restaurant_name = RestaurantName("Test Restaurant".to_string());
        let menu_item_id =
            MenuItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_id = MenuId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap());
        let menu_items = vec![MenuItem {
            id: menu_item_id.clone(),
            name: MenuItemName("Item 1".to_string()),
            price: Money(100u64),
        }];
        let line_items = vec![OrderLineItem {
            id: OrderLineItemId(Uuid::parse_str("02f09a3f-1624-3b1d-8409-44eff7708210").unwrap()),
            quantity: OrderLineItemQuantity(1),
            menu_item_id: menu_item_id.clone(),
            name: MenuItemName("Item 1".to_string()),
        }];

        let create_restaurant_command = Command::CreateRestaurant(CreateRestaurant {
            identifier: restaurant_identifier.clone(),
            name: restaurant_name.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
        });

        let place_order = Command::PlaceOrder(PlaceOrder {
            identifier: restaurant_identifier.clone(),
            order_identifier: order_identifier.clone(),
            line_items: line_items.clone(),
        });

        let restaurant_created_event = Event::RestaurantCreated(RestaurantCreated {
            identifier: restaurant_identifier.clone(),
            name: restaurant_name.clone(),
            menu: RestaurantMenu {
                menu_id: menu_id.clone(),
                items: menu_items.clone(),
                cuisine: RestaurantMenuCuisine::Vietnamese,
            },
            r#final: false,
        });

        let order_placed_event = Event::OrderPlaced(OrderPlaced {
            identifier: restaurant_identifier.clone(),
            order_identifier: order_identifier.clone(),
            line_items: line_items.clone(),
            r#final: false,
        });

        let order_created_event = Event::OrderCreated(OrderCreated {
            identifier: order_identifier.clone(),
            restaurant_identifier: restaurant_identifier.clone(),
            status: OrderStatus::Created,
            line_items: line_items.clone(),
            r#final: false,
        });

        let mut result = crate::handle_all(vec![create_restaurant_command, place_order])
            .unwrap()
            .into_iter();
        assert_eq!(Some(restaurant_created_event), result.next(),);
        assert_eq!(Some(order_placed_event), result.next(),);
        assert_eq!(Some(order_created_event), result.next(),);
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
