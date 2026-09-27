//! Typed, theme-driven property panel for pro applications.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding,
    KeyDownEvent, Render, Window, actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "PropertyInspector";
actions!(property_inspector, [NextProperty, PreviousProperty, ToggleBoolean, ResetProperty]);

pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("down", NextProperty, Some(KEY_CONTEXT)),
        KeyBinding::new("up", PreviousProperty, Some(KEY_CONTEXT)),
        KeyBinding::new("space", ToggleBoolean, Some(KEY_CONTEXT)),
        KeyBinding::new("r", ResetProperty, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Number(f64),
    Text(String),
    Boolean(bool),
    Enum(String),
    /// CSS-style hex string, retained as data so the host owns colour-space conversion.
    Color(String),
    Vector(Vec<f64>),
    Mixed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyKind {
    Number { step: f64 },
    Text,
    Boolean,
    Enum { options: Vec<String> },
    Color,
    Vector { dimensions: usize, step: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub id: String,
    pub label: String,
    pub kind: PropertyKind,
    pub value: PropertyValue,
    pub default: PropertyValue,
    pub disabled: bool,
}

impl Property {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: PropertyKind,
        value: PropertyValue,
        default: PropertyValue,
    ) -> Self {
        let mut property =
            Self { id: id.into(), label: label.into(), kind, value, default, disabled: false };
        property.normalize();
        property
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn normalize(&mut self) {
        let valid = match (&self.kind, &self.value, &self.default) {
            (PropertyKind::Number { .. }, PropertyValue::Number(_), PropertyValue::Number(_)) => {
                true
            }
            (PropertyKind::Text, PropertyValue::Text(_), PropertyValue::Text(_)) => true,
            (PropertyKind::Boolean, PropertyValue::Boolean(_), PropertyValue::Boolean(_)) => true,
            (PropertyKind::Enum { options }, PropertyValue::Enum(v), PropertyValue::Enum(d)) => {
                options.contains(v) && options.contains(d)
            }
            (PropertyKind::Color, PropertyValue::Color(_), PropertyValue::Color(_)) => true,
            (
                PropertyKind::Vector { dimensions, .. },
                PropertyValue::Vector(v),
                PropertyValue::Vector(d),
            ) => {
                *dimensions >= 2
                    && *dimensions <= 3
                    && v.len() == *dimensions
                    && d.len() == *dimensions
            }
            // Mixed is a valid current value for every editor, but defaults remain concrete.
            (PropertyKind::Number { .. }, PropertyValue::Mixed, PropertyValue::Number(_))
            | (PropertyKind::Text, PropertyValue::Mixed, PropertyValue::Text(_))
            | (PropertyKind::Boolean, PropertyValue::Mixed, PropertyValue::Boolean(_))
            | (PropertyKind::Enum { .. }, PropertyValue::Mixed, PropertyValue::Enum(_))
            | (PropertyKind::Color, PropertyValue::Mixed, PropertyValue::Color(_))
            | (PropertyKind::Vector { .. }, PropertyValue::Mixed, PropertyValue::Vector(_)) => true,
            _ => false,
        };
        if !valid {
            self.value = self.default.clone();
        }
        match &mut self.kind {
            PropertyKind::Number { step } | PropertyKind::Vector { step, .. }
                if !step.is_finite() || *step <= 0.0 =>
            {
                *step = 1.0
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyGroup {
    pub id: String,
    pub label: String,
    pub expanded: bool,
    pub properties: Vec<Property>,
}

impl PropertyGroup {
    pub fn new(id: impl Into<String>, label: impl Into<String>, properties: Vec<Property>) -> Self {
        Self { id: id.into(), label: label.into(), expanded: true, properties }
    }

    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValueChanged {
    pub property_id: String,
    pub value: PropertyValue,
}
impl EventEmitter<ValueChanged> for PropertyInspector {}

/// Grouped inspector with typed value proposals and per-property reset controls.
pub struct PropertyInspector {
    groups: Vec<PropertyGroup>,
    controlled: bool,
    disabled: bool,
    active: Option<String>,
    editing_text: bool,
    text_draft: String,
    focus: Option<FocusHandle>,
}

impl PropertyInspector {
    /// Create an uncontrolled inspector whose supplied values become its initial state.
    pub fn new(groups: Vec<PropertyGroup>) -> Self {
        Self {
            groups,
            controlled: false,
            disabled: false,
            active: None,
            editing_text: false,
            text_draft: String::new(),
            focus: None,
        }
    }

    /// Create an inspector whose owner remains authoritative for every property value.
    pub fn controlled(groups: Vec<PropertyGroup>) -> Self {
        Self { controlled: true, ..Self::new(groups) }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn groups(&self) -> &[PropertyGroup] {
        &self.groups
    }

    /// Replace schema and current values without emitting interaction events.
    pub fn set_groups(&mut self, groups: Vec<PropertyGroup>, cx: &mut Context<Self>) {
        self.groups = groups;
        self.reconcile_active();
        cx.notify();
    }

    /// Apply an owner-accepted value in either mode.
    pub fn set_value(&mut self, id: &str, value: PropertyValue, cx: &mut Context<Self>) -> bool {
        let Some(property) = self.property_mut(id) else { return false };
        let before = property.value.clone();
        property.value = value;
        property.normalize();
        let changed = before != property.value;
        if changed {
            cx.notify();
        }
        changed
    }

    fn property(&self, id: &str) -> Option<&Property> {
        self.groups.iter().flat_map(|g| &g.properties).find(|p| p.id == id)
    }

    fn property_mut(&mut self, id: &str) -> Option<&mut Property> {
        self.groups.iter_mut().flat_map(|g| &mut g.properties).find(|p| p.id == id)
    }

    fn visible_property_ids(&self) -> Vec<String> {
        if self.disabled {
            return Vec::new();
        }
        self.groups
            .iter()
            .filter(|group| group.expanded)
            .flat_map(|group| group.properties.iter())
            .filter(|property| !property.disabled)
            .map(|property| property.id.clone())
            .collect()
    }

    fn move_active(&mut self, delta: isize, cx: &mut Context<Self>) {
        let ids = self.visible_property_ids();
        if ids.is_empty() {
            return;
        }
        let current =
            self.active.as_ref().and_then(|id| ids.iter().position(|item| item == id)).unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(ids.len() as isize) as usize;
        self.active = Some(ids[next].clone());
        cx.notify();
    }

    fn reconcile_active(&mut self) {
        let visible = self.visible_property_ids();
        if self.active.as_ref().is_some_and(|id| !visible.contains(id)) {
            self.active = visible.first().cloned();
        }
    }

    fn focus_root(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(focus) = self.focus.as_ref() {
            window.focus(focus, cx);
        }
    }

    fn request(&mut self, id: &str, value: PropertyValue, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let Some(property) = self.property(id) else { return };
        if property.disabled || property.value == value {
            return;
        }
        if !self.controlled
            && let Some(property) = self.property_mut(id)
        {
            property.value = value.clone();
        }
        self.active = Some(id.to_owned());
        cx.emit(ValueChanged { property_id: id.into(), value });
        cx.notify();
    }

    fn reset_active(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.active.clone() else { return };
        if let Some(value) = self.property(&id).map(|p| p.default.clone()) {
            self.request(&id, value, cx);
        }
    }

    fn text_input(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || !self.editing_text {
            return;
        }
        let Some(id) = self.active.clone() else { return };
        let Some(Property { kind: PropertyKind::Text, value, disabled: false, .. }) =
            self.property(&id).cloned()
        else {
            return;
        };
        let text = match value {
            PropertyValue::Text(text) => text,
            PropertyValue::Mixed => self.text_draft.clone(),
            _ => return,
        };
        let key = event.keystroke.key.as_str();
        let next = match key {
            "backspace" => {
                let mut next = text;
                next.pop();
                PropertyValue::Text(next)
            }
            "space" => PropertyValue::Text(format!("{text} ")),
            "enter" | "escape" => {
                self.editing_text = false;
                cx.notify();
                return;
            }
            _ if key.chars().count() == 1 && !event.keystroke.modifiers.platform => {
                PropertyValue::Text(format!("{text}{key}"))
            }
            _ => return,
        };
        if let PropertyValue::Text(ref text) = next {
            self.text_draft = text.clone();
        }
        self.request(&id, next, cx);
    }

    fn toggle_active(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.active.clone() else { return };
        let value = self.property(&id).and_then(|p| match p.value {
            PropertyValue::Boolean(v) => Some(PropertyValue::Boolean(!v)),
            PropertyValue::Mixed if matches!(p.kind, PropertyKind::Boolean) => {
                Some(PropertyValue::Boolean(true))
            }
            _ => None,
        });
        if let Some(value) = value {
            self.request(&id, value, cx);
        }
    }

    fn next(&mut self, _: &NextProperty, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(1, cx);
        self.focus_root(window, cx);
    }
    fn previous(&mut self, _: &PreviousProperty, window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(-1, cx);
        self.focus_root(window, cx);
    }
    fn toggle(&mut self, _: &ToggleBoolean, _: &mut Window, cx: &mut Context<Self>) {
        self.toggle_active(cx);
    }
    fn reset(&mut self, _: &ResetProperty, _: &mut Window, cx: &mut Context<Self>) {
        self.reset_active(cx);
    }

    fn render_property(
        &self,
        group_index: usize,
        property_index: usize,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let property = &self.groups[group_index].properties[property_index];
        let id = property.id.clone();
        let label = property.label.clone();
        let value = property.value.clone();
        let kind = property.kind.clone();
        let default = property.default.clone();
        let disabled = self.disabled || property.disabled;
        let active = self.active.as_deref() == Some(&id);
        let is_mixed = value == PropertyValue::Mixed;
        let selector_id = id.clone();
        let reset_id = id.clone();
        let reset_default = default.clone();
        let reset = div()
            .id(format!("reset-{id}"))
            .debug_selector({
                let selector = format!("reset-{id}");
                move || selector.clone()
            })
            .role(gpui_pre::accesskit::Role::Button)
            .aria_label(format!("Reset {label}"))
            .tab_index(if disabled { -1 } else { 0 })
            .h(px(theme.controls.xsmall))
            .px(px(theme.spacing.small))
            .rounded(px(theme.radii.small))
            .text_size(px(theme.typography.caption))
            .text_color(theme.colors.text_muted)
            .hover(|this| this.bg(theme.colors.elevated_surface).text_color(theme.colors.text))
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled && value != default, |this| {
                this.on_click(cx.listener(move |this, _, _, cx| {
                    this.request(&reset_id, reset_default.clone(), cx)
                }))
            })
            .child("Reset");

        let mut row = div()
            .id(format!("property-{id}"))
            .debug_selector(move || format!("property-{selector_id}"))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(label.clone())
            .flex()
            .items_center()
            .gap(px(theme.spacing.small))
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.xsmall))
            .border_t(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .when(active, |el| el.bg(theme.colors.elevated_surface))
            .when(disabled, |el| el.text_color(theme.colors.disabled));

        let display = |value: String, label: String, role: gpui_pre::accesskit::Role| {
            div()
                .id(format!("display-{label}"))
                .role(role)
                .aria_label(label)
                .tab_index(if disabled { -1 } else { 0 })
                .min_w(px(theme.controls.small * 2.0))
                .h(px(theme.controls.xsmall))
                .px(px(theme.spacing.small))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(theme.radii.small))
                .border(px(theme.borders.hairline))
                .border_color(if active { theme.colors.focus } else { theme.colors.border })
                .bg(theme.colors.surface)
                .text_size(px(theme.typography.caption))
                .text_color(if is_mixed { theme.colors.text_muted } else { theme.colors.text })
                .child(value)
        };

        let mut editor = div().flex().items_center().gap(px(theme.spacing.xsmall));
        match (&kind, &value) {
            (PropertyKind::Number { step }, PropertyValue::Number(number)) => {
                let step = *step;
                let number = *number;
                let down_id = id.clone();
                let up_id = id.clone();
                editor = editor
                    .child(step_button(
                        "−",
                        format!("Decrease {label}"),
                        theme,
                        disabled,
                        cx.listener(move |this, _, _, cx| {
                            this.request(&down_id, PropertyValue::Number(number - step), cx)
                        }),
                    ))
                    .child(display(
                        format_number(number),
                        label.clone(),
                        gpui_pre::accesskit::Role::SpinButton,
                    ))
                    .child(step_button(
                        "+",
                        format!("Increase {label}"),
                        theme,
                        disabled,
                        cx.listener(move |this, _, _, cx| {
                            this.request(&up_id, PropertyValue::Number(number + step), cx)
                        }),
                    ));
            }
            (PropertyKind::Number { .. }, PropertyValue::Mixed) => {
                if let PropertyValue::Number(default_number) = &default {
                    let step = match &kind {
                        PropertyKind::Number { step } => *step,
                        _ => 1.0,
                    };
                    let default_number = *default_number;
                    let down_id = id.clone();
                    let up_id = id.clone();
                    editor = editor
                        .child(step_button(
                            "−",
                            format!("Decrease {label}"),
                            theme,
                            disabled,
                            cx.listener(move |this, _, _, cx| {
                                this.request(
                                    &down_id,
                                    PropertyValue::Number(default_number - step),
                                    cx,
                                )
                            }),
                        ))
                        .child(display(
                            "Mixed".into(),
                            label.clone(),
                            gpui_pre::accesskit::Role::SpinButton,
                        ))
                        .child(step_button(
                            "+",
                            format!("Increase {label}"),
                            theme,
                            disabled,
                            cx.listener(move |this, _, _, cx| {
                                this.request(
                                    &up_id,
                                    PropertyValue::Number(default_number + step),
                                    cx,
                                )
                            }),
                        ));
                }
            }
            (PropertyKind::Text, PropertyValue::Text(text)) => {
                let text_id = id.clone();
                let initial_text = text.clone();
                editor = editor.child(
                    div()
                        .id(format!("text-{id}"))
                        .debug_selector({
                            let selector = format!("text-{id}");
                            move || selector.clone()
                        })
                        .role(gpui_pre::accesskit::Role::TextInput)
                        .aria_label(label.clone())
                        .aria_value(text.clone())
                        .tab_index(if disabled { -1 } else { 0 })
                        .min_w(px(theme.controls.small * 3.0))
                        .h(px(theme.controls.xsmall))
                        .px(px(theme.spacing.small))
                        .flex()
                        .items_center()
                        .rounded(px(theme.radii.small))
                        .border(px(theme.borders.hairline))
                        .border_color(if active { theme.colors.focus } else { theme.colors.border })
                        .bg(theme.colors.surface)
                        .text_size(px(theme.typography.caption))
                        .text_color(theme.colors.text)
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.active = Some(text_id.clone());
                                this.editing_text = true;
                                this.text_draft = initial_text.clone();
                                cx.notify();
                            }))
                        })
                        .child(if self.editing_text && active {
                            format!("{text}▏")
                        } else {
                            text.clone()
                        }),
                );
            }
            (PropertyKind::Text, PropertyValue::Mixed) => {
                let text_id = id.clone();
                editor = editor.child(
                    div()
                        .id(format!("text-{id}"))
                        .debug_selector({
                            let selector = format!("text-{id}");
                            move || selector.clone()
                        })
                        .role(gpui_pre::accesskit::Role::TextInput)
                        .aria_label(label.clone())
                        .aria_value(if self.editing_text && active {
                            self.text_draft.clone()
                        } else {
                            String::new()
                        })
                        .tab_index(if disabled { -1 } else { 0 })
                        .min_w(px(theme.controls.small * 3.0))
                        .h(px(theme.controls.xsmall))
                        .px(px(theme.spacing.small))
                        .flex()
                        .items_center()
                        .rounded(px(theme.radii.small))
                        .border(px(theme.borders.hairline))
                        .border_color(if active { theme.colors.focus } else { theme.colors.border })
                        .bg(theme.colors.surface)
                        .text_size(px(theme.typography.caption))
                        .text_color(theme.colors.text_muted)
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.active = Some(text_id.clone());
                                this.editing_text = true;
                                this.text_draft.clear();
                                cx.notify();
                            }))
                        })
                        .child(if self.editing_text && active {
                            format!("{}▏", self.text_draft)
                        } else {
                            "Mixed".into()
                        }),
                );
            }
            (PropertyKind::Boolean, PropertyValue::Boolean(value)) => {
                let next = !value;
                let bool_id = id.clone();
                let bool_selector = format!("boolean-{id}");
                editor = editor.child(
                    div()
                        .id(format!("boolean-{id}"))
                        .debug_selector(move || bool_selector.clone())
                        .role(gpui_pre::accesskit::Role::CheckBox)
                        .aria_label(label.clone())
                        .aria_toggled(gpui_pre::accesskit::Toggled::from(*value))
                        .tab_index(if disabled { -1 } else { 0 })
                        .w(px(theme.controls.small))
                        .h(px(theme.controls.xsmall * 0.68))
                        .rounded(px(theme.radii.pill))
                        .bg(if *value { theme.colors.accent } else { theme.colors.border })
                        .flex()
                        .items_center()
                        .when(*value, |el| el.justify_end())
                        .when(!*value, |el| el.justify_start())
                        .p(px(theme.spacing.xsmall))
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.request(&bool_id, PropertyValue::Boolean(next), cx)
                            }))
                        })
                        .child(
                            div()
                                .size(px(theme.spacing.small))
                                .rounded(px(theme.radii.pill))
                                .bg(theme.colors.surface),
                        ),
                );
            }
            (PropertyKind::Boolean, PropertyValue::Mixed) => {
                let bool_id = id.clone();
                let bool_selector = format!("boolean-{id}");
                editor = editor.child(
                    div()
                        .id(format!("boolean-{id}"))
                        .debug_selector(move || bool_selector.clone())
                        .role(gpui_pre::accesskit::Role::CheckBox)
                        .aria_label(label.clone())
                        .aria_toggled(gpui_pre::accesskit::Toggled::Mixed)
                        .aria_description("Mixed value")
                        .tab_index(if disabled { -1 } else { 0 })
                        .w(px(theme.controls.small))
                        .h(px(theme.controls.xsmall * 0.68))
                        .rounded(px(theme.radii.pill))
                        .bg(theme.colors.border)
                        .flex()
                        .items_center()
                        .justify_start()
                        .p(px(theme.spacing.xsmall))
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.request(&bool_id, PropertyValue::Boolean(true), cx)
                            }))
                        })
                        .child(
                            div()
                                .size(px(theme.spacing.small))
                                .rounded(px(theme.radii.pill))
                                .bg(theme.colors.surface),
                        ),
                );
            }
            (PropertyKind::Enum { options }, PropertyValue::Enum(current)) => {
                let next = cycle_option(options, current);
                let enum_id = id.clone();
                editor = editor.child(
                    div()
                        .id(format!("enum-{id}"))
                        .debug_selector({
                            let selector = format!("enum-{id}");
                            move || selector.clone()
                        })
                        .role(gpui_pre::accesskit::Role::ComboBox)
                        .aria_label(label.clone())
                        .tab_index(if disabled { -1 } else { 0 })
                        .min_w(px(theme.controls.small * 2.75))
                        .h(px(theme.controls.xsmall))
                        .px(px(theme.spacing.small))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(theme.spacing.small))
                        .rounded(px(theme.radii.small))
                        .border(px(theme.borders.hairline))
                        .border_color(theme.colors.border)
                        .bg(theme.colors.surface)
                        .text_size(px(theme.typography.caption))
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.request(&enum_id, PropertyValue::Enum(next.clone()), cx)
                            }))
                        })
                        .child(current.clone())
                        .child("⌄"),
                );
            }
            (PropertyKind::Enum { options }, PropertyValue::Mixed) => {
                let first = options.first().cloned().unwrap_or_default();
                let enum_id = id.clone();
                editor = editor.child(
                    div()
                        .id(format!("enum-{id}"))
                        .debug_selector({
                            let selector = format!("enum-{id}");
                            move || selector.clone()
                        })
                        .role(gpui_pre::accesskit::Role::ComboBox)
                        .aria_label(label.clone())
                        .aria_value("Mixed")
                        .tab_index(if disabled { -1 } else { 0 })
                        .min_w(px(theme.controls.small * 2.75))
                        .h(px(theme.controls.xsmall))
                        .px(px(theme.spacing.small))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(theme.spacing.small))
                        .rounded(px(theme.radii.small))
                        .border(px(theme.borders.hairline))
                        .border_color(theme.colors.border)
                        .bg(theme.colors.surface)
                        .text_size(px(theme.typography.caption))
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.request(&enum_id, PropertyValue::Enum(first.clone()), cx)
                            }))
                        })
                        .child("Mixed")
                        .child("⌄"),
                );
            }
            (PropertyKind::Color, PropertyValue::Color(color)) => {
                let parsed = parse_hex_rgb(color);
                editor = editor.child(
                    div().flex().items_center().gap(px(theme.spacing.xsmall)).child(
                        div()
                            .size(px(theme.spacing.medium))
                            .rounded(px(theme.radii.small))
                            .bg(rgba_from_hex(color)),
                    ),
                );
                for (channel, value) in [parsed.0, parsed.1, parsed.2].into_iter().enumerate() {
                    let down_id = id.clone();
                    let up_id = id.clone();
                    let down_color = alter_hex_channel(color, channel, -1);
                    let up_color = alter_hex_channel(color, channel, 1);
                    editor = editor.child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.xsmall))
                            .child(step_button(
                                "−",
                                format!(
                                    "Decrease {label} {} channel",
                                    ["red", "green", "blue"][channel]
                                ),
                                theme,
                                disabled,
                                cx.listener(move |this, _, _, cx| {
                                    this.request(
                                        &down_id,
                                        PropertyValue::Color(down_color.clone()),
                                        cx,
                                    )
                                }),
                            ))
                            .child(
                                div()
                                    .text_size(px(theme.typography.caption))
                                    .text_color(theme.colors.text_muted)
                                    .child(format!("{} {value}", ["R", "G", "B"][channel])),
                            )
                            .child(step_button(
                                "+",
                                format!(
                                    "Increase {label} {} channel",
                                    ["red", "green", "blue"][channel]
                                ),
                                theme,
                                disabled,
                                cx.listener(move |this, _, _, cx| {
                                    this.request(&up_id, PropertyValue::Color(up_color.clone()), cx)
                                }),
                            )),
                    );
                }
            }
            (PropertyKind::Color, PropertyValue::Mixed) => {
                if let PropertyValue::Color(default_color) = &default {
                    editor = editor.child(
                        div()
                            .size(px(theme.spacing.medium))
                            .rounded(px(theme.radii.small))
                            .bg(rgba_from_hex(default_color)),
                    );
                    let channels = parse_hex_rgb(default_color);
                    for (channel, value) in
                        [channels.0, channels.1, channels.2].into_iter().enumerate()
                    {
                        let down_id = id.clone();
                        let up_id = id.clone();
                        let down_color = alter_hex_channel(default_color, channel, -1);
                        let up_color = alter_hex_channel(default_color, channel, 1);
                        editor = editor.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(theme.spacing.xsmall))
                                .child(
                                    div()
                                        .text_size(px(theme.typography.caption))
                                        .text_color(theme.colors.text_muted)
                                        .child(format!("{} {value}", ["R", "G", "B"][channel])),
                                )
                                .child(step_button(
                                    "−",
                                    format!(
                                        "Decrease {label} {} channel",
                                        ["red", "green", "blue"][channel]
                                    ),
                                    theme,
                                    disabled,
                                    cx.listener(move |this, _, _, cx| {
                                        this.request(
                                            &down_id,
                                            PropertyValue::Color(down_color.clone()),
                                            cx,
                                        )
                                    }),
                                ))
                                .child(step_button(
                                    "+",
                                    format!(
                                        "Increase {label} {} channel",
                                        ["red", "green", "blue"][channel]
                                    ),
                                    theme,
                                    disabled,
                                    cx.listener(move |this, _, _, cx| {
                                        this.request(
                                            &up_id,
                                            PropertyValue::Color(up_color.clone()),
                                            cx,
                                        )
                                    }),
                                )),
                        );
                    }
                } else {
                    editor = editor.child(display(
                        "Mixed".into(),
                        label.clone(),
                        gpui_pre::accesskit::Role::Button,
                    ));
                }
            }
            (PropertyKind::Vector { step, dimensions }, PropertyValue::Vector(values)) => {
                let step = *step;
                let dimensions = *dimensions;
                for (index, number) in values.iter().copied().take(dimensions).enumerate() {
                    let down_id = id.clone();
                    let up_id = id.clone();
                    let mut down = values.clone();
                    let mut up = values.clone();
                    down[index] -= step;
                    up[index] += step;
                    editor = editor
                        .child(step_button(
                            "−",
                            format!("Decrease {label} component {}", index + 1),
                            theme,
                            disabled,
                            cx.listener(move |this, _, _, cx| {
                                this.request(&down_id, PropertyValue::Vector(down.clone()), cx)
                            }),
                        ))
                        .child(display(
                            format_number(number),
                            format!("{label} component {}", index + 1),
                            gpui_pre::accesskit::Role::SpinButton,
                        ))
                        .child(step_button(
                            "+",
                            format!("Increase {label} component {}", index + 1),
                            theme,
                            disabled,
                            cx.listener(move |this, _, _, cx| {
                                this.request(&up_id, PropertyValue::Vector(up.clone()), cx)
                            }),
                        ));
                }
            }
            (PropertyKind::Vector { .. }, PropertyValue::Mixed) => {
                if let PropertyValue::Vector(default_values) = &default {
                    let (step, dimensions) = match &kind {
                        PropertyKind::Vector { step, dimensions } => (*step, *dimensions),
                        _ => unreachable!(),
                    };
                    for (index, number) in
                        default_values.iter().copied().take(dimensions).enumerate()
                    {
                        let down_id = id.clone();
                        let up_id = id.clone();
                        let mut down = default_values.clone();
                        let mut up = default_values.clone();
                        down[index] -= step;
                        up[index] += step;
                        editor = editor
                            .child(step_button(
                                "−",
                                format!("Decrease {label} component {}", index + 1),
                                theme,
                                disabled,
                                cx.listener(move |this, _, _, cx| {
                                    this.request(&down_id, PropertyValue::Vector(down.clone()), cx)
                                }),
                            ))
                            .child(display(
                                format_number(number),
                                format!("{label} component {}", index + 1),
                                gpui_pre::accesskit::Role::SpinButton,
                            ))
                            .child(step_button(
                                "+",
                                format!("Increase {label} component {}", index + 1),
                                theme,
                                disabled,
                                cx.listener(move |this, _, _, cx| {
                                    this.request(&up_id, PropertyValue::Vector(up.clone()), cx)
                                }),
                            ));
                    }
                }
            }
            _ => {}
        }

        row = row.child(
            div()
                .flex_1()
                .min_w(px(theme.controls.small * 2.375))
                .text_size(px(theme.typography.caption))
                .text_color(if disabled { theme.colors.disabled } else { theme.colors.text_muted })
                .child(label),
        );
        if is_mixed {
            row = row.child(
                div()
                    .id(format!("mixed-{id}"))
                    .aria_label("Mixed value")
                    .text_size(px(theme.typography.caption))
                    .text_color(theme.colors.text_muted)
                    .child("Mixed"),
            );
        }
        row.child(editor).child(reset)
    }
}

