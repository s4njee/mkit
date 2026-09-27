//! Example of a persistent GPUI status node with AccessKit live-region semantics.

extern crate gpui_pre as gpui;

use gpui_pre::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Role, Window, div, prelude::*,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};

/// Stable element identifier for this status region.
pub const STATUS_REGION_ID: &str = "e4-operation-status";

/// Stateful status view. Updating its message preserves the accessibility node
/// identity and notifies GPUI to rebuild the node with a new value.
pub struct StatusAnnouncer {
    message: String,
    revision: u64,
}

impl StatusAnnouncer {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), revision: 0 }
    }

    /// Set the current status and notify GPUI when its value changes.
    ///
    /// Returns `true` only when a new value was committed. Repeating the same
    /// message does not schedule a redundant render or live-region update.
    pub fn update_status(&mut self, message: impl Into<String>, cx: &mut Context<Self>) -> bool {
        let message = message.into();
        if self.message == message {
            return false;
        }

        self.message = message;
        self.revision += 1;
        cx.notify();
        true
    }

    /// Current visible and accessible status value.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Accessible label supplied to adapters that use the node name for live
    /// region events.
    pub fn accessible_name(&self) -> String {
        format!("Operation status: {}", self.message)
    }

    /// Number of distinct status updates applied to this view.
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

impl Render for StatusAnnouncer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let message = self.message.clone();
        let accessible_name = self.accessible_name();
        let revision = self.revision;
        let colors = &cx.global::<Theme>().colors;

        div()
            .size_full()
            .bg(colors.background)
            .text_color(colors.text)
            .flex()
            .flex_col()
            .items_start()
            .gap_3()
            .p_6()
            .child(div().text_2xl().child("Status announcement"))
            .child(
                div()
                    .id(STATUS_REGION_ID)
                    .debug_selector(move || format!("status-revision-{revision}"))
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(colors.surface)
                    .a11y_role(Role::Status)
                    .a11y_name(accessible_name)
                    .a11y_value(message.clone())
                    .a11y_live_region(LiveRegionPriority::Polite)
                    .child(message),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{STATUS_REGION_ID, StatusAnnouncer};
    use gpui_pre::TestAppContext;

    #[gpui_pre::test]
    fn status_update_notifies_and_keeps_the_same_region_identity(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (status, visual) = cx.add_window_view(|_, _| StatusAnnouncer::new("Ready"));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("status-revision-0").is_some());

        let changed =
            status.update(visual, |status, cx| status.update_status("Upload complete", cx));
        assert!(changed);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("status-revision-1").is_some());

        let state = status.read_with(visual, |status, _| {
            (status.message().to_owned(), status.accessible_name(), status.revision())
        });
        assert_eq!(
            state,
            ("Upload complete".to_owned(), "Operation status: Upload complete".to_owned(), 1)
        );
        assert_eq!(STATUS_REGION_ID, "e4-operation-status");
    }

    #[gpui_pre::test]
    fn duplicate_status_does_not_notify_or_advance_revision(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (status, visual) = cx.add_window_view(|_, _| StatusAnnouncer::new("Ready"));
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let changed = status.update(visual, |status, cx| status.update_status("Ready", cx));

        assert!(!changed);
        let state = status.read_with(visual, |status, _| {
            (status.message().to_owned(), status.accessible_name(), status.revision())
        });
        assert_eq!(state, ("Ready".to_owned(), "Operation status: Ready".to_owned(), 0));
    }
}
