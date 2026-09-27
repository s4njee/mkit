//! Stateless semantic progress indicator using shared theme tokens.
extern crate gpui_pre as gpui;
use gpui_pre::{IntoElement, RenderOnce, div, prelude::*, px, relative};
use mkit_core::theme::Theme;
#[derive(IntoElement)]
pub struct Progress {
    label: String,
    min: f32,
    max: f32,
    value: Option<f32>,
}
impl Progress {
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        Self { label: label.into(), min: 0.0, max: 100.0, value: Some(value) }
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        if min.is_finite() && max.is_finite() && max > min {
            self.min = min;
            self.max = max;
        } else {
            self.min = 0.0;
            self.max = 100.0;
        }
        self
    }
    pub fn indeterminate(mut self) -> Self {
        self.value = None;
        self
    }
}
impl RenderOnce for Progress {
    fn render(self, _: &mut gpui_pre::Window, cx: &mut gpui_pre::App) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        let value =
            self.value.map(|v| if v.is_finite() { v.clamp(self.min, self.max) } else { self.min });
        let fill = value.map_or(0.35, |v| (v - self.min) / (self.max - self.min));
        let inset = if value.is_none() { 0.325 } else { 0.0 };
        div()
            .id("progress")
            .role(gpui_pre::accesskit::Role::ProgressIndicator)
            .aria_label(self.label)
            .aria_min_numeric_value(self.min as f64)
            .aria_max_numeric_value(self.max as f64)
            .when_some(value, |d, v| d.aria_numeric_value(v as f64))
            .when(value.is_none(), |d| d.aria_description("In progress"))
            .w_full()
            .h(px(t.spacing.xsmall))
            .rounded(px(t.radii.pill))
            .bg(t.colors.border)
            .child(
                div()
                    .h_full()
                    .ml(relative(inset))
                    .w(relative(fill))
                    .rounded(px(t.radii.pill))
                    .bg(t.colors.accent),
            )
    }
}
