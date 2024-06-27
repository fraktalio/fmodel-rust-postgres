use crate::domain::api::RestaurantEvent;
use crate::domain::restaurant_view::{RestaurantView, RestaurantViewState};
use crate::framework::application::materialized_view::MaterializedView;
use crate::infrastructure::restaurant_view_state_repository::RestaurantViewStateRepository;

/// A convenient type alias for the restaurant materialized view.
pub type RestaurantMeterializedView<'a> = MaterializedView<
    Option<RestaurantViewState>,
    RestaurantEvent,
    RestaurantViewStateRepository,
    RestaurantView<'a>,
>;
