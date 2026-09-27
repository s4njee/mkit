//! Stateless status bar with token-based styling and priority-based narrow layouts.

extern crate gpui_pre as gpui;

use gpui_pre::{AnyElement, App, IntoElement, RenderOnce, Window, div, prelude::*, px, relative};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_STATUS_BAR_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapsePriority {
    Never,
    Low,
    #[default]
    Normal,
    High,
}

impl CollapsePriority {
    fn rank(self) -> Option<u8> {
        match self {
            Self::Low => Some(0),
            Self::Normal => Some(1),
            Self::High => Some(2),
            Self::Never => None,
        }
    }
}

enum StatusContent {
    Text(String),
    Element(AnyElement),
    Progress { label: String, min: f32, max: f32, value: Option<f32> },
}

pub struct StatusBarItem {
    key: String,
    content: StatusContent,
    estimated_width: Option<f32>,
    collapse_priority: CollapsePriority,
}

impl StatusBarItem {
    pub fn text(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(key, StatusContent::Text(text.into()))
    }

    /// Add a caller-owned button element. Its button semantics and event handlers are preserved.
    pub fn button<E>(key: impl Into<String>, button: E) -> Self
    where
        E: IntoElement + 'static,
    {
        Self::new(key, StatusContent::Element(button.into_any_element()))
    }

    /// Add a determinate progress indicator over the conventional 0–100 range.
    pub fn progress(key: impl Into<String>, label: impl Into<String>, value: f32) -> Self {
        Self::new(
            key,
            StatusContent::Progress {
                label: label.into(),
                min: 0.0,
                max: 100.0,
                value: Some(value),
            },
        )
    }

    /// Add an indeterminate progress indicator.
    pub fn indeterminate_progress(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(
            key,
            StatusContent::Progress { label: label.into(), min: 0.0, max: 100.0, value: None },
        )
    }

    fn new(key: impl Into<String>, content: StatusContent) -> Self {
        Self {
            key: key.into(),
            content,
            estimated_width: None,
            collapse_priority: CollapsePriority::Normal,
        }
    }

    /// Set the estimated width in logical pixels used by narrow-width collapse decisions.
    pub fn estimated_width(mut self, width: f32) -> Self {
        self.estimated_width = (width.is_finite() && width > 0.0).then_some(width);
        self
    }

    pub fn collapse_priority(mut self, priority: CollapsePriority) -> Self {
        self.collapse_priority = priority;
        self
    }

    fn width(&self, theme: &Theme) -> f32 {
        self.estimated_width.unwrap_or_else(|| match &self.content {
            StatusContent::Text(text) => {
                text.chars().count() as f32 * theme.typography.body * 0.55
                    + theme.spacing.small * 2.0
            }
            StatusContent::Element(_) => theme.controls.large * 3.0,
            StatusContent::Progress { .. } => theme.controls.large * 4.0,
        })
    }

    fn render(self, id: &str, theme: &Theme) -> AnyElement {
        let width = self.width(theme);
        let key = self.key;
        let item_id = format!("{id}-{key}");
        match self.content {
            StatusContent::Text(text) => div()
                .id(item_id.clone())
                .w(px(width))
                .h(px(theme.controls.large))
                .flex()
                .items_center()
                .overflow_hidden()
                .text_color(theme.colors.text_muted)
                .text_size(px(theme.typography.body))
                .role(gpui_pre::accesskit::Role::Status)
                .aria_label(text.clone())
                .a11y_live_region(LiveRegionPriority::Polite)
                .child(text)
                .into_any_element(),
            StatusContent::Element(element) => div()
                .id(item_id.clone())
                .w(px(width))
                .h(px(theme.controls.large))
                .flex()
                .items_center()
                .flex_shrink_0()
                .child(element)
                .into_any_element(),
            StatusContent::Progress { label, min, max, value } => {
                let min = if min.is_finite() { min } else { 0.0 };
                let max = if max.is_finite() && max > min { max } else { min + 100.0 };
                let value =
                    value.map(|value| if value.is_finite() { value.clamp(min, max) } else { min });
                let fill = value.map_or(0.35, |value| (value - min) / (max - min));
                div()
                    .id(item_id)
                    .w(px(width))
                    .h(px(theme.controls.large))
                    .flex()
                    .items_center()
                    .role(gpui_pre::accesskit::Role::ProgressIndicator)
                    .aria_label(label)
                    .aria_min_numeric_value(min as f64)
                    .aria_max_numeric_value(max as f64)
                    .when_some(value, |element, value| element.aria_numeric_value(value as f64))
                    .when(value.is_none(), |element| element.aria_description("In progress"))
                    .child(
                        div()
                            .w_full()
                            .h(px(theme.spacing.xsmall))
                            .overflow_hidden()
                            .rounded(px(theme.radii.pill))
                            .bg(theme.colors.border)
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(fill))
                                    .rounded(px(theme.radii.pill))
                                    .bg(theme.colors.accent),
                            ),
                    )
                    .into_any_element()
            }
        }
    }
}

