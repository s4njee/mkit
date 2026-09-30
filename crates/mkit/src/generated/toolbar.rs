//! Stateful toolbar with one roving tab stop and an explicit overflow capacity.
extern crate gpui_pre as gpui;

use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, PathBuilder, Render,
    Rgba, Window, actions, canvas, div, point, prelude::*, px, relative,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
fn is_high_contrast(t: &Theme) -> bool {
    t.name == "high-contrast"
}
fn is_dark(t: &Theme) -> bool {
    relative_luminance(t.colors.background) < 0.5
}
/// Resolved colours for one toolbar control; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct ItemLook {
    bg: Rgba,
    fg: Rgba,
    border: Rgba,
    shadow: bool,
    hover_bg: Option<Rgba>,
    hover_border: Option<Rgba>,
    ring: Rgba,
}
fn item_look(t: &Theme, state: &ButtonState) -> ItemLook {
    let c = t.colors;
    let pressed = state.pressed && state.expanded.is_none();
    let open = state.expanded == Some(true);
    if is_high_contrast(t) {
        let (bg, fg, border, hover_border) = match (state.disabled, pressed, open) {
            (true, true, _) => (c.disabled, c.accent_text, c.disabled, None),
            (true, false, _) => (c.background, c.disabled, c.disabled, None),
            (false, true, _) => (c.accent, c.accent_text, c.accent, Some(c.text)),
            (false, false, true) => (c.background, c.text, c.accent, None),
            (false, false, false) => (c.background, c.text, c.border, Some(c.accent)),
        };
        return ItemLook {
            bg,
            fg,
            border,
            shadow: false,
            hover_bg: None,
            hover_border,
            ring: c.focus,
        };
    }
    let dark = is_dark(t);
    // shadcn "accent"/"muted": text mixed into the background.
    let muted = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    // The outline border is the web's translucent dark `input`/`border`, composited opaque.
    let outline = if dark { mix(c.text, c.background, 0.1) } else { c.border };
    let (bg, fg, border, hover_bg) = if pressed {
        // shadcn `default` (the preview's `ui-btn--primary` for `aria-pressed`).
        (c.accent, c.accent_text, c.accent, mix(c.accent, c.background, 0.9))
    } else if open {
        // shadcn `data-[state=open]:bg-accent` on the overflow trigger.
        (muted, c.text, outline, muted)
    } else {
        (c.background, c.text, outline, muted)
    };
    if state.disabled {
        // The preview's `opacity: .5`, flattened over the background part by part.
        return ItemLook {
            bg: mix(bg, c.background, 0.5),
            fg: mix(fg, c.background, 0.5),
            border: mix(border, c.background, 0.5),
            shadow: false,
            hover_bg: None,
            hover_border: None,
            ring: c.focus.opacity(0.5),
        };
    }
    ItemLook {
        bg,
        fg,
        border,
        shadow: true,
        hover_bg: Some(hover_bg),
        hover_border: None,
        ring: c.focus.opacity(0.5),
    }
}
/// Resolved container and overflow-menu colours.
#[derive(Clone, Copy)]
struct BarLook {
    bar_bg: Rgba,
    bar_border: Rgba,
    separator: Rgba,
    pane: Rgba,
    pane_border: Rgba,
    row_text: Rgba,
    row_muted: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    active_muted: Rgba,
    disabled_text: Rgba,
    disabled_muted: Rgba,
    icon_stroke: IconStroke,
}
fn bar_look(t: &Theme) -> BarLook {
    let c = t.colors;
    if is_high_contrast(t) {
        return BarLook {
            bar_bg: c.background,
            bar_border: c.border,
            separator: c.border,
            pane: c.background,
            pane_border: c.border,
            row_text: c.text,
            row_muted: c.text_muted,
            active_bg: c.accent,
            active_text: c.accent_text,
            active_muted: c.accent_text,
            disabled_text: c.disabled,
            disabled_muted: c.disabled,
            icon_stroke: IconStroke::Pixels(t.borders.regular),
        };
    }
    let dark = is_dark(t);
    let pane = c.surface;
    let border = if dark { mix(c.text, c.background, 0.1) } else { c.border };
    BarLook {
        bar_bg: c.background,
        bar_border: border,
        separator: border,
        pane,
        pane_border: if dark { mix(c.text, pane, 0.1) } else { c.border },
        row_text: c.text,
        row_muted: c.text_muted,
        active_bg: mix(c.text, c.background, if dark { 0.12 } else { 0.04 }),
        active_text: c.text,
        active_muted: c.text_muted,
        disabled_text: mix(c.text, pane, 0.5),
        disabled_muted: mix(c.text_muted, pane, 0.5),
        icon_stroke: IconStroke::Relative,
    }
}
#[derive(Clone, Copy)]
enum IconStroke {
    /// Lucide's 2-unit stroke on its 24-unit grid, scaled with the icon.
    Relative,
    Pixels(f32),
}
/// Decorative Lucide `check` (20,6 → 9,17 → 4,12) drawn as a vector path on a 24-unit grid.
fn check_icon(size: f32, stroke: IconStroke, color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let width = match stroke {
                IconStroke::Relative => unit * 2.0,
                IconStroke::Pixels(width) => px(width),
            };
            let mut path = PathBuilder::stroke(width);
            for (i, (x, y)) in [(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)].into_iter().enumerate() {
                let p = bounds.origin + point(unit * x, unit * y);
                if i == 0 { path.move_to(p) } else { path.line_to(p) }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
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
/// shadcn/ui focus ring width, drawn outside the control.
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

pub const KEY_CONTEXT: &str = "MkitToolbar";
actions!(toolbar, [Next, Previous, NextVertical, PreviousVertical, First, Last, Activate, Dismiss]);

pub fn default_key_bindings() -> [gpui_pre::KeyBinding; 9] {
    [
        gpui_pre::KeyBinding::new("right", Next, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("left", Previous, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("down", NextVertical, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("up", PreviousVertical, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("enter", Activate, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("space", Activate, Some(KEY_CONTEXT)),
        gpui_pre::KeyBinding::new("escape", Dismiss, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowPolicy {
    #[default]
    MayOverflow,
    NeverOverflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl Choice {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into(), disabled: false }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Button {
        id: String,
        label: String,
        disabled: bool,
        overflow: OverflowPolicy,
    },
    Toggle {
        id: String,
        label: String,
        disabled: bool,
        pressed: bool,
        overflow: OverflowPolicy,
    },
    ToggleGroup {
        id: String,
        label: String,
        choices: Vec<Choice>,
        value: Option<String>,
        overflow: OverflowPolicy,
    },
    Separator,
}

impl Item {
    pub fn button(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Button {
            id: id.into(),
            label: label.into(),
            disabled: false,
            overflow: OverflowPolicy::MayOverflow,
        }
    }
    pub fn toggle(id: impl Into<String>, label: impl Into<String>, pressed: bool) -> Self {
        Self::Toggle {
            id: id.into(),
            label: label.into(),
            disabled: false,
            pressed,
            overflow: OverflowPolicy::MayOverflow,
        }
    }
    pub fn toggle_group(
        id: impl Into<String>,
        label: impl Into<String>,
        choices: Vec<Choice>,
        value: Option<String>,
    ) -> Self {
        Self::ToggleGroup {
            id: id.into(),
            label: label.into(),
            choices,
            value,
            overflow: OverflowPolicy::MayOverflow,
        }
    }
    pub fn separator() -> Self {
        Self::Separator
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            Self::Button { disabled: value, .. } | Self::Toggle { disabled: value, .. } => {
                *value = disabled
            }
            _ => {}
        }
        self
    }
    pub fn overflow(mut self, policy: OverflowPolicy) -> Self {
        match &mut self {
            Self::Button { overflow, .. }
            | Self::Toggle { overflow, .. }
            | Self::ToggleGroup { overflow, .. } => *overflow = policy,
            Self::Separator => {}
        }
        self
    }
    fn enabled(&self) -> bool {
        match self {
            Self::Button { disabled, .. } | Self::Toggle { disabled, .. } => !disabled,
            Self::ToggleGroup { choices, .. } => choices.iter().any(|choice| !choice.disabled),
            Self::Separator => false,
        }
    }
    fn overflow_policy(&self) -> OverflowPolicy {
        match self {
            Self::Button { overflow, .. }
            | Self::Toggle { overflow, .. }
            | Self::ToggleGroup { overflow, .. } => *overflow,
            Self::Separator => OverflowPolicy::NeverOverflow,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolbarEvent {
    ActionInvoked { id: String },
    ToggleRequested { id: String, pressed: bool },
    GroupValueRequested { group_id: String, value: String },
    OverflowOpened,
    OverflowClosed,
}
impl EventEmitter<ToolbarEvent> for Toolbar {}

pub struct Toolbar {
    label: String,
    items: Vec<Item>,
    orientation: Orientation,
    controlled: bool,
    active: usize,
    active_choice: usize,
    focus: Vec<FocusHandle>,
    visible_capacity: Option<usize>,
    overflow_open: bool,
    overflow_active: usize,
}

impl Toolbar {
    pub fn new(label: impl Into<String>, items: Vec<Item>) -> Self {
        let active = items.iter().position(Item::enabled).unwrap_or(0);
        Self {
            label: label.into(),
            items,
            orientation: Orientation::Horizontal,
            controlled: false,
            active,
            active_choice: 0,
            focus: Vec::new(),
            visible_capacity: None,
            overflow_open: false,
            overflow_active: 0,
        }
    }
    /// Values in `items` are requests-only until the owner replaces them through setters.
    pub fn controlled(label: impl Into<String>, items: Vec<Item>) -> Self {
        let mut toolbar = Self::new(label, items);
        toolbar.controlled = true;
        toolbar
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// Keep at most this many overflow-eligible actions inline; `NeverOverflow` items stay inline.
    pub fn visible_capacity(mut self, capacity: usize) -> Self {
        self.visible_capacity = Some(capacity);
        self.active = self.focusable_indices().first().copied().unwrap_or(0);
        self
    }
    pub fn is_overflow_open(&self) -> bool {
        self.overflow_open
    }
    pub fn active_index(&self) -> usize {
        self.active
    }
    pub fn set_toggle(&mut self, id: &str, pressed: bool, cx: &mut Context<Self>) {
        if let Some(Item::Toggle { pressed: current, .. }) = self
            .items
            .iter_mut()
            .find(|item| matches!(item, Item::Toggle { id: item_id, .. } if item_id == id))
        {
            *current = pressed;
            cx.notify();
        }
    }
    pub fn set_group_value(
        &mut self,
        group_id: &str,
        value: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if let Some(Item::ToggleGroup { value: current, choices, .. }) = self
            .items
            .iter_mut()
            .find(|item| matches!(item, Item::ToggleGroup { id, .. } if id == group_id))
        {
            *current = value.filter(|value| {
                choices.iter().any(|choice| &choice.value == value && !choice.disabled)
            });
            cx.notify();
        }
    }
    fn is_visible(&self, index: usize) -> bool {
        let Some(capacity) = self.visible_capacity else { return true };
        if self.items[index].overflow_policy() == OverflowPolicy::NeverOverflow {
            return true;
        }
        let control_count = |item: &Item| match item {
            Item::ToggleGroup { choices, .. } => choices.len(),
            Item::Separator => 0,
            _ => 1,
        };
        let mut used = 0;
        let mut overflow_started = false;
        for earlier in &self.items[..index] {
            if earlier.overflow_policy() == OverflowPolicy::NeverOverflow {
                continue;
            }
            if overflow_started {
                continue;
            }
            let count = control_count(earlier);
            if used + count <= capacity {
                used += count;
            } else {
                overflow_started = true;
            }
        }
        !overflow_started && used + control_count(&self.items[index]) <= capacity
    }
    fn hidden_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                (!matches!(item, Item::Separator) && !self.is_visible(index)).then_some(index)
            })
            .collect()
    }
    fn focusable_indices(&self) -> Vec<usize> {
        let mut indices = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (self.is_visible(index) && item.enabled()).then_some(index))
            .collect::<Vec<_>>();
        if !self.hidden_indices().is_empty() {
            indices.push(self.items.len());
        }
        indices
    }
    fn move_to(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Item::ToggleGroup { choices, .. }) = self.items.get(self.active) {
            let enabled = choices
                .iter()
                .enumerate()
                .filter_map(|(i, choice)| (!choice.disabled).then_some(i))
                .collect::<Vec<_>>();
            if let Some(position) = enabled.iter().position(|i| *i == self.active_choice) {
                let next = position as isize + delta.signum();
                if (0..enabled.len() as isize).contains(&next) {
                    self.active_choice = enabled[next as usize];
                    cx.notify();
                    return;
                }
            }
        }
        let indices = self.focusable_indices();
        if indices.is_empty() {
            return;
        }
        let current = indices.iter().position(|index| *index == self.active).unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(indices.len() as isize) as usize;
        self.active = indices[next];
        self.active_choice = match self.items.get(self.active) {
            Some(Item::ToggleGroup { choices, .. }) if delta < 0 => {
                choices.iter().rposition(|choice| !choice.disabled).unwrap_or(0)
            }
            _ => 0,
        };
        if self.active == indices[current]
            && let Some(Item::ToggleGroup { choices, .. }) = self.items.get(self.active)
        {
            self.active_choice = if delta < 0 {
                choices.iter().rposition(|choice| !choice.disabled).unwrap_or(0)
            } else {
                choices.iter().position(|choice| !choice.disabled).unwrap_or(0)
            };
        }
        if let Some(handle) = self.focus.get(self.active) {
            window.focus(handle, cx);
        }
        cx.notify();
    }
    fn move_next(&mut self, _: &Next, window: &mut Window, cx: &mut Context<Self>) {
        if !self.overflow_open && self.orientation == Orientation::Horizontal {
            self.move_to(1, window, cx);
        }
    }
    fn move_previous(&mut self, _: &Previous, window: &mut Window, cx: &mut Context<Self>) {
        if !self.overflow_open && self.orientation == Orientation::Horizontal {
            self.move_to(-1, window, cx);
        }
    }
    fn move_down(&mut self, _: &NextVertical, window: &mut Window, cx: &mut Context<Self>) {
        if self.overflow_open {
            self.move_overflow(1, cx);
        } else if self.orientation == Orientation::Vertical {
            self.move_to(1, window, cx);
        }
    }
    fn move_up(&mut self, _: &PreviousVertical, window: &mut Window, cx: &mut Context<Self>) {
        if self.overflow_open {
            self.move_overflow(-1, cx);
        } else if self.orientation == Orientation::Vertical {
            self.move_to(-1, window, cx);
        }
    }
    fn move_overflow(&mut self, delta: isize, cx: &mut Context<Self>) {
        let enabled = self
            .hidden_indices()
            .iter()
            .enumerate()
            .filter_map(|(position, index)| self.items[*index].enabled().then_some(position))
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            return;
        }
        let current =
            enabled.iter().position(|position| *position == self.overflow_active).unwrap_or(0)
                as isize;
        self.overflow_active =
            enabled[(current + delta).rem_euclid(enabled.len() as isize) as usize];
        cx.notify();
    }
    fn jump(&mut self, last: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.overflow_open {
            let enabled = self
                .hidden_indices()
                .iter()
                .enumerate()
                .filter_map(|(position, index)| self.items[*index].enabled().then_some(position))
                .collect::<Vec<_>>();
            self.overflow_active = if last {
                enabled.last().copied().unwrap_or(0)
            } else {
                enabled.first().copied().unwrap_or(0)
            };
            cx.notify();
            return;
        }
        let indices = self.focusable_indices();
        if let Some(index) = if last { indices.last() } else { indices.first() } {
            self.active = *index;
            self.active_choice =
                if last { self.last_choice(*index) } else { self.first_choice(*index) };
            if let Some(handle) = self.focus.get(self.active) {
                window.focus(handle, cx);
            }
            cx.notify();
        }
    }
    fn first(&mut self, _: &First, window: &mut Window, cx: &mut Context<Self>) {
        self.jump(false, window, cx);
    }
    fn last(&mut self, _: &Last, window: &mut Window, cx: &mut Context<Self>) {
        self.jump(true, window, cx);
    }
    fn activate(&mut self, _: &Activate, _: &mut Window, cx: &mut Context<Self>) {
        if self.overflow_open {
            if let Some(index) = self.hidden_indices().get(self.overflow_active).copied() {
                self.activate_at(index, 0, cx);
                self.overflow_open = false;
                cx.emit(ToolbarEvent::OverflowClosed);
                cx.notify();
            }
        } else {
            self.activate_at(self.active, self.active_choice, cx);
        }
    }
    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        if self.overflow_open {
            self.overflow_open = false;
            cx.emit(ToolbarEvent::OverflowClosed);
            cx.notify();
        }
    }
    fn activate_at(&mut self, index: usize, choice_index: usize, cx: &mut Context<Self>) {
        if index == self.items.len() && !self.hidden_indices().is_empty() {
            self.overflow_open = !self.overflow_open;
            let hidden = self.hidden_indices();
            self.overflow_active =
                hidden.iter().position(|index| self.items[*index].enabled()).unwrap_or(0);
            if self.overflow_open {
                cx.emit(ToolbarEvent::OverflowOpened);
            } else {
                cx.emit(ToolbarEvent::OverflowClosed);
            }
            cx.notify();
            return;
        }
        let Some(item) = self.items.get_mut(index) else { return };
        match item {
            Item::Button { id, disabled, .. } if !*disabled => {
                cx.emit(ToolbarEvent::ActionInvoked { id: id.clone() })
            }
            Item::Toggle { id, disabled, pressed, .. } if !*disabled => {
                let requested = !*pressed;
                if !self.controlled {
                    *pressed = requested;
                    cx.notify();
                }
                cx.emit(ToolbarEvent::ToggleRequested { id: id.clone(), pressed: requested });
            }
            Item::ToggleGroup { id, choices, value, .. } => {
                if let Some(choice) = choices.get(choice_index).filter(|choice| !choice.disabled) {
                    let requested = choice.value.clone();
                    if !self.controlled {
                        *value = Some(requested.clone());
                        cx.notify();
                    }
                    cx.emit(ToolbarEvent::GroupValueRequested {
                        group_id: id.clone(),
                        value: requested,
                    });
                }
            }
            _ => {}
        }
    }
    fn first_choice(&self, index: usize) -> usize {
        match self.items.get(index) {
            Some(Item::ToggleGroup { choices, .. }) => {
                choices.iter().position(|choice| !choice.disabled).unwrap_or(0)
            }
            _ => 0,
        }
    }
    fn last_choice(&self, index: usize) -> usize {
        match self.items.get(index) {
            Some(Item::ToggleGroup { choices, .. }) => {
                choices.iter().rposition(|choice| !choice.disabled).unwrap_or(0)
            }
            _ => 0,
        }
    }
}

impl Render for Toolbar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let needed = self.items.len() + 1;
        while self.focus.len() < needed {
            self.focus.push(cx.focus_handle());
        }
        self.focus.truncate(needed);
        for index in 0..self.items.len() {
            self.focus[index] = self.focus[index]
                .clone()
                .tab_stop(index == self.active)
                .tab_index(if index == self.active { 0 } else { 1 });
        }
        let overflow_active = self.active == self.items.len();
        self.focus[self.items.len()] = self.focus[self.items.len()]
            .clone()
            .tab_stop(overflow_active)
            .tab_index(if overflow_active { 0 } else { 1 });
        let row = self.orientation == Orientation::Horizontal;
        let hidden = self.hidden_indices();
        let entity = cx.entity();
        let bar = bar_look(&theme);
        // The toolbar hugs its controls like the preview; the inner wrapper (no role) anchors the
        // overflow menu to the toolbar's own edges.
        let shell = div().id("mkit-toolbar-shell").w_full().flex().flex_col().items_start();
        let mut anchor = div().relative().flex().flex_col();
        let mut root = div()
            .id("mkit-toolbar")
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::Toolbar)
            .aria_label(self.label.clone())
            .aria_orientation(if row {
                gpui_pre::accesskit::Orientation::Horizontal
            } else {
                gpui_pre::accesskit::Orientation::Vertical
            })
            .flex()
            .when(row, |el| el.flex_row().items_center())
            .when(!row, |el| el.flex_col().items_stretch())
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.xsmall))
            .rounded(px(theme.radii.medium))
            .border(px(theme.borders.regular))
            .border_color(bar.bar_border)
            .bg(bar.bar_bg)
            .shadow(vec![box_shadow(theme.shadows.small)])
            .on_action(cx.listener(Self::move_next))
            .on_action(cx.listener(Self::move_previous))
            .on_action(cx.listener(Self::move_down))
            .on_action(cx.listener(Self::move_up))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last))
            .on_action(cx.listener(Self::activate))
            .on_action(cx.listener(Self::dismiss));
        for (index, item) in self.items.iter().enumerate() {
            if !self.is_visible(index) {
                continue;
            }
            if let Item::Separator = item {
                let previous_is_action = self.items[..index]
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(candidate_index, _)| self.is_visible(*candidate_index))
                    .is_some_and(|(_, candidate)| !matches!(candidate, Item::Separator));
                let next_is_action = self.items[index + 1..]
                    .iter()
                    .enumerate()
                    .find(|(offset, _)| self.is_visible(index + 1 + *offset))
                    .is_some_and(|(_, candidate)| !matches!(candidate, Item::Separator));
                if previous_is_action && next_is_action {
                    root = root.child(
                        div()
                            .flex_none()
                            .when(row, |el| {
                                el.w(px(theme.borders.hairline)).h(px(theme.spacing.xlarge))
                            })
                            .when(!row, |el| el.h(px(theme.borders.hairline)))
                            .bg(bar.separator),
                    );
                }
                continue;
            }
            let handle = &self.focus[index];
            match item {
                Item::Button { id, label, disabled, .. } => {
                    let _id = id.clone();
                    let entity = entity.clone();
                    root = root.child(button_element(
                        theme,
                        handle,
                        format!("toolbar-action-{index}"),
                        label,
                        ButtonState {
                            active: index == self.active,
                            disabled: *disabled,
                            toggle: false,
                            pressed: false,
                            expanded: None,
                        },
                        move |cx| {
                            entity.update(cx, |toolbar, cx| {
                                toolbar.active = index;
                                toolbar.activate_at(index, 0, cx);
                            })
                        },
                    ));
                }
                Item::Toggle { id: _, label, disabled, pressed, .. } => {
                    let pressed = *pressed;
                    let disabled = *disabled;
                    let entity = entity.clone();
                    root = root.child(button_element(
                        theme,
                        handle,
                        format!("toolbar-action-{index}"),
                        label,
                        ButtonState {
                            active: index == self.active,
                            disabled,
                            toggle: true,
                            pressed,
                            expanded: None,
                        },
                        move |cx| {
                            entity.update(cx, |toolbar, cx| {
                                toolbar.active = index;
                                toolbar.activate_at(index, 0, cx);
                            })
                        },
                    ));
                }
                Item::ToggleGroup { id, label, choices, value, .. } => {
                    let mut group = div()
                        .id(format!("toolbar-group-{index}"))
                        .role(gpui_pre::accesskit::Role::Group)
                        .aria_label(label.clone())
                        .flex()
                        .items_center()
                        .gap(px(theme.spacing.xsmall));
                    for (choice_index, choice) in choices.iter().enumerate() {
                        let enabled = !choice.disabled;
                        let selected = value.as_deref() == Some(&choice.value);
                        let group_id = id.clone();
                        let requested_value = choice.value.clone();
                        let entity = entity.clone();
                        let choice_label = choice.label.clone();
                        group = group.child(button_element(
                            theme,
                            handle,
                            format!("toolbar-choice-{index}-{choice_index}"),
                            &choice_label,
                            ButtonState {
                                active: index == self.active && choice_index == self.active_choice,
                                disabled: !enabled,
                                toggle: true,
                                pressed: selected,
                                expanded: None,
                            },
                            move |cx| {
                                entity.update(cx, |toolbar, cx| {
                                    toolbar.active = index;
                                    toolbar.active_choice = choice_index;
                                    if !enabled {
                                        return;
                                    }
                                    if let Some(Item::ToggleGroup { value: current, .. }) =
                                        toolbar.items.get_mut(index)
                                    {
                                        if !toolbar.controlled {
                                            *current = Some(requested_value.clone());
                                            cx.notify();
                                        }
                                        cx.emit(ToolbarEvent::GroupValueRequested {
                                            group_id: group_id.clone(),
                                            value: requested_value.clone(),
                                        });
                                    }
                                })
                            },
                        ));
                    }
                    root = root.child(group);
                }
                Item::Separator => {}
            }
        }
        if !hidden.is_empty() {
            let trigger_entity = entity.clone();
            root = root.child(button_element(
                theme,
                &self.focus[self.items.len()],
                "toolbar-overflow-trigger".to_owned(),
                "More actions",
                ButtonState {
                    active: overflow_active,
                    disabled: false,
                    toggle: false,
                    pressed: self.overflow_open,
                    expanded: Some(self.overflow_open),
                },
                move |cx| {
                    trigger_entity
                        .update(cx, |toolbar, cx| toolbar.activate_at(toolbar.items.len(), 0, cx))
                },
            ));
            if self.overflow_open {
                let icon_size = theme.spacing.large;
                let mut menu = div()
                    .id("toolbar-overflow-menu")
                    .debug_selector(|| "toolbar-overflow-menu".into())
                    .role(gpui_pre::accesskit::Role::Menu)
                    .flex()
                    .flex_col()
                    .min_w(px(theme.spacing.xxlarge * 4.0))
                    .p(px(theme.spacing.xsmall))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(bar.pane_border)
                    .bg(bar.pane)
                    .shadow(vec![box_shadow(theme.shadows.medium)]);
                for (menu_index, index) in hidden.into_iter().enumerate() {
                    let active = self.overflow_active == menu_index;
                    let (id, label, disabled, checked) = match &self.items[index] {
                        Item::Button { id, label, disabled, .. } => {
                            (id.clone(), label.clone(), *disabled, None)
                        }
                        Item::Toggle { id, label, disabled, pressed, .. } => {
                            (id.clone(), label.clone(), *disabled, Some(*pressed))
                        }
                        Item::ToggleGroup { id, label, .. } => {
                            (id.clone(), label.clone(), false, None)
                        }
                        Item::Separator => continue,
                    };
                    let (fg, muted) = if disabled {
                        (bar.disabled_text, bar.disabled_muted)
                    } else if active {
                        (bar.active_text, bar.active_muted)
                    } else {
                        (bar.row_text, bar.row_muted)
                    };
                    let menu_entity = entity.clone();
                    menu = menu.child(
                        div()
                            .id(format!("toolbar-overflow-{id}"))
                            .role(if checked.is_some() {
                                gpui_pre::accesskit::Role::MenuItemCheckBox
                            } else {
                                gpui_pre::accesskit::Role::MenuItem
                            })
                            .when(active, |el| el.aria_active_descendant().bg(bar.active_bg))
                            .aria_label(label.clone())
                            .when(disabled, |el| {
                                el.a11y_synthetic_children(|builder| {
                                    builder.parent_node().set_disabled()
                                })
                            })
                            .when(checked.is_some(), |el| {
                                el.aria_toggled(checked.unwrap_or(false).into())
                            })
                            .when(!disabled, |el| {
                                el.on_click(move |_, _, cx| {
                                    menu_entity.update(cx, |toolbar, cx| {
                                        toolbar.activate_at(index, 0, cx);
                                        toolbar.overflow_open = false;
                                        cx.emit(ToolbarEvent::OverflowClosed);
                                        cx.notify();
                                    })
                                })
                            })
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.small))
                            .px(px(theme.spacing.small))
                            .h(px(theme.controls.small))
                            .rounded(px(theme.radii.small))
                            .text_size(px(theme.typography.body))
                            .text_color(fg)
                            .when_some(checked, |el, checked| {
                                el.child(div().size(px(icon_size)).flex_none().when(
                                    checked,
                                    |slot| {
                                        slot.child(check_icon(icon_size, bar.icon_stroke, muted))
                                    },
                                ))
                            })
                            .child(div().flex_grow(1.0).whitespace_nowrap().child(label)),
                    );
                }
                // shadcn's `sideOffset={4}` below the toolbar, aligned to its trailing edge
                // (horizontal) or leading edge (vertical).
                menu = menu
                    .absolute()
                    .top(relative(1.))
                    .mt(px(theme.spacing.xsmall))
                    .when(row, |el| el.right(px(0.)))
                    .when(!row, |el| el.left(px(0.)));
                anchor = anchor.child(menu);
            }
        }
        shell.child(anchor.child(root))
    }
}

