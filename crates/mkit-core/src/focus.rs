//! Focus scopes, roving tab stops, and token-driven keyboard focus indication.
//!
//! Views still own their typed actions and call these helpers from rebindable
//! key contexts. A scope only traps focus while its view handles Tab actions.

use gpui_pre::{
    App, FocusHandle, InteractiveElement, KeyBinding, Styled, WeakFocusHandle, Window, actions, px,
};

use crate::theme::Theme;

actions!(
    mkit_core_focus,
    [FocusForward, FocusBackward, RoveNext, RovePrevious, RoveFirst, RoveLast, RoveUp, RoveDown]
);

/// Key context for an active dialog or popover focus trap.
pub const FOCUS_SCOPE_CONTEXT: &str = "MkitFocusScope";
/// Key context for a horizontal toolbar.
pub const ROVING_HORIZONTAL_CONTEXT: &str = "MkitRovingHorizontal";
/// Key context for a vertical list.
pub const ROVING_VERTICAL_CONTEXT: &str = "MkitRovingVertical";
/// Key context for a two-dimensional grid.
pub const ROVING_GRID_CONTEXT: &str = "MkitRovingGrid";

/// Register default, rebindable keys for focus scopes and roving groups.
///
/// Call once while setting up an app. Views attach the appropriate context and
/// handle these typed actions with their own `on_action` listeners.
pub fn bind_focus_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusForward, Some(FOCUS_SCOPE_CONTEXT)),
        KeyBinding::new("shift-tab", FocusBackward, Some(FOCUS_SCOPE_CONTEXT)),
        KeyBinding::new("left", RovePrevious, Some(ROVING_HORIZONTAL_CONTEXT)),
        KeyBinding::new("right", RoveNext, Some(ROVING_HORIZONTAL_CONTEXT)),
        KeyBinding::new("home", RoveFirst, Some(ROVING_HORIZONTAL_CONTEXT)),
        KeyBinding::new("end", RoveLast, Some(ROVING_HORIZONTAL_CONTEXT)),
        KeyBinding::new("up", RovePrevious, Some(ROVING_VERTICAL_CONTEXT)),
        KeyBinding::new("down", RoveNext, Some(ROVING_VERTICAL_CONTEXT)),
        KeyBinding::new("home", RoveFirst, Some(ROVING_VERTICAL_CONTEXT)),
        KeyBinding::new("end", RoveLast, Some(ROVING_VERTICAL_CONTEXT)),
        KeyBinding::new("left", RovePrevious, Some(ROVING_GRID_CONTEXT)),
        KeyBinding::new("right", RoveNext, Some(ROVING_GRID_CONTEXT)),
        KeyBinding::new("up", RoveUp, Some(ROVING_GRID_CONTEXT)),
        KeyBinding::new("down", RoveDown, Some(ROVING_GRID_CONTEXT)),
        KeyBinding::new("home", RoveFirst, Some(ROVING_GRID_CONTEXT)),
        KeyBinding::new("end", RoveLast, Some(ROVING_GRID_CONTEXT)),
    ]);
}

/// Ordered focus handles for one active dialog or popover.
///
/// The owner must render these handles with `track_focus`, call [`move_focus`]
/// for scoped Tab actions, and remove the key context when the overlay closes.
#[derive(Debug)]
pub struct FocusScope {
    stops: Vec<FocusHandle>,
    return_to: Option<WeakFocusHandle>,
}

/// Direction of movement within a focus scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusDirection {
    Forward,
    Backward,
}

impl FocusScope {
    /// Create a scope from its enabled stops in visual order.
    pub fn new(stops: Vec<FocusHandle>) -> Self {
        Self { stops, return_to: None }
    }

    /// Remember the control that should regain focus when the overlay closes.
    pub fn return_to(mut self, opener: &FocusHandle) -> Self {
        self.return_to = Some(opener.downgrade());
        self
    }

    /// Focus the first stop. Returns false if the scope is empty.
    pub fn focus_first(&self, window: &mut Window, cx: &mut App) -> bool {
        let Some(first) = self.stops.first() else { return false };
        first.focus(window, cx);
        true
    }

    /// Wrap focus among this scope's stops without entering the surrounding UI.
    ///
    /// If focus is outside the scope, forward starts at the first stop and
    /// backward starts at the last stop.
    pub fn move_focus(&self, direction: FocusDirection, window: &mut Window, cx: &mut App) -> bool {
        let Some(next) = wrapped_index(
            self.stops.iter().position(|stop| stop.contains_focused(window, cx)),
            self.stops.len(),
            direction,
        ) else {
            return false;
        };
        self.stops[next].focus(window, cx);
        true
    }

    /// Restore the opener's focus if its handle is still alive.
    pub fn restore_focus(&self, window: &mut Window, cx: &mut App) -> bool {
        let Some(opener) = self.return_to.as_ref().and_then(WeakFocusHandle::upgrade) else {
            return false;
        };
        opener.focus(window, cx);
        true
    }
}

