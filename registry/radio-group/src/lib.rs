//! Single-choice radio group with roving keyboard selection.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyBinding, Render, Window,
    actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;
pub const KEY_CONTEXT: &str = "RadioGroup";
actions!(radio_group, [Next, Previous, Left, Right, First, Last, Select]);
pub fn default_key_bindings() -> [KeyBinding; 7] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("right", Right, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Left, Some(KEY_CONTEXT)),
        KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Select, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
}
impl OptionItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), disabled: false }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeRequested(pub String);
impl EventEmitter<ChangeRequested> for RadioGroup {}
pub struct RadioGroup {
    label: String,
    options: Vec<OptionItem>,
    selected: Option<String>,
    controlled: bool,
    disabled: bool,
    orientation: Orientation,
    focus: Vec<FocusHandle>,
    focused: Option<usize>,
}
impl RadioGroup {
    pub fn value(&self) -> Option<&str> {
        self.selected.as_deref()
    }
    pub fn focused_option(&self) -> Option<&str> {
        self.focused.and_then(|i| self.options.get(i)).map(|option| option.id.as_str())
    }
    pub fn new(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        default_value: Option<String>,
    ) -> Self {
        Self {
            label: label.into(),
            options,
            selected: default_value,
            controlled: false,
            disabled: false,
            orientation: Orientation::Vertical,
            focus: Vec::new(),
            focused: None,
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        options: Vec<OptionItem>,
        value: Option<String>,
    ) -> Self {
        Self {
            label: label.into(),
            options,
            selected: value,
            controlled: true,
            disabled: false,
            orientation: Orientation::Vertical,
            focus: Vec::new(),
            focused: None,
        }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn set_value(&mut self, v: Option<String>) {
        self.selected = v;
    }
    fn choose(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let Some(o) = self.options.get(i).filter(|o| !o.disabled) else { return };
        let id = o.id.clone();
        self.focused = Some(i);
        if let Some(focus) = self.focus.get(i) {
            focus.focus(window, cx);
        }
        if self.selected.as_deref() == Some(&id) {
            return;
        }
        if !self.controlled {
            self.selected = Some(id.clone());
        }
        cx.emit(ChangeRequested(id));
    }
    fn move_by(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        if self.focused.is_none() && self.selected.is_none() {
            let first_or_last = if delta >= 0 {
                self.options.iter().position(|o| !o.disabled)
            } else {
                self.options.iter().rposition(|o| !o.disabled)
            };
            if let Some(i) = first_or_last {
                self.choose(i, window, cx);
            }
            return;
        }
        let current = self
            .focused
            .or_else(|| {
                self.selected.as_ref().and_then(|id| self.options.iter().position(|o| &o.id == id))
            })
            .unwrap_or(0);
        if let Some(i) = next_enabled(&self.options, current, delta) {
            self.choose(i, window, cx);
        }
    }
    fn next(&mut self, _: &Next, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Vertical {
            self.move_by(1, window, cx)
        }
    }
    fn prev(&mut self, _: &Previous, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Vertical {
            self.move_by(-1, window, cx)
        }
    }
    fn right(&mut self, _: &Right, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Horizontal {
            self.move_by(1, window, cx)
        }
    }
    fn left(&mut self, _: &Left, window: &mut Window, cx: &mut Context<Self>) {
        if self.orientation == Orientation::Horizontal {
            self.move_by(-1, window, cx)
        }
    }
    fn first(&mut self, _: &First, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.options.iter().position(|o| !o.disabled) {
            self.choose(i, window, cx)
        }
    }
    fn last(&mut self, _: &Last, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.options.iter().rposition(|o| !o.disabled) {
            self.choose(i, window, cx)
        }
    }
    fn select(&mut self, _: &Select, window: &mut Window, cx: &mut Context<Self>) {
        let i = self
            .focused
            .or_else(|| {
                self.selected.as_ref().and_then(|id| self.options.iter().position(|o| &o.id == id))
            })
            .unwrap_or(0);
        self.choose(i, window, cx)
    }
}
impl Focusable for RadioGroup {
    fn focus_handle(&self, _: &gpui_pre::App) -> FocusHandle {
        self.focus
            .get(self.focused.unwrap_or(0))
            .or_else(|| self.focus.first())
            .expect("radio options need focus")
            .clone()
    }
}
impl Render for RadioGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        while self.focus.len() < self.options.len() {
            self.focus.push(cx.focus_handle());
        }
        if self.focused.is_none() {
            self.focused = self
                .selected
                .as_ref()
                .and_then(|id| self.options.iter().position(|o| &o.id == id && !o.disabled))
                .or_else(|| self.options.iter().position(|o| !o.disabled));
        }
        let t = *cx.global::<Theme>();
        let label = self.label.clone();
        let selected = self.selected.clone();
        let disabled = self.disabled;
        let orientation = self.orientation;
        let options = self.options.clone();
        let entity = cx.entity();
        let mut root = div()
            .id("radio-group")
            .debug_selector(|| "radio-group-root".to_owned())
            .key_context(KEY_CONTEXT)
            .role(gpui_pre::accesskit::Role::RadioGroup)
            .aria_label(label.clone())
            .aria_orientation(match orientation {
                Orientation::Vertical => gpui_pre::accesskit::Orientation::Vertical,
                Orientation::Horizontal => gpui_pre::accesskit::Orientation::Horizontal,
            })
            .when(disabled, |d| {
                d.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::prev))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last))
            .on_action(cx.listener(Self::select))
            .flex()
            .flex_col()
            .when(orientation == Orientation::Horizontal, |d| d.flex_row())
            .gap(px(t.spacing.small));
        root = root.child(
            div().text_size(px(t.typography.body_emphasis)).text_color(t.colors.text).child(label),
        );
        for (i, o) in options.iter().enumerate() {
            let active = selected.as_deref() == Some(&o.id);
            let unavailable = disabled || o.disabled;
            let target = entity.clone();
            let focus = self.focus[i].clone();
            let debug_id = o.id.clone();
            root = root.child(
                div()
                    .id(o.id.clone())
                    .debug_selector(move || format!("radio-option-{debug_id}"))
                    .role(gpui_pre::accesskit::Role::RadioButton)
                    .aria_label(o.label.clone())
                    .aria_toggled(gpui_pre::accesskit::Toggled::from(active))
                    .when(unavailable, |d| {
                        d.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                    })
                    .when(!unavailable, |d| {
                        d.track_focus(&focus)
                            .tab_stop(self.focused == Some(i))
                            .tab_index(if self.focused == Some(i) { 0 } else { 1 })
                    })
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.small))
                    .when(!unavailable, |d| {
                        d.on_click(move |_, window, cx| {
                            target.update(cx, |group, cx| group.choose(i, window, cx))
                        })
                    })
                    .child(
                        div()
                            .size(px(t.controls.xsmall * 0.58))
                            .rounded(px(t.radii.pill))
                            .border(px(t.borders.regular))
                            .border_color(if active { t.colors.accent } else { t.colors.border })
                            .bg(if active { t.colors.accent } else { t.colors.surface }),
                    )
                    .child(
                        div()
                            .text_size(px(t.typography.body))
                            .text_color(if unavailable { t.colors.disabled } else { t.colors.text })
                            .child(o.label.clone()),
                    ),
            );
        }
        root
    }
}

/// Finds the next enabled option, wrapping in either direction.
pub fn next_enabled(options: &[OptionItem], current: usize, direction: isize) -> Option<usize> {
    let n = options.len();
    if n == 0 {
        return None;
    }
    (1..=n)
        .map(|step| (current as isize + direction * step as isize).rem_euclid(n as isize) as usize)
        .find(|&i| !options[i].disabled)
}
#[cfg(test)]
mod tests {
    use super::{OptionItem, next_enabled};
    #[test]
    fn navigation_wraps_and_skips_disabled_options() {
        let options = vec![
            OptionItem::new("a", "A"),
            OptionItem::new("b", "B").disabled(true),
            OptionItem::new("c", "C"),
        ];
        assert_eq!(next_enabled(&options, 0, 1), Some(2));
        assert_eq!(next_enabled(&options, 2, 1), Some(0));
        assert_eq!(next_enabled(&options, 0, -1), Some(2));
    }

    #[test]
    fn all_disabled_has_no_navigation_target() {
        let options = vec![OptionItem::new("a", "A").disabled(true)];
        assert_eq!(next_enabled(&options, 0, 1), None);
    }
}