#[derive(IntoElement)]
pub struct StatusBar {
    id: String,
    label: String,
    leading: Vec<StatusBarItem>,
    trailing: Vec<StatusBarItem>,
    available_width: Option<f32>,
}

impl StatusBar {
    pub fn new(label: impl Into<String>) -> Self {
        let id = NEXT_STATUS_BAR_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id: format!("mkit-status-bar-{id}"),
            label: label.into(),
            leading: Vec::new(),
            trailing: Vec::new(),
            available_width: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn leading(mut self, item: StatusBarItem) -> Self {
        self.leading.push(item);
        self
    }

    pub fn trailing(mut self, item: StatusBarItem) -> Self {
        self.trailing.push(item);
        self
    }

    /// Set the parent layout's current width in logical pixels to enable priority collapsing.
    pub fn available_width(mut self, width: f32) -> Self {
        self.available_width = (width.is_finite() && width > 0.0).then_some(width);
        self
    }

    fn visibility(&self, theme: &Theme) -> (Vec<bool>, Vec<bool>) {
        let mut leading = vec![true; self.leading.len()];
        let mut trailing = vec![true; self.trailing.len()];
        let Some(available_width) = self.available_width else {
            return (leading, trailing);
        };

        let mut visible_width = self.leading.iter().map(|item| item.width(theme)).sum::<f32>()
            + self.trailing.iter().map(|item| item.width(theme)).sum::<f32>();
        let has_both_regions = |leading: &[bool], trailing: &[bool]| {
            leading.iter().any(|visible| *visible) && trailing.iter().any(|visible| *visible)
        };
        loop {
            let leading_count = leading.iter().filter(|visible| **visible).count();
            let trailing_count = trailing.iter().filter(|visible| **visible).count();
            let overhead = theme.spacing.medium * 2.0
                + theme.spacing.small
                    * (leading_count.saturating_sub(1) + trailing_count.saturating_sub(1)) as f32
                + if has_both_regions(&leading, &trailing) { theme.spacing.medium } else { 0.0 };
            if visible_width + overhead <= available_width {
                break;
            }
            let candidate = self
                .leading
                .iter()
                .enumerate()
                .filter(|(index, item)| leading[*index] && item.collapse_priority.rank().is_some())
                .map(|(index, item)| (item.collapse_priority.rank().unwrap(), index, true))
                .chain(
                    self.trailing
                        .iter()
                        .enumerate()
                        .filter(|(index, item)| {
                            trailing[*index] && item.collapse_priority.rank().is_some()
                        })
                        .map(|(index, item)| {
                            (item.collapse_priority.rank().unwrap(), index, false)
                        }),
                )
                .min_by_key(|(rank, index, is_leading)| {
                    (*rank, if *is_leading { 0 } else { 1 }, *index)
                });
            let Some((_, index, is_leading)) = candidate else {
                break;
            };
            let item = if is_leading {
                leading[index] = false;
                &self.leading[index]
            } else {
                trailing[index] = false;
                &self.trailing[index]
            };
            visible_width -= item.width(theme);
        }
        (leading, trailing)
    }
}

impl RenderOnce for StatusBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let (leading_visible, trailing_visible) = self.visibility(&theme);
        let leading_items = self
            .leading
            .into_iter()
            .zip(leading_visible)
            .filter_map(|(item, visible)| visible.then(|| item.render(&self.id, &theme)));
        let trailing_items = self
            .trailing
            .into_iter()
            .zip(trailing_visible)
            .filter_map(|(item, visible)| visible.then(|| item.render(&self.id, &theme)));
        div()
            .id(self.id.clone())
            .w_full()
            .h(px(theme.controls.large))
            .px(px(theme.spacing.medium))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(theme.spacing.medium))
            .overflow_hidden()
            .border_t(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface)
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.label)
            .child(
                div()
                    .id(format!("{}-leading", self.id))
                    .flex()
                    .items_center()
                    .gap(px(theme.spacing.small))
                    .children(leading_items),
            )
            .child(
                div()
                    .id(format!("{}-trailing", self.id))
                    .flex()
                    .items_center()
                    .gap(px(theme.spacing.small))
                    .children(trailing_items),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{FocusHandle, TestAppContext, actions};
    use std::{cell::Cell, rc::Rc};

    actions!(status_bar_tests, [Activate]);

    struct StatusUpdateFixture {
        message: String,
        action_focus: Option<FocusHandle>,
        activations: Rc<Cell<usize>>,
    }

    impl gpui::Render for StatusUpdateFixture {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            let theme = *cx.global::<Theme>();
            let focus = self.action_focus.get_or_insert_with(|| cx.focus_handle()).clone();
            let button_focus = focus.clone();
            let button = div()
                .id("status-bar-action")
                .debug_selector(|| "status-bar-action".to_owned())
                .key_context("StatusBarButton")
                .track_focus(&button_focus)
                .tab_stop(true)
                .tab_index(0)
                .role(gpui::accesskit::Role::Button)
                .aria_label("Details")
                .on_action(cx.listener(|this, _: &Activate, _, _| {
                    this.activations.set(this.activations.get() + 1);
                }))
                .h(px(theme.controls.large))
                .px(px(theme.spacing.small))
                .child("Details");
            StatusBar::new("Status updates")
                .leading(StatusBarItem::text("message", self.message.clone()))
                .trailing(StatusBarItem::button("details", button).estimated_width(80.0))
        }
    }

    #[test]
    fn collapse_hides_low_priority_first_and_never_hides_essential_items() {
        let theme = mkit_core::theme::SHADCN_LIGHT;
        let bar = StatusBar::new("Status")
            .available_width(220.0)
            .leading(
                StatusBarItem::text("document", "Document ready")
                    .estimated_width(100.0)
                    .collapse_priority(CollapsePriority::Never),
            )
            .leading(
                StatusBarItem::text("sync", "Synced")
                    .estimated_width(60.0)
                    .collapse_priority(CollapsePriority::Low),
            )
            .trailing(
                StatusBarItem::text("selection", "3 objects")
                    .estimated_width(72.0)
                    .collapse_priority(CollapsePriority::High),
            );

        let (leading, trailing) = bar.visibility(&theme);
        assert_eq!(leading, [true, false]);
        assert_eq!(trailing, [true]);
    }

    #[test]
    fn no_width_hint_keeps_every_item_visible_and_equal_priorities_collapse_in_order() {
        let theme = mkit_core::theme::SHADCN_LIGHT;
        let full = StatusBar::new("Status")
            .leading(StatusBarItem::text("one", "One"))
            .trailing(StatusBarItem::text("two", "Two"));
        assert_eq!(full.visibility(&theme), (vec![true], vec![true]));

        let narrow = StatusBar::new("Status")
            .available_width(1.0)
            .leading(StatusBarItem::text("one", "One").estimated_width(30.0))
            .trailing(StatusBarItem::text("two", "Two").estimated_width(30.0));
        assert_eq!(narrow.visibility(&theme), (vec![false], vec![false]));
    }

    #[gpui::test]
    fn child_button_keeps_keyboard_activation_and_status_updates_keep_focus(
        cx: &mut TestAppContext,
    ) {
        cx.update(|app| {
            mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT);
            app.bind_keys([
                gpui::KeyBinding::new("enter", Activate, Some("StatusBarButton")),
                gpui::KeyBinding::new("space", Activate, Some("StatusBarButton")),
            ]);
        });
        let activations = Rc::new(Cell::new(0));
        let (view, window) = cx.add_window_view({
            let activations = activations.clone();
            move |_, _| StatusUpdateFixture {
                message: "Saving".to_owned(),
                action_focus: None,
                activations,
            }
        });
        window.update(|window, app| {
            window.draw(app).clear(app);
            let focus = view.read(app).action_focus.as_ref().unwrap().clone();
            focus.focus(window, app);
        });

        window.simulate_keystrokes("enter space");
        assert_eq!(activations.get(), 2);
        let focus = window.update(|_, app| view.read(app).action_focus.as_ref().unwrap().clone());
        assert!(window.update(|window, _| focus.is_focused(window)));

        view.update(window, |view, cx| {
            view.message = "Saved".to_owned();
            cx.notify();
        });
        window.update(|window, app| window.draw(app).clear(app));
        assert!(window.update(|window, _| focus.is_focused(window)));
    }
}