fn wrapped_index(current: Option<usize>, len: usize, direction: FocusDirection) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match (current, direction) {
        (None, FocusDirection::Forward) => 0,
        (None, FocusDirection::Backward) => len - 1,
        (Some(index), FocusDirection::Forward) => (index + 1) % len,
        (Some(0), FocusDirection::Backward) => len - 1,
        (Some(index), FocusDirection::Backward) => index - 1,
    })
}

/// A group in which exactly one item participates in normal Tab navigation.
///
/// Supply only enabled items, in display order. After changing a list or grid,
/// rebuild the group with the new handles and restore the desired active index.
/// Render items with the handles returned by [`Self::handle`]; clones supplied
/// to [`Self::new`] do not receive later tab-stop changes.
#[derive(Debug)]
pub struct RovingFocus {
    items: Vec<FocusHandle>,
    active: usize,
}

impl RovingFocus {
    /// Create a group and make only the first item a tab stop.
    pub fn new(items: Vec<FocusHandle>) -> Option<Self> {
        if items.is_empty() {
            return None;
        }
        let mut group = Self { items, active: 0 };
        group.sync_tab_stops();
        Some(group)
    }

    /// The index of the sole tab stop.
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// The handle to pass to an item's `track_focus` while rendering.
    pub fn handle(&self, index: usize) -> Option<&FocusHandle> {
        self.items.get(index)
    }

    /// Focus an item by index and make it the sole tab stop.
    pub fn focus_at(&mut self, index: usize, window: &mut Window, cx: &mut App) -> bool {
        if index >= self.items.len() {
            return false;
        }
        self.active = index;
        self.sync_tab_stops();
        self.items[index].focus(window, cx);
        true
    }

    /// Move to the next item, wrapping at the end.
    pub fn next(&mut self, window: &mut Window, cx: &mut App) {
        self.focus_at((self.active + 1) % self.items.len(), window, cx);
    }

    /// Move to the previous item, wrapping at the start.
    pub fn previous(&mut self, window: &mut Window, cx: &mut App) {
        let next = if self.active == 0 { self.items.len() - 1 } else { self.active - 1 };
        self.focus_at(next, window, cx);
    }

    /// Focus the first item.
    pub fn first(&mut self, window: &mut Window, cx: &mut App) {
        self.focus_at(0, window, cx);
    }

    /// Focus the last item.
    pub fn last(&mut self, window: &mut Window, cx: &mut App) {
        self.focus_at(self.items.len() - 1, window, cx);
    }

    /// Move one grid row. At an incomplete final row, use its final item.
    pub fn row(&mut self, columns: usize, down: bool, window: &mut Window, cx: &mut App) -> bool {
        if columns == 0 {
            return false;
        }
        let target = if down {
            self.active.saturating_add(columns).min(self.items.len() - 1)
        } else {
            self.active.saturating_sub(columns)
        };
        self.focus_at(target, window, cx)
    }

    fn sync_tab_stops(&mut self) {
        for (index, handle) in self.items.iter_mut().enumerate() {
            *handle = handle.clone().tab_stop(index == self.active);
        }
    }
}

/// Apply the active theme's keyboard-only focus ring to a focusable element.
///
/// Call after `track_focus`. GPUI applies `focus_visible` only for keyboard
/// navigation; the width and color come from the theme global.
pub fn focus_visible_with_theme<E: InteractiveElement>(element: E, theme: &Theme) -> E {
    let color = theme.colors.focus;
    let width = px(theme.borders.strong);
    element.focus_visible(move |style| style.border(width).border_color(color))
}

#[cfg(test)]
mod tests {
    use super::{
        FOCUS_SCOPE_CONTEXT, FocusBackward, FocusDirection, FocusForward, FocusScope,
        ROVING_HORIZONTAL_CONTEXT, RoveNext, RovingFocus, bind_focus_keys, wrapped_index,
    };
    use gpui_pre::{
        Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
        TestAppContext, Window, div,
    };

    struct FocusFixture {
        scope_handle: FocusHandle,
        scope: FocusScope,
        first: FocusHandle,
        middle: FocusHandle,
        last: FocusHandle,
    }

    impl FocusFixture {
        fn new(cx: &mut Context<Self>) -> Self {
            let first = cx.focus_handle().tab_stop(true);
            let middle = cx.focus_handle().tab_stop(true);
            let last = cx.focus_handle().tab_stop(true);
            Self {
                scope_handle: cx.focus_handle(),
                scope: FocusScope::new(vec![first.clone(), middle.clone(), last.clone()]),
                first,
                middle,
                last,
            }
        }
    }

