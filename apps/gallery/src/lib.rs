//! Component gallery view and the legacy fixture shared with the MCP inspector.

use gpui_pre::{Context, Entity, Render, Window, div, prelude::*, px};
use mkit::combobox::{Combobox, OptionItem};
use mkit::core::theme::{DARK, HIGH_CONTRAST, LIGHT, Theme};
use mkit::scrubbable_number_field::ScrubbableNumberField;

pub mod e7;
pub mod e8;

pub use mkit_example_hello::InspectorFixture;

/// Interactive preview of the E5 pilot components.
pub struct ComponentGallery {
    combobox: Option<Entity<Combobox>>,
    number_field: Option<Entity<ScrubbableNumberField>>,
}

impl ComponentGallery {
    pub fn new() -> Self {
        Self { combobox: None, number_field: None }
    }
}

impl Default for ComponentGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ComponentGallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let combobox = self
            .combobox
            .get_or_insert_with(|| {
                cx.new(|_| {
                    Combobox::new(
                        "Favorite fruit",
                        vec![
                            OptionItem::new("apple", "Apple"),
                            OptionItem::new("pear", "Pear"),
                            OptionItem::new("plum", "Plum"),
                            OptionItem::new("peach", "Peach"),
                        ],
                        Some("pear".to_owned()),
                    )
                })
            })
            .clone();
        let number_field = self
            .number_field
            .get_or_insert_with(|| {
                cx.new(|_| {
                    ScrubbableNumberField::new(12.5)
                        .label("Opacity")
                        .bounds(Some(0.0), Some(100.0))
                        .step(0.5)
                })
            })
            .clone();

        let theme_button = |id: &'static str, label: &'static str, selected: bool| {
            let background = if selected { theme.colors.accent } else { theme.colors.surface };
            let foreground = if selected { theme.colors.accent_text } else { theme.colors.text };
            div()
                .id(id)
                .px(px(theme.spacing.medium))
                .py(px(theme.spacing.xsmall))
                .rounded(px(theme.radii.small))
                .border(px(theme.borders.hairline))
                .border_color(theme.colors.border)
                .bg(background)
                .text_color(foreground)
                .child(label)
        };

        div()
            .id("mkit-component-gallery")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.large))
            .child(
                div()
                    .text_size(px(theme.typography.heading_large))
                    .child("mkit component gallery"),
            )
            .child(
                div()
                    .text_color(theme.colors.text_muted)
                    .child("E5 pilot previews. Type to filter the combobox; use the number field's arrows or drag horizontally to scrub."),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme.spacing.small))
                    .child(div().text_color(theme.colors.text_muted).child("Theme"))
                    .child(theme_button("theme-light", "Light", theme.name == LIGHT.name)
                        .on_click(cx.listener(|_, _, _, cx| { cx.set_global(LIGHT); cx.notify(); })))
                    .child(theme_button("theme-dark", "Dark", theme.name == DARK.name)
                        .on_click(cx.listener(|_, _, _, cx| { cx.set_global(DARK); cx.notify(); })))
                    .child(theme_button("theme-contrast", "High contrast", theme.name == HIGH_CONTRAST.name)
                        .on_click(cx.listener(|_, _, _, cx| { cx.set_global(HIGH_CONTRAST); cx.notify(); }))),
            )
            .child(
                div()
                    .p(px(theme.spacing.large))
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .child(div().text_size(px(theme.typography.heading)).child("Combobox"))
                    .child(div().text_color(theme.colors.text_muted).child("Editable single selection with filtered options."))
                    .child(combobox),
            )
            .child(
                div()
                    .p(px(theme.spacing.large))
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .child(div().text_size(px(theme.typography.heading)).child("Scrubbable number field"))
                    .child(div().text_color(theme.colors.text_muted).child("Edit with the keyboard, or drag horizontally to change the value."))
                    .child(number_field),
            )
    }
}

/// Construct the gallery's inspectable preview view.
///
/// The MCP inspector keeps this small fixture for its existing click/state
/// contract. The standalone gallery app launches [`ComponentGallery`].
pub fn inspector_fixture() -> InspectorFixture {
    InspectorFixture::gallery()
}
