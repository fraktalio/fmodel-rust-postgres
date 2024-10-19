use crate::framework::domain::api::Identifier;
use pgrx::{PostgresEnum, PostgresType};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use pgrx::FromDatum;

// ########################################################
// #################### Value Objects #####################
// ########################################################

// The 'newtype' pattern is typical in functional programming. In Haskell, this pattern is supported via the 'newtype' declaration, which allows the programmer to define a new type identical to an existing one except for its name. This is useful for creating type-safe abstractions, enabling the programmer to enforce stronger type constraints on using specific values.
// Similarly, in Rust, the 'newtype' idiom brings compile-time guarantees that the correct value type is supplied. The 'newtype' is a struct that wraps a single value and provides a new type for that value. A 'newtype' is the same as the underlying type at runtime, so it will not introduce any performance overhead.
#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct RestaurantId(pub Uuid);
impl fmt::Display for RestaurantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct RestaurantName(pub String);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OrderId(pub Uuid);
impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate the formatting to the inner Uuid
        write!(f, "{}", self.0)
    }
}

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Reason(pub String);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Money(pub u64);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MenuId(pub Uuid);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MenuItemId(pub Uuid);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MenuItemName(pub String);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OrderLineItemId(pub Uuid);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OrderLineItemQuantity(pub u32);

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MenuItem {
    pub id: MenuItemId,
    pub name: MenuItemName,
    pub price: Money,
}

#[derive(PostgresEnum, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum RestaurantMenuCuisine {
    Italian,
    Indian,
    Chinese,
    Japanese,
    American,
    Mexican,
    French,
    Thai,
    Vietnamese,
    Greek,
    Korean,
    Spanish,
    Lebanese,
    Turkish,
    Ethiopian,
    Moroccan,
    Egyptian,
    Brazilian,
    Polish,
    German,
    British,
    Irish,
    Other,
}

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct RestaurantMenu {
    pub menu_id: MenuId,
    pub items: Vec<MenuItem>,
    pub cuisine: RestaurantMenuCuisine,
}

#[derive(PostgresType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OrderLineItem {
    pub id: OrderLineItemId,
    pub quantity: OrderLineItemQuantity,
    pub menu_item_id: MenuItemId,
    pub name: MenuItemName,
}

#[derive(PostgresEnum, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum OrderStatus {
    Created,
    Prepared,
    Cancelled,
    Rejected,
}

// ########################################################
// ####################### COMMANDS #######################
// ########################################################

// #### RESTAURANT ####
/// All possible command variants that could be sent to a restaurant
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum RestaurantCommand {
    CreateRestaurant(CreateRestaurant),
    ChangeMenu(ChangeRestaurantMenu),
    PlaceOrder(PlaceOrder),
}
/// Intent/Command to create a new restaurant
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CreateRestaurant {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
}

/// Intent/Command to change the menu of a restaurant
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ChangeRestaurantMenu {
    pub identifier: RestaurantId,
    pub menu: RestaurantMenu,
}

/// Intent/Command to place an order at a restaurant
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PlaceOrder {
    pub identifier: RestaurantId,
    pub order_identifier: OrderId,
    pub line_items: Vec<OrderLineItem>,
}

// #### ORDER ####

/// All possible command variants that could be sent to an order
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum OrderCommand {
    Create(CreateOrder),
    MarkAsPrepared(MarkOrderAsPrepared),
}

/// Intent/Command to create a new order
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CreateOrder {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub line_items: Vec<OrderLineItem>,
}

/// Intent/Command to mark an order as prepared
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct MarkOrderAsPrepared {
    pub identifier: OrderId,
}

// ########################################################
// ######################## EVENTS ########################
// ########################################################

// #### RESTAURANT ####

/// All possible event variants that could be used to update a restaurant
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum RestaurantEvent {
    Created(RestaurantCreated),
    MenuChanged(RestaurantMenuChanged),
    OrderPlaced(OrderPlaced),
}

impl Identifier for RestaurantEvent {
    fn identifier(&self) -> Uuid {
        match self {
            RestaurantEvent::Created(e) => e.identifier.0,
            RestaurantEvent::MenuChanged(e) => e.identifier.0,
            RestaurantEvent::OrderPlaced(e) => e.identifier.0,
        }
    }
}

/// Fact/Event that a restaurant was created
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct RestaurantCreated {
    pub identifier: RestaurantId,
    pub name: RestaurantName,
    pub menu: RestaurantMenu,
    pub r#final: bool,
}

/// Fact/Event that a restaurant's menu was changed
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct RestaurantMenuChanged {
    pub identifier: RestaurantId,
    pub menu: RestaurantMenu,
    pub r#final: bool,
}

/// Fact/Event that an order was placed
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct OrderPlaced {
    pub identifier: RestaurantId,
    pub order_identifier: OrderId,
    pub line_items: Vec<OrderLineItem>,
    pub r#final: bool,
}

// #### ORDER ####

/// All possible event variants that could be used to update an order
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum OrderEvent {
    Created(OrderCreated),
    Prepared(OrderPrepared),
}

impl Identifier for OrderEvent {
    fn identifier(&self) -> Uuid {
        match self {
            OrderEvent::Created(e) => e.identifier.0,
            OrderEvent::Prepared(e) => e.identifier.0,
        }
    }
}

/// Fact/Event that an order was created
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct OrderCreated {
    pub identifier: OrderId,
    pub restaurant_identifier: RestaurantId,
    pub status: OrderStatus,
    pub line_items: Vec<OrderLineItem>,
    pub r#final: bool,
}

/// Fact/Event that an order was prepared
#[derive(PostgresType, Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
pub struct OrderPrepared {
    pub identifier: OrderId,
    pub status: OrderStatus,
    pub r#final: bool,
}
