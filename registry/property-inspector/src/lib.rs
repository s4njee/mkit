//! Typed, theme-driven property panel for pro applications.
extern crate gpui_pre as gpui;

#[cfg(feature = "mkit-mirror")]
use crate::disclosure::{
    DisclosurePanel, DisclosureTrigger, default_key_bindings as disclosure_key_bindings,
};
use gpui_pre::{
    Context, Div, EventEmitter, FocusHandle, Focusable, FontWeight, InteractiveElement, IntoElement,
    KeyBinding, KeyDownEvent, PathBuilder, Render, Rgba, Stateful, Window, actions, canvas, div,
    point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};
#[cfg(not(feature = "mkit-mirror"))]
use mkit_registry_disclosure::{
    DisclosurePanel, DisclosureTrigger, default_key_bindings as disclosure_key_bindings,
};

pub const KEY_CONTEXT: &str = "PropertyInspector";
actions!(property_inspector, [NextProperty, PreviousProperty, ToggleBoolean, ResetProperty]);

/// Inspector bindings followed by the shared Disclosure bindings used by group headers.
pub fn default_key_bindings() -> [KeyBinding; 6] {
    let [disclosure_space, disclosure_enter] = disclosure_key_bindings();
    [
        KeyBinding::new("down", NextProperty, Some(KEY_CONTEXT)),
        KeyBinding::new("up", PreviousProperty, Some(KEY_CONTEXT)),
        KeyBinding::new("space", ToggleBoolean, Some(KEY_CONTEXT)),
        KeyBinding::new("r", ResetProperty, Some(KEY_CONTEXT)),
        disclosure_space,
        disclosure_enter,
    ]
}

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    card: Rgba,
    card_border: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    /// Pointer-hover fill for rows; `None` outlines the row instead (high contrast).
    hover_bg: Option<Rgba>,
    hover_border: Rgba,
    /// Weight of the shadcn "muted" mix, reused for ghost hovers over a row fill.
    muted_weight: Option<f32>,
    text: Rgba,
    label: Rgba,
    placeholder: Rgba,
    field_bg: Rgba,
    field_border: Rgba,
    select_bg: Rgba,
    button_bg: Rgba,
    button_border: Rgba,
    button_hover_bg: Option<Rgba>,
    button_hover_border: Option<Rgba>,
    icon: Rgba,
    track_off: Rgba,
    track_on: Rgba,
    track_border_off: Rgba,
    track_border_on: Rgba,
    thumb_off: Rgba,
    thumb_on: Rgba,
    reset_idle: Rgba,
    /// Mixed badge fill; `None` fills with the muted mix over the row fill.
    badge_bg: Option<Rgba>,
    badge_text: Rgba,
    caret: Rgba,
    focus: Rgba,
    ring: Rgba,
    shadow: ShadowToken,
    high_contrast: bool,
}

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}

/// shadcn's disabled `opacity: .5` as one layer: composite opaque over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    if color.a == 0. { color } else { mix(composite(color, base), base, 0.5) }
}