struct ButtonState {
    active: bool,
    disabled: bool,
    toggle: bool,
    pressed: bool,
    expanded: Option<bool>,
}

fn button_element(
    theme: Theme,
    handle: &FocusHandle,
    id: String,
    label: &str,
    state: ButtonState,
    click: impl Fn(&mut gpui_pre::App) + 'static,
) -> impl IntoElement {
    let selector = id.clone();
    let look = item_look(&theme, &state);
    div()
        .id(id)
        .debug_selector(move || selector.clone())
        .track_focus(handle)
        .tab_index(if state.active { 0 } else { 1 })
        .role(gpui_pre::accesskit::Role::Button)
        .aria_label(label.to_owned())
        .when_some(state.expanded, |el, expanded| el.aria_expanded(expanded))
        .when(state.toggle, |el| el.aria_toggled(state.pressed.into()))
        .when(state.disabled, |el| el.aria_description("Unavailable"))
        .when(state.disabled, |el| {
            el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
        })
        .when(!state.disabled, |el| el.on_click(move |_, _, cx| click(cx)))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .gap(px(theme.spacing.small))
        .h(px(theme.controls.small))
        .px(px(theme.spacing.medium))
        .rounded(px(theme.radii.medium))
        .border(px(theme.borders.regular))
        .border_color(look.border)
        .bg(look.bg)
        .when(look.shadow, |el| el.shadow(vec![box_shadow(theme.shadows.small)]))
        .text_color(look.fg)
        .text_size(px(theme.typography.body))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .when(!state.disabled, |el| {
            el.hover(move |s| {
                let s = match look.hover_bg {
                    Some(color) => s.bg(color),
                    None => s,
                };
                match look.hover_border {
                    Some(color) => s.border_color(color),
                    None => s,
                }
            })
        })
        .focus_visible(move |s| {
            s.border_color(theme.colors.focus).shadow(vec![focus_ring(look.ring)])
        })
        .child(label.to_owned())
}