impl Focusable for PropertyInspector {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus.clone().expect("focus initialized")
    }
}

impl Render for PropertyInspector {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        let disabled = self.disabled;
        let mut root = div()
            .id("mkit-property-inspector")
            .tab_index(if disabled { -1 } else { 0 })
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::toggle))
            .on_action(cx.listener(Self::reset))
            .on_key_down(cx.listener(Self::text_input))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label("Property inspector")
            .when(disabled, |el| {
                el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .w_full()
            .flex()
            .flex_col()
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .bg(theme.colors.surface);

        for group_index in 0..self.groups.len() {
            let label = self.groups[group_index].label.clone();
            let expanded = self.groups[group_index].expanded;
            let group_id = self.groups[group_index].id.clone();
            let key_group_id = group_id.clone();
            root = root.child(
                div()
                    .id(format!("group-{group_id}"))
                    .debug_selector({
                        let selector = format!("group-{group_id}");
                        move || selector.clone()
                    })
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(label.clone())
                    .aria_expanded(expanded)
                    .tab_index(if disabled { -1 } else { 0 })
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                        if !matches!(event.keystroke.key.as_str(), "enter" | "space")
                            || this.disabled
                        {
                            return;
                        }
                        if let Some(group) = this.groups.iter_mut().find(|g| g.id == key_group_id) {
                            group.expanded = !group.expanded;
                            this.reconcile_active();
                            cx.notify();
                        }
                    }))
                    .w_full()
                    .h(px(theme.controls.small))
                    .px(px(theme.spacing.medium))
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(theme.colors.elevated_surface)
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.body_emphasis))
                    .when(group_index == 0, |el| el.rounded_t(px(theme.radii.medium)))
                    .when(!disabled, |el| {
                        el.on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(group) = this.groups.iter_mut().find(|g| g.id == group_id) {
                                group.expanded = !group.expanded;
                                this.reconcile_active();
                                cx.notify();
                            }
                        }))
                    })
                    .child(label)
                    .child(if expanded { "⌄" } else { "›" }),
            );
            if expanded {
                for property_index in 0..self.groups[group_index].properties.len() {
                    root = root.child(self.render_property(group_index, property_index, theme, cx));
                }
            }
        }
        root
    }
}

