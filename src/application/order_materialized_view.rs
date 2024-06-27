use crate::domain::api::OrderEvent;
use crate::domain::order_view::{OrderView, OrderViewState};
use crate::framework::application::materialized_view::MaterializedView;
use crate::infrastructure::order_view_state_repository::OrderViewStateRepository;

/// A convenient type alias for the order materialized view.
pub type OrderMeterializedView<'a> =
    MaterializedView<Option<OrderViewState>, OrderEvent, OrderViewStateRepository, OrderView<'a>>;
