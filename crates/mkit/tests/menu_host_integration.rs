extern crate gpui_pre as gpui;

use gpui_pre::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, Modifiers,
    ParentElement, Render, Styled, Window, div, point, px,
};
use mkit_core::theme;
use mkit_registry_context_menu::{ContextMenu, MenuItem as ContextItem};
use mkit_registry_dropdown_menu::{DropdownMenu, MenuItem as DropdownItem};

struct DropdownHost {
    menu: Option<Entity<DropdownMenu>>,
    opener: FocusHandle,
    outside: FocusHandle,
}

impl Render for DropdownHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.menu.is_none() {
            let menu = cx.new(|_| {
                DropdownMenu::new(vec![DropdownItem::new("open", "Open").checked(false)])
                    .anchor_at(point(px(120.), px(90.)))
                    .return_focus_to(&self.opener)
            });
            self.menu = Some(menu);
        }
        div()
            .size_full()
            .child(div().id("dropdown-opener").track_focus(&self.opener).tab_stop(true))
            .child(div().id("dropdown-outside").track_focus(&self.outside).tab_stop(true))
            .child(self.menu.as_ref().unwrap().clone())
    }
}

#[gpui_pre::test]
fn dropdown_menu_is_window_anchored_dismisses_outside_and_returns_focus(
    cx: &mut gpui_pre::TestAppContext,
) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, cx| DropdownHost {
        menu: None,
        opener: cx.focus_handle().tab_stop(true),
        outside: cx.focus_handle().tab_stop(true),
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let (menu, opener, outside) = host.read_with(visual, |host, _| {
        (host.menu.as_ref().unwrap().clone(), host.opener.clone(), host.outside.clone())
    });
    visual.update(|window, cx| {
        opener.focus(window, cx);
        menu.update(cx, |menu, cx| menu.set_open(true, cx));
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let bounds = visual.debug_bounds("menu-item-0-0").expect("anchored dropdown row");
    assert!(
        f32::from(bounds.origin.x) > 50.0,
        "menu should be placed at its window anchor: {bounds:?}"
    );
    assert!(f32::from(bounds.origin.y) > 50.0, "menu should honor its y anchor: {bounds:?}");
    assert!(menu.read_with(visual, |menu, _| menu.is_open()));
    visual.simulate_click(bounds.center(), Modifiers::default());
    assert!(menu.read_with(visual, |menu, _| menu.is_open()), "inside clicks keep the menu open");

    visual.simulate_mouse_down(
        point(px(700.), px(600.)),
        gpui_pre::MouseButton::Left,
        Default::default(),
    );
    assert!(!menu.read_with(visual, |menu, _| menu.is_open()));
    assert!(visual.update(|window, _| opener.is_focused(window)));
    assert!(!visual.update(|window, _| outside.is_focused(window)));
}

struct ContextHost {
    menu: Option<Entity<ContextMenu>>,
    invoker: FocusHandle,
    outside: FocusHandle,
}

impl Render for ContextHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.menu.is_none() {
            let menu = cx.new(|_| {
                ContextMenu::new(vec![ContextItem::new("open", "Open").checked(false)])
                    .anchor_at(point(px(160.), px(110.)))
                    .return_focus_to(&self.invoker)
            });
            self.menu = Some(menu);
        }
        div()
            .size_full()
            .child(div().id("context-invoker").track_focus(&self.invoker).tab_stop(true))
            .child(div().id("context-outside").track_focus(&self.outside).tab_stop(true))
            .child(self.menu.as_ref().unwrap().clone())
    }
}

#[gpui_pre::test]
fn context_menu_is_window_anchored_dismisses_outside_and_returns_focus(
    cx: &mut gpui_pre::TestAppContext,
) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, cx| ContextHost {
        menu: None,
        invoker: cx.focus_handle().tab_stop(true),
        outside: cx.focus_handle().tab_stop(true),
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let (menu, invoker, outside) = host.read_with(visual, |host, _| {
        (host.menu.as_ref().unwrap().clone(), host.invoker.clone(), host.outside.clone())
    });
    visual.update(|window, cx| {
        invoker.focus(window, cx);
        menu.update(cx, |menu, cx| menu.set_open(true, cx));
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let bounds = visual.debug_bounds("menu-item-0-0").expect("anchored context row");
    assert!(
        f32::from(bounds.origin.x) > 50.0,
        "menu should be placed at its window anchor: {bounds:?}"
    );
    assert!(f32::from(bounds.origin.y) > 50.0, "menu should honor its y anchor: {bounds:?}");
    assert!(menu.read_with(visual, |menu, _| menu.is_open()));
    visual.simulate_click(bounds.center(), Modifiers::default());
    assert!(menu.read_with(visual, |menu, _| menu.is_open()), "inside clicks keep the menu open");

    visual.simulate_mouse_down(
        point(px(700.), px(600.)),
        gpui_pre::MouseButton::Left,
        Default::default(),
    );
    assert!(!menu.read_with(visual, |menu, _| menu.is_open()));
    assert!(visual.update(|window, _| invoker.is_focused(window)));
    assert!(!visual.update(|window, _| outside.is_focused(window)));
}
