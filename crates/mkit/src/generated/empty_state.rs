//! Stateless, theme-driven empty-state composition.
extern crate gpui_pre as gpui;

use gpui_pre::{
    AnyElement, App, FontWeight, IntoElement, RenderOnce, Rgba, Window, div, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(IntoElement)]
pub struct EmptyState {
    id: usize,
    title: String,
    description: Option<String>,
    icon: Option<AnyElement>,
    actions: Vec<AnyElement>,
}

impl EmptyState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            title: title.into(),
            description: None,
            icon: None,
            actions: Vec::new(),
        }
    }
    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

/// Icon tile fill and optional outline; see the spec's "Theme tokens used" table.
fn icon_tile(t: &Theme) -> (Rgba, Option<Rgba>) {
    let c = t.colors;
    if t.name == "high-contrast" {
        return (c.background, Some(c.border));
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn's `muted`: `text` mixed into `background`, like CSS `color-mix(in srgb, ...)`.
    (composite(Rgba { a: if dark { 0.12 } else { 0.04 }, ..c.text }, c.background), None)
}

impl RenderOnce for EmptyState {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let mut header = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(theme.spacing.small))
            .max_w(px(theme.spacing.xxlarge * 12.0));
        if let Some(icon) = self.icon {
            let (fill, outline) = icon_tile(&theme);
            header = header.child(
                div()
                    .debug_selector(|| "empty-state-icon".to_string())
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(theme.controls.large))
                    .mb(px(theme.spacing.small))
                    .rounded(px(theme.radii.large))
                    .bg(fill)
                    .when_some(outline, |el, color| {
                        el.border(px(theme.borders.hairline)).border_color(color)
                    })
                    .text_color(theme.colors.text)
                    .child(icon),
            );
        }
        header = header.child(
            div()
                .debug_selector(|| "empty-state-title".to_string())
                .text_size(px(theme.typography.heading_small))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.colors.text)
                .text_center()
                .child(self.title.clone()),
        );
        if let Some(description) = self.description {
            header = header.child(
                div()
                    .debug_selector(|| "empty-state-description".to_string())
                    .text_size(px(theme.typography.body))
                    .text_color(theme.colors.text_muted)
                    .text_center()
                    .child(description),
            );
        }
        let mut root = div()
            .id(("mkit-empty-state", self.id))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(self.title)
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(theme.spacing.xlarge))
            .p(px(theme.spacing.xlarge))
            .text_color(theme.colors.text)
            .child(header);
        if !self.actions.is_empty() {
            let mut actions = div()
                .id(format!("empty-state-actions-{}", self.id))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(theme.spacing.small));
            for (index, action) in self.actions.into_iter().enumerate() {
                actions = actions.child(
                    div()
                        .id(format!("empty-state-action-{index}"))
                        .debug_selector({
                            let selector = format!("empty-state-action-{index}");
                            move || selector.clone()
                        })
                        .child(action),
                );
            }
            root = root.child(actions);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Context, IntoElement, Render, TestAppContext};

    struct Host;
    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            EmptyState::new("No projects")
                .description("Create a project to get started.")
                .icon(div().id("empty-state-icon").child("◇"))
                .action(
                    div()
                        .id("create-project")
                        .role(gpui_pre::accesskit::Role::Button)
                        .aria_label("Create project")
                        .child("Create project"),
                )
        }
    }

    #[gpui::test]
    fn renders_optional_icon_description_and_action(cx: &mut TestAppContext) {
        cx.update(|app| mkit_core::theme::set_theme(app, mkit_core::theme::SHADCN_LIGHT));
        let (_, window) = cx.add_window_view(|_, _| Host);
        window.update(|w, cx| w.draw(cx).clear(cx));
        assert!(window.debug_bounds("empty-state-icon").is_some());
        assert!(window.debug_bounds("empty-state-action-0").is_some());
    }
}
