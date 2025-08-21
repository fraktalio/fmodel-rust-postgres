use crate::domain::api::OrderEvent;
use crate::domain::order_view::OrderViewState;
use crate::framework::domain::api::Identifier;
use crate::framework::infrastructure::errors::ErrorMessage;
use crate::framework::infrastructure::to_payload;
use crate::framework::infrastructure::view_state_repository::ViewStateRepository;
use pgrx::{datum::DatumWithOid, JsonB, Spi, Uuid};

/// OrderViewStateRepository struct
/// View state repository is always very specific to the domain. There is no default implementation in the `ViewStateRepository` trait.
pub struct OrderViewStateRepository {}

/// OrderViewStateRepository - struct implementation
impl OrderViewStateRepository {
    /// Create a new OrderViewStateRepository
    pub fn new() -> Self {
        OrderViewStateRepository {}
    }
}

/// Implementation of the view state repository for the order `view` state.
impl ViewStateRepository<OrderEvent, Option<OrderViewState>> for OrderViewStateRepository {
    /// Fetches current state, based on the event.
    fn fetch_state(
        &self,
        event: &OrderEvent,
    ) -> Result<Option<Option<OrderViewState>>, ErrorMessage> {
        let query = "SELECT data FROM orders WHERE id = $1";
        Spi::connect(|client| {
            let mut results = Vec::new();
            let tup_table = client
                .select(
                    query,
                    None,
                    &[DatumWithOid::from(Uuid::from_bytes(
                        event.identifier().into_bytes(),
                    ))],
                )
                .map_err(|err| ErrorMessage {
                    message: "Failed to fetch the order: ".to_string() + &err.to_string(),
                })?;
            for row in tup_table {
                let data = row["data"].value::<JsonB>().map_err(|err| ErrorMessage {
                    message: "Failed to fetch the order/payload (map `data` to `JsonB`): ".to_string() + &err.to_string(),
                })?.ok_or(ErrorMessage {
                    message: "Failed to fetch order data/payload (map `data` to `JsonB`): No data/payload found".to_string(),
                })?;

                results.push(to_payload::<OrderViewState>(data)?);
            }
            Ok(Some(results.into_iter().last()))
        })
    }
    /// Saves the new state.
    fn save(&self, state: &Option<OrderViewState>) -> Result<Option<OrderViewState>, ErrorMessage> {
        let state = state.as_ref().ok_or(ErrorMessage {
            message: "Failed to save the order: state is empty".to_string(),
        })?;
        let data = serde_json::to_value(state).map_err(|err| ErrorMessage {
            message: "Failed to serialize the order: ".to_string() + &err.to_string(),
        })?;

        Spi::connect_mut(|client| {
            client
                .update(
                    "INSERT INTO orders (id, data) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET data = $2 RETURNING data",
                    None,
                    &[
                        DatumWithOid::from(Uuid::from_bytes(state.identifier.0.into_bytes())),
                        DatumWithOid::from(JsonB(data)),
                    ]
                )?
                .first()
                .get_one::<JsonB>().map(|o|{ o.map( |it| to_payload(it).unwrap() )})
        })
            .map(Some)
        .map_err(|err| ErrorMessage {
            message: "Failed to save the order: ".to_string() + &err.to_string(),
        })
            .map(|state| state.unwrap())
    }
}
