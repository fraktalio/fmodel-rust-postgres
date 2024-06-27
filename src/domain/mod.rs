use crate::domain::api::{
    ChangeRestaurantMenu, CreateOrder, CreateRestaurant, MarkOrderAsPrepared, OrderCommand,
    PlaceOrder, RestaurantCommand,
};
use crate::domain::order_decider::{order_decider, Order};
use crate::domain::order_saga::order_saga;
use crate::domain::restaurant_decider::{restaurant_decider, Restaurant};
use crate::domain::restaurant_saga::restaurant_saga;
use crate::framework::domain::api::{DeciderType, EventType, Identifier, IsFinal};
use api::{
    OrderCreated, OrderEvent, OrderPlaced, OrderPrepared, RestaurantCreated, RestaurantEvent,
    RestaurantMenuChanged,
};
use fmodel_rust::decider::Decider;
use fmodel_rust::saga::Saga;
use fmodel_rust::Sum;
use pgrx::PostgresType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod api;
pub mod order_decider;
pub mod order_saga;
pub mod order_view;
pub mod restaurant_decider;
pub mod restaurant_saga;
pub mod restaurant_view;

/// A convenient type alias for the combined Decider
/// This decider is used to combine the Restaurant and Order deciders into a single decider that can handle both Restaurant and Order commands.
pub type OrderAndRestaurantDecider<'a> =
    Decider<'a, Command, (Option<Restaurant>, Option<Order>), Event>;

/// A convenient type alias for the combined Saga
/// This saga is used to combine the Restaurant and Order choreography sagas into a single orchestrating saga that can handle both Restaurant and Order events, and produce Restaurant and Order commands as a result.
pub type OrderAndRestaurantSaga<'a> = Saga<'a, Event, Command>;

/// Combined Decider, combining the Restaurant and Order deciders into a single decider that can handle both Restaurant and Order commands.
pub fn order_restaurant_decider<'a>() -> OrderAndRestaurantDecider<'a> {
    restaurant_decider()
        .combine(order_decider())
        .map_command(&command_to_sum)
        .map_event(&event_to_sum, &sum_to_event)
}

/// Combined Saga, combining the Restaurant and Order choreography sagas into a single orchestrating saga that can handle both Restaurant and Order events, and produce Restaurant and Order commands as a result.
pub fn order_restaurant_saga<'a>() -> OrderAndRestaurantSaga<'a> {
    restaurant_saga()
        .combine(order_saga())
        .map_action_result(&event_to_sum2)
        .map_action(&sum_to_command)
}

/// All possible commands in the order&restaurant domains
#[derive(PostgresType, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum Command {
    CreateRestaurant(CreateRestaurant),
    ChangeRestaurantMenu(ChangeRestaurantMenu),
    PlaceOrder(PlaceOrder),
    CreateOrder(CreateOrder),
    MarkOrderAsPrepared(MarkOrderAsPrepared),
}

/// Implement the Identifier trait for the Command enum
impl Identifier for Command {
    fn identifier(&self) -> Uuid {
        match self {
            Command::CreateRestaurant(cmd) => cmd.identifier.0,
            Command::ChangeRestaurantMenu(cmd) => cmd.identifier.0,
            Command::PlaceOrder(cmd) => cmd.identifier.0,
            Command::CreateOrder(cmd) => cmd.identifier.0,
            Command::MarkOrderAsPrepared(cmd) => cmd.identifier.0,
        }
    }
}

/// All possible events in the order&restaurant domains
#[derive(PostgresType, Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Event {
    RestaurantCreated(RestaurantCreated),
    RestaurantMenuChanged(RestaurantMenuChanged),
    OrderPlaced(OrderPlaced),
    OrderCreated(OrderCreated),
    OrderPrepared(OrderPrepared),
}

/// Implement the Identifier trait for the Event enum
impl Identifier for Event {
    fn identifier(&self) -> Uuid {
        match self {
            Event::RestaurantCreated(evt) => evt.identifier.0,
            Event::RestaurantMenuChanged(evt) => evt.identifier.0,
            Event::OrderPlaced(evt) => evt.identifier.0,
            Event::OrderCreated(evt) => evt.identifier.0,
            Event::OrderPrepared(evt) => evt.identifier.0,
        }
    }
}

/// Implement the EventType trait for the Event enum
impl EventType for Event {
    fn event_type(&self) -> String {
        match self {
            Event::RestaurantCreated(_) => "RestaurantCreated".to_string(),
            Event::RestaurantMenuChanged(_) => "RestaurantMenuChanged".to_string(),
            Event::OrderPlaced(_) => "OrderPlaced".to_string(),
            Event::OrderCreated(_) => "OrderCreated".to_string(),
            Event::OrderPrepared(_) => "OrderPrepared".to_string(),
        }
    }
}

/// Implement the IsFinal trait for the Event enum
impl IsFinal for Event {
    fn is_final(&self) -> bool {
        match self {
            Event::RestaurantCreated(evt) => evt.r#final,
            Event::RestaurantMenuChanged(evt) => evt.r#final,
            Event::OrderPlaced(evt) => evt.r#final,
            Event::OrderCreated(evt) => evt.r#final,
            Event::OrderPrepared(evt) => evt.r#final,
        }
    }
}