fn look(t: &Theme) -> Look {
    let c = t.colors;
    let transparent = c.background.opacity(0.);
    if t.name == "high-contrast" {
        return Look {
            card: c.background,
            card_border: c.border,
            active_bg: c.accent,
            active_text: c.accent_text,
            hover_bg: None,
            hover_border: c.border,
            muted_weight: None,
            text: c.text,
            label: c.text_muted,
            placeholder: c.text_muted,
            field_bg: c.background,
            field_border: c.border,
            select_bg: c.background,
            button_bg: c.background,
            button_border: c.border,
            button_hover_bg: None,
            button_hover_border: Some(c.accent),
            icon: c.text,
            track_off: c.background,
            track_on: c.accent,
            track_border_off: c.border,
            track_border_on: c.accent,
            thumb_off: c.text,
            thumb_on: c.accent_text,
            reset_idle: c.disabled,
            badge_bg: Some(c.text),
            badge_text: c.background,
            caret: c.accent,
            focus: c.focus,
            ring: c.focus,
            shadow: t.shadows.none,
            high_contrast: true,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    let weight = if dark { 0.12 } else { 0.04 };
    // shadcn "accent"/"muted": text mixed into the background.
    let muted = mix(c.text, c.background, weight);
    let card = c.surface;
    let card_border = if dark { mix(c.text, card, 0.1) } else { c.border };
    // shadcn `--input`, and TextField's `dark:bg-input/30` fill.
    let field_bg = if dark { mix(c.text, c.background, 0.045) } else { c.background };
    let input = |fill: Rgba| if dark { mix(c.text, fill, 0.15) } else { c.border };
    Look {
        card,
        card_border,
        active_bg: muted,
        active_text: c.text,
        hover_bg: Some(muted),
        hover_border: transparent,
        muted_weight: Some(weight),
        text: c.text,
        label: c.text_muted,
        placeholder: c.text_muted,
        field_bg,
        field_border: input(field_bg),
        select_bg: c.background,
        button_bg: c.background,
        button_border: if dark { mix(c.text, c.background, 0.1) } else { c.border },
        button_hover_bg: Some(muted),
        button_hover_border: None,
        icon: c.text_muted,
        track_off: input(c.background),
        track_on: c.accent,
        track_border_off: transparent,
        track_border_on: transparent,
        thumb_off: if dark { c.text } else { c.background },
        thumb_on: c.background,
        reset_idle: dim(c.text_muted, card),
        badge_bg: None,
        badge_text: c.text_muted,
        caret: c.accent,
        focus: c.focus,
        ring: c.focus.opacity(0.5),
        shadow: t.shadows.small,
        high_contrast: false,
    }
}

impl Look {
    /// Disabled colours: 50% over the card fill, or solid `disabled` in high contrast.
    fn disabled(self, t: &Theme) -> Self {
        let c = t.colors;
        if self.high_contrast {
            let off = c.disabled;
            return Self {
                text: off,
                label: off,
                placeholder: off,
                field_border: off,
                button_border: off,
                button_hover_border: None,
                icon: off,
                track_on: dim(self.track_on, c.background),
                track_border_off: off,
                track_border_on: dim(self.track_border_on, c.background),
                thumb_off: off,
                reset_idle: off,
                badge_bg: Some(off),
                hover_border: c.background.opacity(0.),
                ..self
            };
        }
        let base = self.card;
        let d = |color: Rgba| dim(color, base);
        Self {
            text: d(self.text),
            label: d(self.label),
            placeholder: d(self.placeholder),
            field_bg: d(self.field_bg),
            field_border: d(self.field_border),
            select_bg: d(self.select_bg),
            button_bg: d(self.button_bg),
            button_border: d(self.button_border),
            button_hover_bg: None,
            icon: d(self.icon),
            track_off: d(self.track_off),
            track_on: d(self.track_on),
            thumb_off: d(self.thumb_off),
            thumb_on: d(self.thumb_on),
            reset_idle: d(self.reset_idle),
            badge_text: d(self.badge_text),
            hover_bg: None,
            shadow: ShadowToken { color: self.shadow.color.opacity(self.shadow.color.a * 0.5), ..self.shadow },
            ..self
        }
    }
}

fn box_shadow(shadow: ShadowToken) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}

/// shadcn/ui focus ring width, drawn outside focused editors and buttons.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}

/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Each
/// polyline is a list of points on a 24-unit grid; the stroke is 2 units, Lucide's default.
fn icon(size: f32, lines: &'static [&'static [(f32, f32)]], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let mut path = PathBuilder::stroke(unit * 2.0);
            for line in lines {
                for (i, (x, y)) in line.iter().enumerate() {
                    let at = bounds.origin + point(unit * *x, unit * *y);
                    if i == 0 { path.move_to(at) } else { path.line_to(at) }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `minus`.
const MINUS: &[&[(f32, f32)]] = &[&[(5., 12.), (19., 12.)]];
/// Lucide `plus`.
const PLUS: &[&[(f32, f32)]] = &[&[(5., 12.), (19., 12.)], &[(12., 5.), (12., 19.)]];
/// Lucide `chevron-down`.
const CHEVRON_DOWN: &[&[(f32, f32)]] = &[&[(6., 9.), (12., 15.), (18., 9.)]];

/// TextField/Select chrome shared by the inline editors: 28px tall, input border, resting
/// shadow, and a focus border plus ring for keyboard focus.
fn field(el: Stateful<Div>, look: &Look, theme: &Theme, fill: Rgba) -> Stateful<Div> {
    let (focus, ring) = (look.focus, look.ring);
    el.h(px(theme.controls.xsmall))
        .px(px(theme.spacing.small))
        .flex()
        .items_center()
        .rounded(px(theme.radii.medium))
        .border(px(theme.borders.regular))
        .border_color(look.field_border)
        .bg(fill)
        .shadow(vec![box_shadow(look.shadow)])
        .text_size(px(theme.typography.body))
        .focus_visible(move |s| s.border_color(focus).shadow(vec![focus_ring(ring)]))
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

    fn toggle_group(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if let Some(group) = self.groups.iter_mut().find(|group| group.id == id) {
            group.expanded = !group.expanded;
            self.reconcile_active();
            cx.notify();
        }
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
        focused: bool,
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
        let base = look(&theme);
        let look = if disabled { base.disabled(&theme) } else { base };
        // Parts drawn straight on the row take the active row's text colour (high contrast only
        // changes it, as Tree does).
        let row_fill = if active { look.active_bg } else { look.card };
        let on_row = |color: Rgba| if active && look.high_contrast { look.active_text } else { color };
        let transparent = look.card.opacity(0.);
        let selector_id = id.clone();
        let reset_id = id.clone();
        let reset_default = default.clone();
        let reset_color = if value != default {
            on_row(look.text)
        } else if active && look.high_contrast {
            dim(look.active_text, look.active_bg)
        } else if look.high_contrast {
            look.reset_idle
        } else {
            dim(theme.colors.text_muted, row_fill)
        };
        let reset_color = if disabled && !look.high_contrast { look.reset_idle } else { reset_color };
        let ghost_hover_bg = look.muted_weight.map(|weight| mix(theme.colors.text, row_fill, weight));
        let ghost_hover_border = on_row(theme.colors.accent);
        let (focus, ring) = (look.focus, look.ring);
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
            .flex()
            .items_center()
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(transparent)
            .text_size(px(theme.typography.body))
            .font_weight(FontWeight::MEDIUM)
            .text_color(reset_color)
            .when(!disabled, |this| {
                this.hover(move |style| match ghost_hover_bg {
                    Some(fill) => style.bg(fill),
                    None => style.border_color(ghost_hover_border),
                })
            })
            .focus_visible(move |s| {
                s.bg(row_fill).border_color(focus).shadow(vec![focus_ring(ring)])
            })
            .when(!disabled && value != default, |this| {
                this.on_click(cx.listener(move |this, _, _, cx| {
                    this.request(&reset_id, reset_default.clone(), cx)
                }))
            })
            .child("Reset");

        let hover_bg = look.hover_bg;
        let hover_border = look.hover_border;
        let mut row = div()
            .id(format!("property-{id}"))
            .debug_selector(move || format!("property-{selector_id}"))
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(label.clone())
            .flex()
            .items_center()
            .gap(px(theme.spacing.small))
            .px(px(theme.spacing.small))
            // The hairline border replaces the old row separator, so the row pitch is unchanged.
            .pt(px(theme.spacing.xsmall))
            .pb(px(theme.spacing.xsmall - theme.borders.hairline))
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(if focused && active { look.focus } else { transparent })
            .text_color(on_row(look.text))
            .when(active, |el| el.bg(look.active_bg))
            .when(!active && !disabled, |el| {
                el.hover(move |style| match hover_bg {
                    Some(fill) => style.bg(fill),
                    None => style.border_color(hover_border),
                })
            });

        let display = |value: String, label: String, role: gpui_pre::accesskit::Role| {
            field(
                div()
                    .id(format!("display-{label}"))
                    .role(role)
                    .aria_label(label)
                    .tab_index(if disabled { -1 } else { 0 }),
                &look,
                &theme,
                look.field_bg,
            )
            .min_w(px(theme.controls.small * 2.0))
            .justify_center()
            .text_color(if is_mixed { look.placeholder } else { look.text })
            .child(value)
        };
        let channel_names = ["red", "green", "blue"];
        let channel_readout = |channel: usize, value: u8| {
            div()
                .text_size(px(theme.typography.caption))
                .text_color(on_row(look.label))
                .child(format!("{} {value}", ["R", "G", "B"][channel]))
        };
        let swatch = |color: &str| {
            div()
                .flex_none()
                .size(px(theme.spacing.medium))
                .rounded(px(theme.radii.small))
                .border(px(theme.borders.hairline))
                .border_color(on_row(look.card_border))
                .bg(rgba_from_hex(color))
        };
        let text_field = |text_id: String, aria_value: String, muted: bool| {
            let selector = format!("text-{text_id}");
            field(
                div()
                    .id(format!("text-{text_id}"))
                    .debug_selector(move || selector.clone())
                    .role(gpui_pre::accesskit::Role::TextInput)
                    .aria_label(label.clone())
                    .aria_value(aria_value)
                    .tab_index(if disabled { -1 } else { 0 }),
                &look,
                &theme,
                look.field_bg,
            )
            .min_w(px(theme.controls.small * 3.0))
            .text_color(if muted { look.placeholder } else { look.text })
        };
        let editing = self.editing_text && active;
        let caret = || {
            div()
                .flex_none()
                .w(px(theme.borders.hairline))
                .h(px(theme.typography.body))
                .bg(look.caret)
        };
        let select = |enum_id: String| {
            let selector = format!("enum-{enum_id}");
            field(
                div()
                    .id(format!("enum-{enum_id}"))
                    .debug_selector(move || selector.clone())
                    .role(gpui_pre::accesskit::Role::ComboBox)
                    .aria_label(label.clone())
                    .tab_index(if disabled { -1 } else { 0 }),
                &look,
                &theme,
                look.select_bg,
            )
            .min_w(px(theme.controls.small * 2.75))
            .justify_between()
            .gap(px(theme.spacing.small))
        };
        let boolean = |bool_id: &str, state: Option<bool>| {
            let selector = format!("boolean-{bool_id}");
            let on = state == Some(true);
            let height = theme.controls.xsmall * 0.68;
            let thumb = (height - 2.0 * theme.borders.hairline).min(theme.spacing.large);
            div()
                .id(format!("boolean-{bool_id}"))
                .debug_selector(move || selector.clone())
                .role(gpui_pre::accesskit::Role::CheckBox)
                .aria_label(label.clone())
                .aria_toggled(match state {
                    Some(value) => gpui_pre::accesskit::Toggled::from(value),
                    None => gpui_pre::accesskit::Toggled::Mixed,
                })
                .when(state.is_none(), |el| el.aria_description("Mixed value"))
                .tab_index(if disabled { -1 } else { 0 })
                .flex_none()
                .w(px(theme.controls.small))
                .h(px(height))
                .rounded(px(theme.radii.pill))
                .border(px(theme.borders.hairline))
                .border_color(if on { look.track_border_on } else { look.track_border_off })
                .bg(if on { look.track_on } else { look.track_off })
                .shadow(vec![box_shadow(look.shadow)])
                .flex()
                .items_center()
                // A mixed value centres the thumb so it differs from both concrete values.
                .map(|el| match state {
                    Some(true) => el.justify_end(),
                    Some(false) => el.justify_start(),
                    None => el.justify_center(),
                })
                .focus_visible(move |s| s.border_color(focus).shadow(vec![focus_ring(ring)]))
                .child(
                    div()
                        .size(px(thumb))
                        .rounded(px(theme.radii.pill))
                        .bg(if on { look.thumb_on } else { look.thumb_off })
                        .shadow(vec![box_shadow(look.shadow)]),
                )
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
                        MINUS,
                        format!("Decrease {label}"),
                        theme,
                        look,
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
                        PLUS,
                        format!("Increase {label}"),
                        theme,
                        look,
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
                            MINUS,
                            format!("Decrease {label}"),
                            theme,
                            look,
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
                            PLUS,
                            format!("Increase {label}"),
                            theme,
                            look,
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
                    text_field(id.clone(), text.clone(), false)
                        .when(editing, |el| {
                            el.border_color(look.focus).shadow(vec![focus_ring(look.ring)])
                        })
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.active = Some(text_id.clone());
                                this.editing_text = true;
                                this.text_draft = initial_text.clone();
                                cx.notify();
                            }))
                        })
                        .child(text.clone())
                        .when(editing, |el| el.child(caret())),
                );
            }
            (PropertyKind::Text, PropertyValue::Mixed) => {
                let text_id = id.clone();
                let aria_value = if editing { self.text_draft.clone() } else { String::new() };
                editor = editor.child(
                    text_field(id.clone(), aria_value, !editing)
                        .when(editing, |el| {
                            el.border_color(look.focus).shadow(vec![focus_ring(look.ring)])
                        })
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.active = Some(text_id.clone());
                                this.editing_text = true;
                                this.text_draft.clear();
                                cx.notify();
                            }))
                        })
                        .child(if editing { self.text_draft.clone() } else { "Mixed".into() })
                        .when(editing, |el| el.child(caret())),
                );
            }
            (PropertyKind::Boolean, PropertyValue::Boolean(value)) => {
                let next = !value;
                let bool_id = id.clone();
                editor = editor.child(boolean(&id, Some(*value)).when(!disabled, |el| {
                    el.on_click(cx.listener(move |this, _, _, cx| {
                        this.request(&bool_id, PropertyValue::Boolean(next), cx)
                    }))
                }));
            }
            (PropertyKind::Boolean, PropertyValue::Mixed) => {
                let bool_id = id.clone();
                editor = editor.child(boolean(&id, None).when(!disabled, |el| {
                    el.on_click(cx.listener(move |this, _, _, cx| {
                        this.request(&bool_id, PropertyValue::Boolean(true), cx)
                    }))
                }));
            }
            (PropertyKind::Enum { options }, PropertyValue::Enum(current)) => {
                let next = cycle_option(options, current);
                let enum_id = id.clone();
                editor = editor.child(
                    select(id.clone())
                        .text_color(look.text)
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.request(&enum_id, PropertyValue::Enum(next.clone()), cx)
                            }))
                        })
                        .child(current.clone())
                        .child(icon(theme.spacing.large, CHEVRON_DOWN, look.icon)),
                );
            }
            (PropertyKind::Enum { options }, PropertyValue::Mixed) => {
                let first = options.first().cloned().unwrap_or_default();
                let enum_id = id.clone();
                editor = editor.child(
                    select(id.clone())
                        .aria_value("Mixed")
                        .text_color(look.placeholder)
                        .when(!disabled, |el| {
                            el.on_click(cx.listener(move |this, _, _, cx| {
                                this.request(&enum_id, PropertyValue::Enum(first.clone()), cx)
                            }))
                        })
                        .child("Mixed")
                        .child(icon(theme.spacing.large, CHEVRON_DOWN, look.icon)),
                );
            }
            (PropertyKind::Color, PropertyValue::Color(color)) => {
                let parsed = parse_hex_rgb(color);
                editor = editor.child(
                    div().flex().items_center().gap(px(theme.spacing.xsmall)).child(swatch(color)),
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
                                MINUS,
                                format!("Decrease {label} {} channel", channel_names[channel]),
                                theme,
                                look,
                                disabled,
                                cx.listener(move |this, _, _, cx| {
                                    this.request(
                                        &down_id,
                                        PropertyValue::Color(down_color.clone()),
                                        cx,
                                    )
                                }),
                            ))
                            .child(channel_readout(channel, value))
                            .child(step_button(
                                PLUS,
                                format!("Increase {label} {} channel", channel_names[channel]),
                                theme,
                                look,
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
                    editor = editor.child(swatch(default_color));
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
                                .child(channel_readout(channel, value))
                                .child(step_button(
                                    MINUS,
                                    format!("Decrease {label} {} channel", channel_names[channel]),
                                    theme,
                                    look,
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
                                    PLUS,
                                    format!("Increase {label} {} channel", channel_names[channel]),
                                    theme,
                                    look,
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
                            MINUS,
                            format!("Decrease {label} component {}", index + 1),
                            theme,
                            look,
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
                            PLUS,
                            format!("Increase {label} component {}", index + 1),
                            theme,
                            look,
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
                                MINUS,
                                format!("Decrease {label} component {}", index + 1),
                                theme,
                                look,
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
                                PLUS,
                                format!("Increase {label} component {}", index + 1),
                                theme,
                                look,
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
                .text_size(px(theme.typography.body))
                .text_color(on_row(look.label))
                .child(label),
        );
        if is_mixed {
            // A filled (secondary) badge, so the tag does not read as another outlined editor.
            let (badge_bg, badge_text) = match look.badge_bg {
                Some(_) if active => (look.active_text, look.active_bg),
                Some(fill) => (fill, look.badge_text),
                None => (
                    mix(theme.colors.text, row_fill, look.muted_weight.unwrap_or(0.)),
                    look.badge_text,
                ),
            };
            let badge_bg = if disabled && !look.high_contrast { dim(badge_bg, look.card) } else { badge_bg };
            row = row.child(
                div()
                    .id(format!("mixed-{id}"))
                    .aria_label("Mixed value")
                    .flex_none()
                    .px(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.small))
                    .border(px(theme.borders.hairline))
                    .border_color(transparent)
                    .bg(badge_bg)
                    .text_size(px(theme.typography.caption))
                    .text_color(badge_text)
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle().tab_index(0)).clone();
        // `:focus-visible`: the active row outline shows while the root has keyboard focus.
        let focused = focus.is_focused(window) && window.last_input_was_keyboard();
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
            .rounded(px(theme.radii.large))
            .border(px(theme.borders.hairline))
            .border_color(look.card_border)
            .bg(look.card);

        let entity = cx.entity().downgrade();
        for group_index in 0..self.groups.len() {
            let label = self.groups[group_index].label.clone();
            let expanded = self.groups[group_index].expanded;
            let group_id = self.groups[group_index].id.clone();
            let toggle_entity = entity.clone();
            let toggle_group_id = group_id.clone();
            if group_index > 0 {
                root = root.child(div().w_full().h(px(theme.borders.hairline)).bg(look.card_border));
            }
            root = root.child(
                DisclosureTrigger::new(format!("group-{group_id}"), label.clone())
                    .expanded(expanded)
                    .disabled(disabled)
                    .on_toggle(move |_, cx| {
                        toggle_entity
                            .update(cx, |this, cx| this.toggle_group(&toggle_group_id, cx))
                            .ok();
                    }),
            );
            if expanded {
                let rows = (0..self.groups[group_index].properties.len())
                    .map(|property_index| {
                        self.render_property(group_index, property_index, theme, focused, cx)
                            .into_any_element()
                    })
                    .collect::<Vec<_>>();
                root = root.child(
                    DisclosurePanel::new(format!("group-panel-{group_id}"), label)
                        .p_0()
                        .px(px(theme.spacing.xsmall))
                        .pb(px(theme.spacing.xsmall))
                        .w_full()
                        .flex()
                        .flex_col()
                        .children(rows),
                );
            }
        }
        root
    }
}

/// Outline Button look with a vector minus/plus, at the inspector's compact size.
fn step_button(
    glyph: &'static [&'static [(f32, f32)]],
    label: String,
    theme: Theme,
    look: Look,
    disabled: bool,
    click: impl Fn(&gpui_pre::ClickEvent, &mut Window, &mut gpui_pre::App) + 'static,
) -> impl IntoElement {
    let selector_label = label.clone();
    let (focus, ring) = (look.focus, look.ring);
    let (hover_bg, hover_border) = (look.button_hover_bg, look.button_hover_border);
    div()
        .id(format!("control-{label}"))
        .debug_selector(move || format!("control-{selector_label}"))
        .role(gpui_pre::accesskit::Role::Button)
        .aria_label(label)
        .tab_index(if disabled { -1 } else { 0 })
        .flex_none()
        .w(px(theme.controls.xsmall * 0.72))
        .h(px(theme.controls.xsmall))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme.radii.medium))
        .border(px(theme.borders.regular))
        .border_color(look.button_border)
        .bg(look.button_bg)
        .shadow(vec![box_shadow(look.shadow)])
        .when(!disabled, |el| {
            el.hover(move |s| {
                let s = match hover_bg {
                    Some(fill) => s.bg(fill),
                    None => s,
                };
                match hover_border {
                    Some(color) => s.border_color(color),
                    None => s,
                }
            })
        })
        .focus_visible(move |s| s.border_color(focus).shadow(vec![focus_ring(ring)]))
        .when(!disabled, |el| el.on_click(click))
        .child(icon(theme.spacing.large, glyph, look.text))
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
        cx.update(|app| app.bind_keys(default_key_bindings()));
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
    fn group_headers_use_the_shared_disclosure_trigger_and_toggle_action(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (inspector, visual) = cx.add_window_view(|_, _| PropertyInspector::new(fixture()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("boolean-visible").is_some());

        // Focus the header through its element-owned focus handle, as Tab or a click would.
        let heading = visual.debug_bounds("group-transform").expect("group heading is rendered");
        visual.simulate_click(heading.center(), Modifiers::default());
        assert!(!inspector.read_with(visual, |view, _| view.groups[0].expanded));

        // Space and Enter on a focused header toggle the group through `disclosure::Toggle`,
        // which outranks the root Space binding, so the boolean property is untouched.
        visual.simulate_keystrokes("space");
        assert!(inspector.read_with(visual, |view, _| view.groups[0].expanded));
        visual.simulate_keystrokes("enter");
        assert!(!inspector.read_with(visual, |view, _| view.groups[0].expanded));
        assert_eq!(
            inspector.read_with(visual, |view, _| view.groups[0].properties[0].value.clone()),
            PropertyValue::Boolean(false)
        );
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("boolean-visible").is_none());
    }

    #[gpui_pre::test]
    fn disabled_inspector_group_headers_do_not_toggle(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (inspector, visual) =
            cx.add_window_view(|_, _| PropertyInspector::new(fixture()).disabled(true));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let heading = visual.debug_bounds("group-transform").expect("group heading is rendered");
        visual.simulate_click(heading.center(), Modifiers::default());
        visual.simulate_keystrokes("enter");
        assert!(inspector.read_with(visual, |view, _| view.groups[0].expanded));
    }

    #[test]
    fn default_bindings_append_the_shared_disclosure_bindings() {
        let bindings = default_key_bindings();
        assert_eq!(bindings.len(), 6);
        assert_eq!(
            bindings[4].keystrokes()[0].unparse(),
            disclosure_key_bindings()[0].keystrokes()[0].unparse()
        );
        assert_eq!(bindings[5].keystrokes()[0].unparse(), "enter");
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
