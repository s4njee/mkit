//! Compiling configuration examples for the E5 component pilots.

use mkit::combobox::{Combobox, OptionItem};
use mkit::scrubbable_number_field::ScrubbableNumberField;

// ANCHOR: combobox_options
fn countries() -> Vec<OptionItem> {
    vec![
        OptionItem::new("us", "United States"),
        OptionItem::new("ca", "Canada"),
        OptionItem::new("cm", "Cameroon"),
    ]
}
// ANCHOR_END: combobox_options

// ANCHOR: combobox_uncontrolled
/// The component owns its selection when no parent value is supplied.
pub fn country_picker() -> Combobox {
    Combobox::new("Country", countries(), Some("us".to_owned()))
}
// ANCHOR_END: combobox_uncontrolled

// ANCHOR: combobox_controlled
/// The owner receives `ValueChanged` and applies the accepted ID with `set_value`.
pub fn owned_country_picker(selected_id: Option<String>) -> Combobox {
    Combobox::controlled("Country", countries(), selected_id)
}
// ANCHOR_END: combobox_controlled

// ANCHOR: number_uncontrolled
/// A bounded value that supports typing, keys, and horizontal pointer scrubbing.
pub fn opacity_field() -> ScrubbableNumberField {
    ScrubbableNumberField::new(12.5).label("Opacity").bounds(Some(0.0), Some(100.0)).step(1.0)
}
// ANCHOR_END: number_uncontrolled

// ANCHOR: number_controlled
/// The owner receives `ValueChanged` and applies its accepted value with `set_value`.
pub fn owned_opacity_field(value: f64) -> ScrubbableNumberField {
    ScrubbableNumberField::controlled(value)
        .label("Opacity")
        .bounds(Some(0.0), Some(100.0))
        .step(1.0)
}
// ANCHOR_END: number_controlled

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_examples_have_the_documented_initial_values() {
        assert_eq!(country_picker().value(), Some("us"));
        assert_eq!(owned_country_picker(Some("ca".to_owned())).value(), Some("ca"));
        assert_eq!(opacity_field().value(), 12.5);
        assert_eq!(owned_opacity_field(120.0).value(), 100.0);
    }
}
