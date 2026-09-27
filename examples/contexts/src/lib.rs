//! Nonvisual example of the app, entity, and async contexts used by a GPUI task.

use gpui_pre::{App, AppContext, AsyncApp, Context, Entity, Global, Task, WeakEntity};

/// State for a small refresh operation that updates a model after async work.
#[derive(Default)]
pub struct RefreshState {
    pub completed: bool,
}

/// The application keeps this strong handle so the example model outlives setup.
pub struct RefreshOwner(pub Entity<RefreshState>);

impl Global for RefreshOwner {}

// ANCHOR: contexts_model
impl RefreshState {
    /// Entity update closures receive a `Context<Self>` alongside mutable state.
    /// It provides entity operations such as `notify`, as well as app operations.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        self.completed = true;
        cx.notify();
    }
}
// ANCHOR_END: contexts_model

// ANCHOR: contexts_start_refresh
/// Start a refresh from an app-level callback. The weak handle lets the task
/// safely skip its update if the owning view/model has been released.
pub fn start_refresh(state: &Entity<RefreshState>, cx: &mut App) -> Task<()> {
    let state = state.downgrade();
    cx.spawn(async move |async_cx: &mut AsyncApp| {
        // The placeholder work stands in for a request or other async operation.
        // No `Window` is needed: this task only updates app-owned model state.
        apply_refresh(async_cx, state);
    })
}

fn apply_refresh(async_cx: &mut AsyncApp, state: WeakEntity<RefreshState>) {
    async_cx.update(|app| {
        if let Some(state) = state.upgrade() {
            state.update(app, |state, cx| state.finish(cx));
        }
    });
}
// ANCHOR_END: contexts_start_refresh

// ANCHOR: contexts_app_setup
/// Create app-owned state, retain it for the app lifetime, and start the refresh.
pub fn setup(cx: &mut App) -> Entity<RefreshState> {
    let state = cx.new(|_| RefreshState::default());
    cx.set_global(RefreshOwner(state.clone()));
    start_refresh(&state, cx).detach();
    state
}
// ANCHOR_END: contexts_app_setup

#[cfg(test)]
mod tests {
    use super::{RefreshState, setup, start_refresh};
    use gpui_kit::{AppContext, TestAppContext};

    #[gpui_kit::test]
    fn app_spawn_updates_entity_through_async_app(cx: &mut TestAppContext) {
        let state = cx.update(setup);
        cx.run_until_parked();

        assert!(state.read_with(cx, |state, _| state.completed));
    }

    #[gpui_kit::test]
    fn async_task_skips_released_entity(cx: &mut TestAppContext) {
        let state = cx.update(|app| app.new(|_| RefreshState::default()));
        let weak = state.downgrade();
        let task = cx.update(|app| start_refresh(&state, app));
        drop(state);
        assert!(!task.is_ready());
        cx.run_until_parked();

        assert!(task.is_ready(), "the async task ran to completion");
        assert!(weak.upgrade().is_none());
    }
}
