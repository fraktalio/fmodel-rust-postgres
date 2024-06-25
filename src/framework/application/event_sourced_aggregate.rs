// ###################################################################
// ###################### Regular Aggregate ##########################
// ###################################################################

use crate::framework::domain::api::{DeciderType, EventType, Identifier, IsFinal};
use crate::framework::infrastructure::errors::ErrorMessage;
use crate::framework::infrastructure::event_repository::{
    EventOrchestratingRepository, EventRepository,
};
use fmodel_rust::decider::{Decider, EventComputation};
use fmodel_rust::saga::Saga;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;
use uuid::Uuid;

/// Event sourced aggregate is composed of a repository and a decider.
/// The repository is responsible for fetching and saving events, and it is `sync`, not `async`.
#[allow(dead_code)]
pub struct EventSourcedAggregate<C, S, E, Repository, Decider>
where
    Repository: EventRepository<C, E>,
    Decider: EventComputation<C, S, E>,
    C: Identifier,
    E: EventType + Identifier + IsFinal + DeciderType + DeserializeOwned + Serialize,
{
    repository: Repository,
    decider: Decider,
    _marker: PhantomData<(C, S, E)>,
}

/// Implementation of the event computation for the event sourced aggregate.
impl<C, S, E, Repository, Decider> EventComputation<C, S, E>
    for EventSourcedAggregate<C, S, E, Repository, Decider>
where
    Repository: EventRepository<C, E>,
    Decider: EventComputation<C, S, E>,
    C: Identifier,
    E: EventType + Identifier + IsFinal + DeciderType + DeserializeOwned + Serialize,
{
    /// Computes new events based on the current events and the command.
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Vec<E> {
        self.decider.compute_new_events(current_events, command)
    }
}

impl<C, S, E, Repository, Decider> EventSourcedAggregate<C, S, E, Repository, Decider>
where
    Repository: EventRepository<C, E>,
    Decider: EventComputation<C, S, E>,
    C: Identifier,
    E: EventType + Identifier + IsFinal + DeciderType + DeserializeOwned + Serialize,
{
    /// Creates a new event sourced aggregate.
    #[allow(dead_code)]
    pub fn new(repository: Repository, decider: Decider) -> Self {
        EventSourcedAggregate {
            repository,
            decider,
            _marker: PhantomData,
        }
    }
    /// Handles the command and returns the new events.
    #[allow(dead_code)]
    pub fn handle(&self, command: &C) -> Result<Vec<(E, Uuid)>, ErrorMessage> {
        let events: Vec<(E, Uuid)> = self.repository.fetch_events(command)?;
        let mut version: Option<Uuid> = None;
        let mut current_events: Vec<E> = vec![];
        for (event, ver) in events {
            version = Some(ver);
            current_events.push(event);
        }
        let new_events = self.decider.compute_new_events(&current_events, command);
        self.repository.save(&new_events, &version)
    }
}

// ###################################################################
// ################### Orchestrating Aggregate #######################
// ###################################################################

/// Event sourced orchestrating aggregate is composed of a repository, a decider, and a saga.
/// The repository is responsible for fetching and saving events, and it is `sync`, not `async`.
pub struct EventSourcedOrchestratingAggregate<'a, C, S, E, Repository>
where
    Repository: EventOrchestratingRepository<C, E>,
    C: Identifier,
    E: Clone
        + EventType
        + Identifier
        + IsFinal
        + DeciderType
        + DeserializeOwned
        + Serialize
        + Debug,
{
    repository: Repository,
    decider: Decider<'a, C, S, E>,
    saga: Saga<'a, E, C>,
    _marker: PhantomData<(C, S, E)>,
}

/// Implementation of the event computation for the event sourced orchestrating aggregate.
impl<'a, C, S, E, Repository> EventComputation<C, S, E>
    for EventSourcedOrchestratingAggregate<'a, C, S, E, Repository>
where
    Repository: EventOrchestratingRepository<C, E>,
    C: Identifier,
    E: Clone
        + EventType
        + Identifier
        + IsFinal
        + DeciderType
        + DeserializeOwned
        + Serialize
        + Debug,
{
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Vec<E> {
        let current_state: S = current_events
            .iter()
            .fold((self.decider.initial_state)(), |state, event| {
                (self.decider.evolve)(&state, event)
            });

        // Initial resulting events from the decider's decision.
        let initial_events = (self.decider.decide)(command, &current_state);

        // Commands to process derived from initial resulting events.
        let commands_to_process: Vec<C> = initial_events
            .iter()
            .flat_map(|event| (self.saga.react)(event))
            .collect();

        // Collect all events including recursively computed new events.
        let mut all_events = initial_events.clone(); // Start with initial events.

        for command in commands_to_process.iter() {
            let previous_events = [
                self.repository
                    .fetch_events(command)
                    .unwrap_or_default()
                    .iter()
                    .map(|(e, _)| e.clone())
                    .collect::<Vec<E>>(),
                initial_events.clone(),
            ]
            .concat();

            // Recursively compute new events and extend the accumulated events list.
            let new_events = self.compute_new_events(&previous_events, command);
            all_events.extend(new_events);
        }

        all_events
    }
}

impl<'a, C, S, E, Repository> EventSourcedOrchestratingAggregate<'a, C, S, E, Repository>
where
    Repository: EventOrchestratingRepository<C, E>,
    C: Identifier,
    E: Clone
        + EventType
        + Identifier
        + IsFinal
        + DeciderType
        + DeserializeOwned
        + Serialize
        + Debug,
{
    /// Creates a new event sourced orchestrating aggregate.
    pub fn new(
        repository: Repository,
        decider: Decider<'a, C, S, E>,
        saga: Saga<'a, E, C>,
    ) -> Self {
        EventSourcedOrchestratingAggregate {
            repository,
            decider,
            saga,
            _marker: PhantomData,
        }
    }
    /// Handles the command and returns the new events that are persisted.
    pub fn handle(&self, command: &C) -> Result<Vec<(E, Uuid)>, ErrorMessage> {
        let events: Vec<E> = self
            .repository
            .fetch_events(command)?
            .into_iter()
            .map(|(e, _)| e)
            .collect();
        let new_events = self.compute_new_events(&events, command);
        self.repository.save(&new_events)
    }

    /// Handles the list of commands and returns the new events that are persisted.
    /// This method is useful for processing multiple commands in a single transaction.
    /// Effects/Events of the previous commands are visible to the subsequent commands.
    pub fn handle_all(&self, commands: &[C]) -> Result<Vec<(E, Uuid)>, ErrorMessage> {
        let mut all_new_events: Vec<E> = Vec::new();

        for command in commands {
            // Fetch events for the current command
            let fetched_events: Vec<E> = self
                .repository
                .fetch_events(command)?
                .into_iter()
                .map(|(e, _)| e)
                .collect();

            // Combine all previous new events with fetched events for the current command
            let combined_events: Vec<E> = fetched_events
                .into_iter()
                .chain(all_new_events.iter().cloned())
                .collect();

            // Compute new events based on the combined events and the current command
            let new_events = self.compute_new_events(&combined_events, command);

            // Accumulate all new events
            all_new_events.extend(new_events);
        }

        // Save all new events at the end
        self.repository.save(&all_new_events)
    }
}