    impl Render for FocusFixture {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .key_context(FOCUS_SCOPE_CONTEXT)
                .track_focus(&self.scope_handle)
                .tab_group()
                .on_action(cx.listener(|this, _: &FocusForward, window, cx| {
                    this.scope.move_focus(FocusDirection::Forward, window, cx);
                }))
                .on_action(cx.listener(|this, _: &FocusBackward, window, cx| {
                    this.scope.move_focus(FocusDirection::Backward, window, cx);
                }))
                .child(div().id("first").track_focus(&self.first).tab_stop(true))
                .child(div().id("middle").track_focus(&self.middle).tab_stop(true))
                .child(div().id("last").track_focus(&self.last).tab_stop(true))
        }
    }

    struct RovingFixture {
        group: RovingFocus,
    }

    impl RovingFixture {
        fn new(cx: &mut Context<Self>) -> Self {
            let handles = vec![cx.focus_handle(), cx.focus_handle(), cx.focus_handle()];
            Self { group: RovingFocus::new(handles).expect("nonempty group") }
        }
    }

    impl Render for RovingFixture {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .key_context(ROVING_HORIZONTAL_CONTEXT)
                .on_action(cx.listener(|this, _: &RoveNext, window, cx| {
                    this.group.next(window, cx);
                    cx.notify();
                }))
                .children((0..3).map(|index| {
                    div()
                        .id(index)
                        .track_focus(self.group.handle(index).expect("rendered group item"))
                }))
        }
    }

    #[test]
    fn scope_wraps_and_recovers_focus_from_outside() {
        assert_eq!(wrapped_index(Some(2), 3, FocusDirection::Forward), Some(0));
        assert_eq!(wrapped_index(Some(0), 3, FocusDirection::Backward), Some(2));
        assert_eq!(wrapped_index(None, 3, FocusDirection::Forward), Some(0));
        assert_eq!(wrapped_index(None, 3, FocusDirection::Backward), Some(2));
        assert_eq!(wrapped_index(None, 0, FocusDirection::Forward), None);
    }

    #[gpui_pre::test]
    fn scope_and_roving_group_move_real_window_focus(cx: &mut TestAppContext) {
        let (view, visual) = cx.add_window_view(|_, cx| FocusFixture::new(cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let scope = view.read_with(visual, |fixture, _| {
            FocusScope::new(vec![
                fixture.first.clone(),
                fixture.middle.clone(),
                fixture.last.clone(),
            ])
        });
        visual.update(|window, cx| scope.focus_first(window, cx));
        assert!(visual.update(|window, cx| view.read(cx).first.is_focused(window)));
        visual.update(|window, cx| scope.move_focus(FocusDirection::Backward, window, cx));
        assert!(visual.update(|window, cx| view.read(cx).last.is_focused(window)));
        visual.update(|window, cx| scope.move_focus(FocusDirection::Forward, window, cx));
        assert!(visual.update(|window, cx| view.read(cx).first.is_focused(window)));

        let mut roving = view.read_with(visual, |fixture, _| {
            RovingFocus::new(vec![
                fixture.first.clone(),
                fixture.middle.clone(),
                fixture.last.clone(),
            ])
            .expect("nonempty focus group")
        });
        assert_eq!(roving.items.iter().filter(|handle| handle.tab_stop).count(), 1);
        visual.update(|window, cx| roving.next(window, cx));
        assert_eq!(roving.active_index(), 1);
        assert!(visual.update(|window, cx| view.read(cx).middle.is_focused(window)));
        visual.update(|window, cx| roving.row(2, true, window, cx));
        assert_eq!(roving.active_index(), 2);
        assert!(visual.update(|window, cx| view.read(cx).last.is_focused(window)));
        assert_eq!(roving.items.iter().filter(|handle| handle.tab_stop).count(), 1);
    }

    #[gpui_pre::test]
    fn scoped_tab_keys_wrap_in_a_rendered_window(cx: &mut TestAppContext) {
        cx.update(bind_focus_keys);
        let (view, visual) = cx.add_window_view(|_, cx| FocusFixture::new(cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let first = view.read_with(visual, |fixture, _| fixture.first.clone());
        visual.update(|window, cx| first.focus(window, cx));
        visual.simulate_keystrokes("tab tab tab");
        assert!(visual.update(|window, cx| view.read(cx).first.is_focused(window)));
        visual.simulate_keystrokes("shift-tab");
        assert!(visual.update(|window, cx| view.read(cx).last.is_focused(window)));
    }

    #[gpui_pre::test]
    fn roving_action_updates_the_rendered_tab_stop(cx: &mut TestAppContext) {
        cx.update(bind_focus_keys);
        let (view, visual) = cx.add_window_view(|_, cx| RovingFixture::new(cx));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let first = view.read_with(visual, |fixture, _| fixture.group.handle(0).unwrap().clone());
        visual.update(|window, cx| first.focus(window, cx));
        visual.simulate_keystrokes("right");
        assert_eq!(view.read_with(visual, |fixture, _| fixture.group.active_index()), 1);
        assert!(
            visual
                .update(|window, cx| { view.read(cx).group.handle(1).unwrap().is_focused(window) })
        );
        view.read_with(visual, |fixture, _| {
            assert!(!fixture.group.handle(0).unwrap().tab_stop);
            assert!(fixture.group.handle(1).unwrap().tab_stop);
            assert!(!fixture.group.handle(2).unwrap().tab_stop);
        });
    }
}