/// Implement the DeciderType trait for the Event enum
impl DeciderType for Event {
    fn decider_type(&self) -> String {
        match self {
            Event::RestaurantCreated(_) => "Restaurant".to_string(),
            Event::RestaurantMenuChanged(_) => "Restaurant".to_string(),
            Event::OrderPlaced(_) => "Restaurant".to_string(),
            Event::OrderCreated(_) => "Order".to_string(),
            Event::OrderPrepared(_) => "Order".to_string(),
        }
    }
}

/// Mapper functions to convert between the `FModel` Sum type and the more appropriate domain specific Command/API type
/// This is necessary because the `FModel` Sum type is used to combine the Restaurant and Order deciders into a single decider that can handle both Restaurant and Order commands.
/// We don't want to expose the `FModel` Sum type to the API, so we need to convert between the `FModel` Sum type and the more appropriate Command/API type.
pub fn command_to_sum(command: &Command) -> Sum<RestaurantCommand, OrderCommand> {
    match command {
        Command::CreateRestaurant(c) => {
            Sum::First(RestaurantCommand::CreateRestaurant(c.to_owned()))
        }
        Command::ChangeRestaurantMenu(c) => Sum::First(RestaurantCommand::ChangeMenu(c.to_owned())),
        Command::PlaceOrder(c) => Sum::First(RestaurantCommand::PlaceOrder(c.to_owned())),
        Command::CreateOrder(c) => Sum::Second(OrderCommand::Create(c.to_owned())),
        Command::MarkOrderAsPrepared(c) => Sum::Second(OrderCommand::MarkAsPrepared(c.to_owned())),
    }
}

pub fn event_to_sum(event: &Event) -> Sum<RestaurantEvent, OrderEvent> {
    match event {
        Event::RestaurantCreated(e) => Sum::First(RestaurantEvent::Created(e.to_owned())),
        Event::RestaurantMenuChanged(e) => Sum::First(RestaurantEvent::MenuChanged(e.to_owned())),
        Event::OrderPlaced(e) => Sum::First(RestaurantEvent::OrderPlaced(e.to_owned())),
        Event::OrderCreated(e) => Sum::Second(OrderEvent::Created(e.to_owned())),
        Event::OrderPrepared(e) => Sum::Second(OrderEvent::Prepared(e.to_owned())),
    }
}

pub fn event_to_sum2(event: &Event) -> Sum<OrderEvent, RestaurantEvent> {
    match event {
        Event::RestaurantCreated(e) => Sum::Second(RestaurantEvent::Created(e.to_owned())),
        Event::RestaurantMenuChanged(e) => Sum::Second(RestaurantEvent::MenuChanged(e.to_owned())),
        Event::OrderPlaced(e) => Sum::Second(RestaurantEvent::OrderPlaced(e.to_owned())),
        Event::OrderCreated(e) => Sum::First(OrderEvent::Created(e.to_owned())),
        Event::OrderPrepared(e) => Sum::First(OrderEvent::Prepared(e.to_owned())),
    }
}

pub fn sum_to_command(command: &Sum<OrderCommand, RestaurantCommand>) -> Command {
    match command {
        Sum::Second(c) => match c {
            RestaurantCommand::CreateRestaurant(c) => Command::CreateRestaurant(c.to_owned()),
            RestaurantCommand::ChangeMenu(c) => Command::ChangeRestaurantMenu(c.to_owned()),
            RestaurantCommand::PlaceOrder(c) => Command::PlaceOrder(c.to_owned()),
        },
        Sum::First(c) => match c {
            OrderCommand::Create(c) => Command::CreateOrder(c.to_owned()),
            OrderCommand::MarkAsPrepared(c) => Command::MarkOrderAsPrepared(c.to_owned()),
        },
    }
}

pub fn sum_to_event(event: &Sum<RestaurantEvent, OrderEvent>) -> Event {
    match event {
        Sum::First(e) => match e {
            RestaurantEvent::Created(e) => Event::RestaurantCreated(e.to_owned()),
            RestaurantEvent::MenuChanged(e) => Event::RestaurantMenuChanged(e.to_owned()),
            RestaurantEvent::OrderPlaced(e) => Event::OrderPlaced(e.to_owned()),
        },
        Sum::Second(e) => match e {
            OrderEvent::Created(e) => Event::OrderCreated(e.to_owned()),
            OrderEvent::Prepared(e) => Event::OrderPrepared(e.to_owned()),
        },
    }
}

pub fn event_to_restaurant_event(event: &Event) -> Option<RestaurantEvent> {
    match event {
        Event::RestaurantCreated(e) => Some(RestaurantEvent::Created(e.to_owned())),
        Event::RestaurantMenuChanged(e) => Some(RestaurantEvent::MenuChanged(e.to_owned())),
        Event::OrderPlaced(e) => Some(RestaurantEvent::OrderPlaced(e.to_owned())),
        Event::OrderCreated(_e) => None,
        Event::OrderPrepared(_e) => None,
    }
}
