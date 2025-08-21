use crate::framework::domain::api::{DeciderType, EventType, Identifier, IsFinal};
use crate::framework::infrastructure::errors::ErrorMessage;
use pgrx::datum::DatumWithOid;
use pgrx::{JsonB, Spi, Uuid};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use uuid::Uuid as UUID;

/// Converts a `JsonB` to an event payload type.
fn to_event<E: DeserializeOwned>(jsonb: JsonB) -> Result<E, ErrorMessage> {
    let value = jsonb.0.clone();
    serde_json::from_value(value).map_err(|err| ErrorMessage {
        message: "Failed to deserialize event: ".to_string() + &err.to_string(),
    })
}
/// A trait for event repositories.
pub trait EventRepository<C, E>
where
    C: Identifier + DeciderType,
    E: Identifier + EventType + IsFinal + DeciderType + DeserializeOwned + Serialize,
{
    /// Fetches current events, based on the command.
    /// This method fetches the latest event for the given command. We can do this because every event carries the state of the aggregate/account
    fn fetch_events(&self, command: &C) -> Result<Vec<(E, UUID)>, ErrorMessage> {
        let query =
            "SELECT * FROM events WHERE decider_id = $1 AND decider = $2 ORDER BY events.offset";

        Spi::connect(|client| {
            let mut results = Vec::new();
            let tup_table = client
                .select(
                    query,
                    None,
                    &[
                        DatumWithOid::from(command.identifier().to_string()),
                        DatumWithOid::from(command.decider_type()),
                    ],
                )
                .map_err(|err| ErrorMessage {
                    message: "Failed to fetch events: ".to_string() + &err.to_string(),
                })?;
            for row in tup_table {
                let data = row["data"].value::<JsonB>().map_err(|err| ErrorMessage {
                    message: "Failed to fetch event data/payload (map `data` to `JsonB`): ".to_string() + &err.to_string(),
                })?
                .ok_or(ErrorMessage {
                    message: "Failed to fetch event data/payload (map `data` to `JsonB`): No data/payload found".to_string(),
                })?;
                let event_id = row["event_id"]
                    .value::<Uuid>()
                    .map_err(|err| ErrorMessage {
                        message: "Failed to fetch event id (map `event_id` to `Uuid`): "
                            .to_string()
                            + &err.to_string(),
                    })?
                    .ok_or(ErrorMessage {
                        message:
                            "Failed to fetch event id (map `data` to `JsonB`): No event id found"
                                .to_string(),
                    })?;

                results.push((to_event(data)?, UUID::from_bytes(*event_id.as_bytes())));
            }
            Ok(results)
        })
    }
    /// Saves events.
    fn save(
        &self,
        events: &[E],
        latest_version: &Option<UUID>,
    ) -> Result<Vec<(E, UUID)>, ErrorMessage> {
        let query = "
        INSERT INTO events (event, event_id, decider, decider_id, data, command_id, previous_id, final)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *";

        Spi::connect_mut(|client| {
            let mut results = Vec::new();
            let mut version = latest_version.to_owned();
            for event in events {
                let data = serde_json::to_value(event).map_err(|err| ErrorMessage {
                    message: "Failed to save event! Failed to serialize event data/payload: "
                        .to_string()
                        + &err.to_string(),
                })?;
                let event_id: UUID = UUID::new_v4();
                let tup_table = client
                    .update(
                        query,
                        None,
                        &[
                            DatumWithOid::from(event.event_type()),
                            DatumWithOid::from(Uuid::from_bytes(event_id.into_bytes())),
                            DatumWithOid::from(event.decider_type()),
                            DatumWithOid::from(Uuid::from_bytes(event.identifier().into_bytes())),
                            DatumWithOid::from(JsonB(data)),
                            DatumWithOid::from(Uuid::from_bytes(event_id.into_bytes())),
                            DatumWithOid::from(version.map(|v| Uuid::from_bytes(v.into_bytes()))),
                            DatumWithOid::from(event.is_final()),
                        ],
                    )
                    .map_err(|err| ErrorMessage {
                        message: "Failed to save event: ".to_string() + &err.to_string(),
                    })?;

                for row in tup_table {
                    let data = row["data"].value::<JsonB>().map_err(|err| ErrorMessage {
                        message: "Failed to save event data/payload (map `data` to `JsonB`): ".to_string() + &err.to_string(),
                    })?.ok_or(ErrorMessage {
                        message: "Failed to save event data/payload (map `data` to `JsonB`): No data/payload found".to_string(),
                    })?;
                    let event_id = row["event_id"]
                        .value::<Uuid>()
                        .map_err(|err| ErrorMessage {
                            message: "Failed to save event id (map `event_id` to `Uuid`): "
                                .to_string()
                                + &err.to_string(),
                        })?
                        .ok_or(ErrorMessage {
                            message:
                                "Failed to save event id (map `data` to `JsonB`): No event id found"
                                    .to_string(),
                        })?;

                    results.push((to_event(data)?, UUID::from_bytes(*event_id.as_bytes())));
                }
                version = Some(event_id);
            }
            Ok(results)
        })
    }
}