impl Focusable for Toolbar {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus
            .get(self.active)
            .cloned()
            .expect("toolbar focus handles initialize during render")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, MouseButton, TestAppContext, point};
    use std::{cell::RefCell, rc::Rc};

    fn fixture() -> Vec<Item> {
        vec![
            Item::button("save", "Save"),
            Item::button("disabled", "Disabled").disabled(true),
            Item::toggle("bold", "Bold", false),
        ]
    }

    #[gpui_pre::test]
    fn arrow_navigation_skips_disabled_and_pointer_invokes_action(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (toolbar, visual) =
            cx.add_window_view(|_, _| Toolbar::new("Document actions", fixture()));
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        let _sub = visual.update(|_, app| {
            app.subscribe(&toolbar, move |_, event: &ToolbarEvent, _| {
                log.borrow_mut().push(event.clone())
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| toolbar.read(cx).focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right");
        assert_eq!(toolbar.read_with(visual, |toolbar, _| toolbar.active_index()), 2);
        let bounds = visual.debug_bounds("toolbar-action-2").expect("toggle is rendered");
        let center = point(bounds.center().x, bounds.center().y);
        visual.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());
        assert_eq!(
            events.borrow().last(),
            Some(&ToolbarEvent::ToggleRequested { id: "bold".into(), pressed: true })
        );
    }

    #[gpui_pre::test]
    fn capacity_opens_menu_and_never_overflow_stays_inline(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let items = vec![
            Item::button("save", "Save"),
            Item::button("pin", "Pin").overflow(OverflowPolicy::NeverOverflow),
            Item::button("share", "Share"),
        ];
        let (toolbar, visual) =
            cx.add_window_view(|_, _| Toolbar::new("Document actions", items).visible_capacity(0));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("toolbar-overflow-trigger").is_some());
        assert_eq!(toolbar.read_with(visual, |toolbar, _| toolbar.active_index()), 1);
        visual.update(|window, cx| toolbar.read(cx).focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("end enter");
        assert!(toolbar.read_with(visual, |toolbar, _| toolbar.is_overflow_open()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(visual.debug_bounds("toolbar-overflow-menu").is_some());
        visual.simulate_keystrokes("down escape");
        assert!(!toolbar.read_with(visual, |toolbar, _| toolbar.is_overflow_open()));
    }

    #[gpui_pre::test]
    fn toolbar_arrows_choose_toggle_group_values(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let group = Item::toggle_group(
            "alignment",
            "Alignment",
            vec![
                Choice::new("left", "Left"),
                Choice::new("center", "Center"),
                Choice::new("right", "Right"),
            ],
            None,
        );
        let (toolbar, visual) =
            cx.add_window_view(|_, _| Toolbar::new("Document actions", vec![group]));
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        let _sub = visual.update(|_, app| {
            app.subscribe(&toolbar, move |_, event: &ToolbarEvent, _| {
                log.borrow_mut().push(event.clone())
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| toolbar.read(cx).focus_handle(cx).focus(window, cx));
        visual.simulate_keystrokes("right enter");
        assert_eq!(
            events.borrow().last(),
            Some(&ToolbarEvent::GroupValueRequested {
                group_id: "alignment".into(),
                value: "center".into()
            })
        );
    }

    #[test]
    fn controlled_values_wait_for_owner_and_overflow_policy_is_explicit() {
        let toolbar =
            Toolbar::controlled("Document actions", vec![Item::toggle("bold", "Bold", false)]);
        assert!(toolbar.controlled);
        assert_eq!(toolbar.items[0].overflow_policy(), OverflowPolicy::MayOverflow);
        assert_eq!(
            Item::button("pin", "Pin").overflow(OverflowPolicy::NeverOverflow).overflow_policy(),
            OverflowPolicy::NeverOverflow
        );
    }

    #[test]
    fn groups_overflow_atomically_and_disabled_candidates_remain_in_menu() {
        let group = Item::toggle_group(
            "alignment",
            "Alignment",
            vec![Choice::new("left", "Left"), Choice::new("center", "Center")],
            None,
        );
        let toolbar = Toolbar::new(
            "Document actions",
            vec![
                Item::button("save", "Save"),
                group,
                Item::button("disabled", "Disabled").disabled(true),
            ],
        )
        .visible_capacity(1);
        assert_eq!(toolbar.hidden_indices(), vec![1, 2]);
        let zero = Toolbar::new(
            "Document actions",
            vec![Item::button("disabled", "Disabled").disabled(true)],
        )
        .visible_capacity(0);
        assert_eq!(zero.focusable_indices(), vec![1]);
    }
}
