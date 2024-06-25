use crate::application::order_restaurant_aggregate::OrderAndRestaurantAggregate;
use crate::domain::{order_restaurant_decider, order_restaurant_saga, Command, Event};
use crate::framework::infrastructure::errors::ErrorMessage;
use crate::infrastructure::order_restaurant_event_repository::OrderAndRestaurantEventRepository;
use pgrx::prelude::*;

mod application;
mod domain;
mod framework;
mod infrastructure;

pg_module_magic!();

extension_sql_file!(
    "../sql/event_sourcing.sql",
    name = "event_sourcing",
    bootstrap
);

/// Command handler for the whole domain / both, orders and restaurants included.
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

/// Compound command handler for the domain / both, orders and restaurants included
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

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::domain::api::{CreateRestaurant, RestaurantCreated};
    use crate::domain::api::{
        MenuId, MenuItem, MenuItemId, MenuItemName, Money, RestaurantId, RestaurantMenu,
        RestaurantMenuCuisine, RestaurantName,
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
