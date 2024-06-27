use crate::domain::api::RestaurantEvent;
use crate::domain::restaurant_view::RestaurantViewState;
use crate::framework::domain::api::Identifier;
use crate::framework::infrastructure::errors::ErrorMessage;
use crate::framework::infrastructure::to_payload;
use crate::framework::infrastructure::view_state_repository::ViewStateRepository;
use pgrx::{IntoDatum, JsonB, PgBuiltInOids, Spi};

/// RestaurantViewStateRepository struct
pub struct RestaurantViewStateRepository {}

/// RestaurantViewStateRepository - struct implementation
impl RestaurantViewStateRepository {
    /// Create a new RestaurantViewStateRepository
    pub fn new() -> Self {
        RestaurantViewStateRepository {}
    }
}

/// Implementation of the view state repository for the restaurant `view` state.
impl ViewStateRepository<RestaurantEvent, Option<RestaurantViewState>>
    for RestaurantViewStateRepository
{
    /// Fetches current state, based on the event.
    fn fetch_state(
        &self,
        event: &RestaurantEvent,
    ) -> Result<Option<Option<RestaurantViewState>>, ErrorMessage> {
        let query = "SELECT data FROM restaurants WHERE id = $1";
        Spi::connect(|client| {
            let mut results = Vec::new();
            let tup_table = client
                .select(
                    query,
                    None,
                    Some(vec![(
                        PgBuiltInOids::UUIDOID.oid(),
                        event.identifier().to_string().into_datum(),
                    )]),
                )
                .map_err(|err| ErrorMessage {
                    message: "Failed to fetch the restaurant: ".to_string() + &err.to_string(),
                })?;
            for row in tup_table {
                let data = row["data"].value::<JsonB>().map_err(|err| ErrorMessage {
                    message: "Failed to fetch the restaurant/payload (map `data` to `JsonB`): ".to_string() + &err.to_string(),
                })?.ok_or(ErrorMessage {
                    message: "Failed to fetch event data/payload (map `data` to `JsonB`): No data/payload found".to_string(),
                })?;

                results.push(to_payload::<RestaurantViewState>(data)?);
            }
            Ok(Some(results.into_iter().last()))
        })
    }
    /// Saves the new state.
    fn save(
        &self,
        state: &Option<RestaurantViewState>,
    ) -> Result<Option<RestaurantViewState>, ErrorMessage> {
        let state = state.as_ref().ok_or(ErrorMessage {
            message: "Failed to save the restaurant: state is empty".to_string(),
        })?;
        let data = serde_json::to_value(state).map_err(|err| ErrorMessage {
            message: "Failed to serialize the restaurant: ".to_string() + &err.to_string(),
        })?;

        Spi::connect(|mut client| {
            client
                .update(
                    "INSERT INTO restaurants (id, data) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET data = $2 RETURNING data",
                    None,
                    Some(vec![
                        (
                            PgBuiltInOids::UUIDOID.oid(),
                            state.identifier.to_string().into_datum(),
                        ),
                        (
                            PgBuiltInOids::JSONBOID.oid(),
                            JsonB(data).into_datum(),
                        ),
                    ]),
                )?
                .first()
                .get_one::<JsonB>().map(|o|{ o.map( |it| to_payload(it).unwrap() )})
        })
            .map(|state| Some(state))
        .map_err(|err| ErrorMessage {
            message: "Failed to save the restaurant: ".to_string() + &err.to_string(),
        })
            .map(|state| state.unwrap())
    }
}