fn step_button(
    text: &'static str,
    label: String,
    theme: Theme,
    disabled: bool,
    click: impl Fn(&gpui_pre::ClickEvent, &mut Window, &mut gpui_pre::App) + 'static,
) -> impl IntoElement {
    let selector_label = label.clone();
    div()
        .id(format!("control-{label}"))
        .debug_selector(move || format!("control-{selector_label}"))
        .role(gpui_pre::accesskit::Role::Button)
        .aria_label(label)
        .tab_index(if disabled { -1 } else { 0 })
        .w(px(theme.controls.xsmall * 0.72))
        .h(px(theme.controls.xsmall))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme.radii.small))
        .border(px(theme.borders.hairline))
        .border_color(theme.colors.border)
        .bg(theme.colors.surface)
        .text_color(theme.colors.text)
        .when(disabled, |el| el.opacity(0.5))
        .when(!disabled, |el| el.on_click(click))
        .child(text)
}

fn cycle_option(options: &[String], current: &str) -> String {
    if options.is_empty() {
        return current.to_owned();
    }
    let index = options.iter().position(|item| item == current).unwrap_or(0);
    options[(index + 1) % options.len()].clone()
}

fn parse_hex_rgb(value: &str) -> (u8, u8, u8) {
    let hex = value.trim_start_matches('#');
    if hex.len() == 6
        && let Ok(rgb) = u32::from_str_radix(hex, 16)
    {
        return ((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    }
    (128, 128, 128)
}

fn rgba_from_hex(value: &str) -> gpui_pre::Rgba {
    let (red, green, blue) = parse_hex_rgb(value);
    gpui_pre::Rgba {
        r: red as f32 / 255.0,
        g: green as f32 / 255.0,
        b: blue as f32 / 255.0,
        a: 1.0,
    }
}

fn alter_hex_channel(value: &str, channel: usize, amount: i16) -> String {
    let parsed = parse_hex_rgb(value);
    let mut rgb = [parsed.0, parsed.1, parsed.2];
    if let Some(component) = rgb.get_mut(channel) {
        *component = (*component as i16 + amount).clamp(0, 255) as u8;
    }
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 { format!("{value:.0}") } else { format!("{value:.2}") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext};

    fn fixture() -> Vec<PropertyGroup> {
        vec![PropertyGroup::new(
            "transform",
            "Transform",
            vec![Property::new(
                "visible",
                "Visible",
                PropertyKind::Boolean,
                PropertyValue::Boolean(false),
                PropertyValue::Boolean(true),
            )],
        )]
    }

    fn text_fixture() -> Vec<PropertyGroup> {
        vec![PropertyGroup::new(
            "metadata",
            "Metadata",
            vec![Property::new(
                "title",
                "Title",
                PropertyKind::Text,
                PropertyValue::Text("foo".into()),
                PropertyValue::Text("".into()),
            )],
        )]
    }

    #[test]
    fn mixed_values_preserve_declared_editor_kind_and_reset_default() {
        let property = Property::new(
            "opacity",
            "Opacity",
            PropertyKind::Number { step: 0.1 },
            PropertyValue::Mixed,
            PropertyValue::Number(1.0),
        );
        assert_eq!(property.value, PropertyValue::Mixed);
        assert_eq!(property.default, PropertyValue::Number(1.0));
    }

    #[test]
    fn invalid_values_fall_back_to_the_typed_default() {
        let property = Property::new(
            "enabled",
            "Enabled",
            PropertyKind::Boolean,
            PropertyValue::Text("bad".into()),
            PropertyValue::Boolean(true),
        );
        assert_eq!(property.value, PropertyValue::Boolean(true));
    }

    #[test]
    fn enum_cycles_in_stable_declared_order_and_handles_empty_options() {
        let options = vec!["Solid".to_string(), "Dash".to_string(), "Dot".to_string()];
        assert_eq!(cycle_option(&options, "Dash"), "Dot");
        assert_eq!(cycle_option(&options, "Dot"), "Solid");
        assert_eq!(cycle_option(&[], "Missing"), "Missing");
    }

    #[test]
    fn number_format_omits_decimal_for_whole_values() {
        assert_eq!(format_number(4.0), "4");
        assert_eq!(format_number(0.125), "0.12");
    }

    #[test]
    fn color_channel_control_clamps_and_returns_normalized_hex() {
        assert_eq!(alter_hex_channel("#00FF10", 0, -1), "#00FF10");
        assert_eq!(alter_hex_channel("#00FF10", 1, 1), "#00FF10");
        assert_eq!(alter_hex_channel("#00FF10", 2, 1), "#00FF11");
        assert_eq!(alter_hex_channel("bad", 0, 1), "#818080");
    }

    #[gpui_pre::test]
    fn gpui_group_collapse_and_boolean_editor_work_by_pointer(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (inspector, visual) = cx.add_window_view(|_, _| PropertyInspector::new(fixture()));
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let boolean = visual.debug_bounds("boolean-visible").expect("boolean editor is rendered");
        visual.simulate_click(boolean.center(), Modifiers::default());
        assert_eq!(
            inspector.read_with(visual, |view, _| view.groups[0].properties[0].value.clone()),
            PropertyValue::Boolean(true)
        );

        visual.update(|window, cx| window.draw(cx).clear(cx));
        let heading = visual.debug_bounds("group-transform").expect("group heading is rendered");
        visual.simulate_click(heading.center(), Modifiers::default());
        assert!(!inspector.read_with(visual, |view, _| view.groups[0].expanded));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("boolean-visible").is_none());
        let heading =
            visual.debug_bounds("group-transform").expect("group heading remains rendered");
        visual.simulate_click(heading.center(), Modifiers::default());
        visual.simulate_keystrokes("space");
        assert!(!inspector.read_with(visual, |view, _| view.groups[0].expanded));
    }

    #[gpui_pre::test]
    fn gpui_text_editor_emits_basic_keyboard_edits(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (inspector, visual) = cx.add_window_view(|_, _| PropertyInspector::new(text_fixture()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let field = visual.debug_bounds("text-title").expect("text editor is rendered");
        visual.simulate_click(field.center(), Modifiers::default());
        visual.simulate_keystrokes("x space backspace enter");
        assert_eq!(
            inspector.read_with(visual, |view, _| view.groups[0].properties[0].value.clone()),
            PropertyValue::Text("foox".into())
        );
    }

    #[gpui_pre::test]
    fn gpui_colour_channel_button_emits_a_new_hex_value(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let groups = vec![PropertyGroup::new(
            "appearance",
            "Appearance",
            vec![Property::new(
                "tint",
                "Tint",
                PropertyKind::Color,
                PropertyValue::Color("#112233".into()),
                PropertyValue::Color("#000000".into()),
            )],
        )];
        let (inspector, visual) = cx.add_window_view(|_, _| PropertyInspector::new(groups));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let red_plus = visual
            .debug_bounds("control-Increase Tint red channel")
            .expect("colour channel increment is rendered");
        visual.simulate_click(red_plus.center(), Modifiers::default());
        assert_eq!(
            inspector.read_with(visual, |view, _| view.groups[0].properties[0].value.clone()),
            PropertyValue::Color("#122233".into())
        );
    }

    #[gpui_pre::test]
    fn gpui_enum_cycles_and_reset_restores_default(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let groups = vec![PropertyGroup::new(
            "appearance",
            "Appearance",
            vec![Property::new(
                "style",
                "Style",
                PropertyKind::Enum { options: vec!["Solid".into(), "Dashed".into()] },
                PropertyValue::Enum("Solid".into()),
                PropertyValue::Enum("Solid".into()),
            )],
        )];
        let (inspector, visual) = cx.add_window_view(|_, _| PropertyInspector::new(groups));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let combo = visual.debug_bounds("enum-style").expect("enum editor is rendered");
        visual.simulate_click(combo.center(), Modifiers::default());
        assert_eq!(
            inspector.read_with(visual, |view, _| view.groups[0].properties[0].value.clone()),
            PropertyValue::Enum("Dashed".into())
        );
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let reset = visual.debug_bounds("reset-style").expect("reset action is rendered");
        visual.simulate_click(reset.center(), Modifiers::default());
        assert_eq!(
            inspector.read_with(visual, |view, _| view.groups[0].properties[0].value.clone()),
            PropertyValue::Enum("Solid".into())
        );
    }

    #[gpui_pre::test]
    fn arrow_navigation_skips_disabled_and_collapsed_rows_and_keeps_root_focus(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let groups = vec![
            PropertyGroup::new(
                "transform",
                "Transform",
                vec![
                    Property::new(
                        "visible",
                        "Visible",
                        PropertyKind::Boolean,
                        PropertyValue::Boolean(true),
                        PropertyValue::Boolean(false),
                    ),
                    Property::new(
                        "locked",
                        "Locked",
                        PropertyKind::Boolean,
                        PropertyValue::Boolean(false),
                        PropertyValue::Boolean(false),
                    )
                    .disabled(true),
                    Property::new(
                        "opacity",
                        "Opacity",
                        PropertyKind::Number { step: 0.1 },
                        PropertyValue::Number(0.8),
                        PropertyValue::Number(1.0),
                    ),
                ],
            ),
            PropertyGroup::new(
                "appearance",
                "Appearance",
                vec![Property::new(
                    "name",
                    "Name",
                    PropertyKind::Text,
                    PropertyValue::Text("Card".into()),
                    PropertyValue::Text("Layer".into()),
                )],
            )
            .collapsed(),
        ];
        let (inspector, visual) = cx.add_window_view(|_, _| PropertyInspector::new(groups));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| inspector.focus_handle(cx).focus(window, cx));

        visual.simulate_keystrokes("down");
        assert_eq!(
            inspector.read_with(visual, |view, _| view.active.clone()),
            Some("opacity".to_owned())
        );
        visual.simulate_keystrokes("down");
        assert_eq!(
            inspector.read_with(visual, |view, _| view.active.clone()),
            Some("visible".to_owned())
        );
        visual.simulate_keystrokes("up");
        assert_eq!(
            inspector.read_with(visual, |view, _| view.active.clone()),
            Some("opacity".to_owned())
        );
        let focus = inspector.read_with(visual, |view, cx| view.focus_handle(cx));
        assert!(visual.update(|window, _| focus.is_focused(window)));
        assert!(visual.debug_bounds("property-name").is_none());
    }
}
