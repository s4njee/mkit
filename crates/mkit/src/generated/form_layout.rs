//! Aligned rows for labeled controls and helper text.
extern crate gpui_pre as gpui;

use gpui_pre::{AnyElement, App, FontWeight, IntoElement, RenderOnce, Window, div, prelude::*, px};
use mkit_core::theme::Theme;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub struct FormRow {
    label: String,
    control: AnyElement,
    description: Option<String>,
    error: Option<String>,
}

impl FormRow {
    pub fn new(label: impl Into<String>, control: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            control: control.into_any_element(),
            description: None,
            error: None,
        }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn error(mut self, text: impl Into<String>) -> Self {
        self.error = Some(text.into());
        self
    }
}

#[derive(IntoElement)]
pub struct FormLayout {
    id: usize,
    label: Option<String>,
    label_width: Option<f32>,
    compact: bool,
    rows: Vec<FormRow>,
}

impl FormLayout {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            label: None,
            label_width: None,
            compact: false,
            rows: Vec::new(),
        }
    }
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = Some(width.max(0.0));
        self
    }
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    pub fn row(mut self, row: FormRow) -> Self {
        self.rows.push(row);
        self
    }
}

impl Default for FormLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for FormLayout {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        // shadcn field metrics: 24px between fields, 8px between a control and its helper
        // text, and the docs-site preview's 16px label column gap. Compact steps each down.
        let row_gap = if self.compact { theme.spacing.large } else { theme.spacing.xlarge };
        let column_gap = if self.compact { theme.spacing.medium } else { theme.spacing.large };
        let helper_gap = if self.compact { theme.spacing.xsmall } else { theme.spacing.small };
        let label_width = self.label_width.unwrap_or(theme.spacing.xxlarge * 4.0);
        let mut root = div()
            .id(("mkit-form-layout", self.id))
            .role(gpui_pre::accesskit::Role::Group)
            .when_some(self.label, |el, label| el.aria_label(label))
            .flex()
            .flex_col()
            .gap(px(row_gap));
        for row in self.rows {
            let mut control = div().flex().flex_col().gap(px(helper_gap)).child(row.control);
            if let Some(description) = row.description {
                control = control.child(
                    div()
                        .text_size(px(theme.typography.caption))
                        .text_color(theme.colors.text_muted)
                        .child(description),
                );
            }
            if let Some(error) = row.error {
                control = control.child(
                    div()
                        .text_size(px(theme.typography.caption))
                        .text_color(theme.colors.danger)
                        .child(error),
                );
            }
            root = root.child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(column_gap))
                    .child(
                        // The label box is one default control tall and centres its text, so a
                        // label lines up with a `controls.medium` field beside it.
                        div()
                            .w(px(label_width))
                            .flex_none()
                            .min_h(px(theme.controls.medium))
                            .flex()
                            .items_center()
                            .text_size(px(theme.typography.body))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.colors.text)
                            .child(row.label),
                    )
                    .child(control.flex_1()),
            );
        }
        root
    }
}