/// A trait for event orchestrating repositories.
pub trait EventOrchestratingRepository<C, E>
where
    C: Identifier + DeciderType,
    E: Clone
        + Identifier
        + EventType
        + IsFinal
        + DeciderType
        + DeserializeOwned
        + Serialize
        + Debug,
{
    /// Fetches current events, based on the command.
    /// This method fetches the latest event for the given command. We can do this because every event carries the state of the aggregate/account
    fn fetch_events(&self, command: &C) -> Result<Vec<(E, UUID)>, ErrorMessage> {
        let query =
            "SELECT * FROM events WHERE decider_id = $1 AND decider = $2 ORDER BY events.offset";

        Spi::connect_mut(|client| {
            let mut results = Vec::new();
            let tup_table = client
                .select(
                    query,
                    None,
                    &[
                        DatumWithOid::from(command.identifier().to_string()),
                        DatumWithOid::from(command.decider_type()),
                    ],
                )
                .map_err(|err| ErrorMessage {
                    message: "Failed to fetch events: ".to_string() + &err.to_string(),
                })?;
            for row in tup_table {
                let data = row["data"].value::<JsonB>().map_err(|err| ErrorMessage {
                    message: "Failed to fetch event data/payload (map `data` to `JsonB`): ".to_string() + &err.to_string(),
                })?.ok_or(ErrorMessage {
                    message: "Failed to fetch event data/payload (map `data` to `JsonB`): No data/payload found".to_string(),
                })?;
                let event_id = row["event_id"]
                    .value::<Uuid>()
                    .map_err(|err| ErrorMessage {
                        message: "Failed to fetch event id (map `event_id` to `Uuid`): "
                            .to_string()
                            + &err.to_string(),
                    })?
                    .ok_or(ErrorMessage {
                        message:
                            "Failed to fetch event id (map `data` to `JsonB`): No event id found"
                                .to_string(),
                    })?;
                results.push((to_event(data)?, UUID::from_bytes(*event_id.as_bytes())));
            }
            Ok(results)
        })
    }

    /// Fetches the latest version of the event stream to which the event belongs.
    fn fetch_latest_version(&self, event: &E) -> Result<Option<UUID>, ErrorMessage> {
        let query =
            "SELECT * FROM events WHERE decider_id = $1 AND decider = $2 ORDER BY events.offset DESC LIMIT 1";
        Spi::connect(|client| {
            let mut results = Vec::new();
            let tup_table = client
                .select(
                    query,
                    None,
                    &[
                        DatumWithOid::from(event.identifier().to_string()),
                        DatumWithOid::from(event.decider_type()),
                    ],
                )
                .map_err(|err| ErrorMessage {
                    message: "Failed to fetch latest event / version: ".to_string()
                        + &err.to_string(),
                })?;
            for row in tup_table {
                let event_id = row["event_id"]
                    .value::<Uuid>()
                    .map_err(|err| ErrorMessage {
                        message: "Failed to fetch latest event id (map `event_id` to `Uuid`): "
                            .to_string()
                            + &err.to_string(),
                    })?
                    .ok_or(ErrorMessage {
                        message:
                        "Failed to fetch latest event id (map `data` to `JsonB`): No event id found"
                            .to_string(),
                    })?;
                results.push(UUID::from_bytes(*event_id.as_bytes()));
            }
            Ok(results.first().cloned())
        })
    }
    /// Saves events.
    fn save(&self, events: &[E]) -> Result<Vec<(E, UUID)>, ErrorMessage> {
        let query = "
        INSERT INTO events (event, event_id, decider, decider_id, data, command_id, previous_id, final)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *";

        Spi::connect_mut(|client| {
            let mut results = Vec::new();
            for event in events {
                let data = serde_json::to_value(event).map_err(|err| ErrorMessage {
                    message: "Failed to save event! Failed to serialize event data/payload: "
                        .to_string()
                        + &err.to_string(),
                })?;
                let version = self.fetch_latest_version(event)?;
                let event_id: UUID = UUID::new_v4();
                let tup_table = client
                    .update(
                        query,
                        None,
                        &[
                            DatumWithOid::from(event.event_type()),
                            DatumWithOid::from(Uuid::from_bytes(event_id.into_bytes())),
                            DatumWithOid::from(event.decider_type()),
                            DatumWithOid::from(Uuid::from_bytes(event.identifier().into_bytes())),
                            DatumWithOid::from(JsonB(data)),
                            DatumWithOid::from(Uuid::from_bytes(event_id.into_bytes())),
                            DatumWithOid::from(version.map(|v| Uuid::from_bytes(v.into_bytes()))),
                            DatumWithOid::from(event.is_final()),
                        ],
                    )
                    .map_err(|err| ErrorMessage {
                        message: "Failed to save event: ".to_string() + &err.to_string(),
                    })?;

                for row in tup_table {
                    let data = row["data"].value::<JsonB>().map_err(|err| ErrorMessage {
                        message: "Failed to save event data/payload (map `data` to `JsonB`): ".to_string() + &err.to_string(),
                    })?.ok_or(ErrorMessage {
                        message: "Failed to save event data/payload (map `data` to `JsonB`): No data/payload found".to_string(),
                    })?;
                    let event_id = row["event_id"]
                        .value::<Uuid>()
                        .map_err(|err| ErrorMessage {
                            message: "Failed to save event id (map `event_id` to `Uuid`): "
                                .to_string()
                                + &err.to_string(),
                        })?
                        .ok_or(ErrorMessage {
                            message:
                                "Failed to save event id (map `data` to `JsonB`): No event id found"
                                    .to_string(),
                        })?;
                    results.push((to_event(data)?, UUID::from_bytes(*event_id.as_bytes())));
                }
            }
            Ok(results)
        })
    }
}
