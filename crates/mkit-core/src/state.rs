//! Shared state patterns for stateful components.
//!
//! This module models how a component handles a user-requested value change.
//! It deliberately does not implement GPUI's [`gpui_pre::EventEmitter`]: each
//! component owns its own entity type and event types, and emits the returned
//! [`StateChange`] from its `Context<T>`.

/// Whether a component owns its value or receives it from its parent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateMode {
    /// The component commits user-requested values locally.
    Uncontrolled,
    /// The parent supplies the displayed value and decides whether to accept
    /// each user-requested value.
    Controlled,
}

/// A value and the ownership mode used for user-requested changes.
///
/// A controlled state still stores the latest value supplied by its parent.
/// Calling [`request_change`](Self::request_change) never replaces that value
/// in controlled mode; the parent can accept the proposal by calling
/// [`set_value`](Self::set_value) with the next prop value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentState<T> {
    value: T,
    mode: StateMode,
}

impl<T> ComponentState<T> {
    /// Create state whose value is owned by the component.
    pub const fn uncontrolled(value: T) -> Self {
        Self { value, mode: StateMode::Uncontrolled }
    }

    /// Create state whose value is supplied and controlled by a parent.
    pub const fn controlled(value: T) -> Self {
        Self { value, mode: StateMode::Controlled }
    }

    /// Return the current value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Return the ownership mode.
    pub const fn mode(&self) -> StateMode {
        self.mode
    }

    /// Replace the value supplied by the owner.
    ///
    /// For uncontrolled state this is useful when a component is explicitly
    /// reset. For controlled state this represents new parent-provided props.
    pub fn set_value(&mut self, value: T) {
        self.value = value;
    }

    /// Process a user-requested value and return the typed change payload.
    ///
    /// Uncontrolled state commits the proposal before returning. Controlled
    /// state leaves its current value untouched; the caller emits the returned
    /// proposal and waits for the owner to send an updated value. A payload is
    /// returned for every request, including a request equal to the current
    /// value, so deduplication policy stays with the component/parent contract.
    pub fn request_change(&mut self, value: T) -> StateChange<T>
    where
        T: Clone,
    {
        let previous = self.value.clone();
        if self.mode == StateMode::Uncontrolled {
            self.value = value.clone();
        }
        StateChange { previous, value, mode: self.mode }
    }
}

/// Typed payload describing one user-requested state change.
///
/// Components commonly wrap this in a domain-named event (for example,
/// `SelectionChanged(StateChange<usize>)`) and implement
/// `EventEmitter<SelectionChanged>` on their entity type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateChange<T> {
    previous: T,
    value: T,
    mode: StateMode,
}

impl<T> StateChange<T> {
    /// The value visible immediately before the request.
    pub const fn previous(&self) -> &T {
        &self.previous
    }

    /// The value requested by the user.
    ///
    /// In controlled mode this is a proposal; it does not become the displayed
    /// value until the owner supplies it through [`ComponentState::set_value`].
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// The state ownership mode at the time of the request.
    pub const fn mode(&self) -> StateMode {
        self.mode
    }

    /// Whether the request was committed locally.
    pub const fn was_committed(&self) -> bool {
        matches!(self.mode, StateMode::Uncontrolled)
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentState, StateMode};

    #[test]
    fn uncontrolled_change_commits_and_reports_previous_and_next_values() {
        let mut state = ComponentState::uncontrolled(2);

        let change = state.request_change(5);

        assert_eq!(state.value(), &5);
        assert_eq!(change.previous(), &2);
        assert_eq!(change.value(), &5);
        assert_eq!(change.mode(), StateMode::Uncontrolled);
        assert!(change.was_committed());
    }

    #[test]
    fn controlled_change_reports_proposal_without_committing_until_owner_update() {
        let mut state = ComponentState::controlled(String::from("closed"));

        let change = state.request_change(String::from("open"));

        assert_eq!(state.value(), "closed");
        assert_eq!(change.previous(), "closed");
        assert_eq!(change.value(), "open");
        assert_eq!(change.mode(), StateMode::Controlled);
        assert!(!change.was_committed());

        state.set_value(String::from("open"));
        assert_eq!(state.value(), "open");
    }

    #[test]
    fn same_value_request_still_returns_a_change_payload() {
        let mut state = ComponentState::uncontrolled(7);

        let change = state.request_change(7);

        assert_eq!(change.previous(), &7);
        assert_eq!(change.value(), &7);
        assert_eq!(state.value(), &7);
    }
}
