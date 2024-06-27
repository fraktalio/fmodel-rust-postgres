use crate::framework::infrastructure::errors::ErrorMessage;
use crate::framework::infrastructure::view_state_repository::ViewStateRepository;
use fmodel_rust::view::ViewStateComputation;
use std::marker::PhantomData;

/// Materialized View.
///
/// It is using a `View` / [ViewStateComputation] to compute new state based on the current state and the event.
/// It is using a [ViewStateRepository] to fetch the current state and to save the new state.
///
/// Generic parameters:
///
/// - `S` - State
/// - `E` - Event
/// - `Repository` - View State repository
/// - `View` - View
pub struct MaterializedView<S, E, Repository, View>
where
    Repository: ViewStateRepository<E, S>,
    View: ViewStateComputation<E, S>,
{
    repository: Repository,
    view: View,
    _marker: PhantomData<(S, E)>,
}

/// Implementation of the view state computation for the materialized view.
impl<S, E, Repository, View> ViewStateComputation<E, S> for MaterializedView<S, E, Repository, View>
where
    Repository: ViewStateRepository<E, S>,
    View: ViewStateComputation<E, S>,
{
    /// Computes new state based on the current state and the events.
    fn compute_new_state(&self, current_state: Option<S>, events: &[&E]) -> S {
        self.view.compute_new_state(current_state, events)
    }
}

/// Implementation of the `handle` method for the materialized view.
impl<S, E, Repository, View> MaterializedView<S, E, Repository, View>
where
    Repository: ViewStateRepository<E, S>,
    View: ViewStateComputation<E, S>,
{
    /// Creates a new instance of [MaterializedView].
    pub fn new(repository: Repository, view: View) -> Self {
        MaterializedView {
            repository,
            view,
            _marker: PhantomData,
        }
    }
    /// Handles the event by fetching the state from the repository, computing new state based on the current state and the event, and saving the new state to the repository.
    pub fn handle(&self, event: &E) -> Result<S, ErrorMessage> {
        let state = self.repository.fetch_state(event)?;
        let new_state = self.compute_new_state(state, &[event]);
        self.repository.save(&new_state)
    }
}
