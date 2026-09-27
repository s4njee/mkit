extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, IntoElement, Modifiers, ParentElement, Render, TestAppContext, Window, div,
};
use mkit::{
    breadcrumbs::{Breadcrumbs, Crumb, default_key_bindings},
    core::theme,
};
use std::{cell::RefCell, rc::Rc};
struct Host {
    routes: Rc<RefCell<Vec<String>>>,
    disabled: bool,
}
impl Host {
    fn new(disabled: bool) -> Self {
        Self { routes: Rc::new(RefCell::new(Vec::new())), disabled }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let routes = self.routes.clone();
        let crumbs = Breadcrumbs::new(
            "Breadcrumb",
            vec![Crumb::new("Home").target("/home"), Crumb::new("Current")],
        )
        .disabled(self.disabled)
        .on_navigate(move |target| routes.borrow_mut().push(target));
        div().child(crumbs)
    }
}
#[gpui_pre::test]
fn ancestor_link_pointer_requests_navigation(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| Host::new(false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let link = visual.debug_bounds("breadcrumbs-crumb-0").expect("ancestor link bounds").center();
    visual.simulate_click(link, Modifiers::default());
    host.read_with(visual, |host, _| assert_eq!(*host.routes.borrow(), vec!["/home"]));
}
#[gpui_pre::test]
fn disabled_breadcrumb_links_do_not_request_navigation(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| Host::new(true));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let link = visual.debug_bounds("breadcrumbs-crumb-0").expect("ancestor text bounds").center();
    visual.simulate_click(link, Modifiers::default());
    host.read_with(visual, |host, _| assert!(host.routes.borrow().is_empty()));
}

#[gpui_pre::test]
fn enabled_ancestor_link_activates_after_tab_focus(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|app| app.bind_keys(default_key_bindings()));
    let (host, visual) = cx.add_window_view(|_, _| Host::new(false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.update(|window, cx| window.focus_next(cx));
    visual.simulate_keystrokes("enter");
    host.read_with(visual, |host, _| assert_eq!(*host.routes.borrow(), vec!["/home"]));
}

#[gpui_pre::test]
fn focused_ancestor_survives_parent_rerender(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|app| app.bind_keys(default_key_bindings()));
    let (host, visual) = cx.add_window_view(|_, _| Host::new(false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual.update(|window, cx| window.focus_next(cx));
    let focused_ancestor = visual.update(|window, cx| window.focused(cx).expect("ancestor focus"));

    host.update(visual, |_, cx| cx.notify());
    visual.update(|window, cx| window.draw(cx).clear(cx));

    assert!(visual.update(|window, _| focused_ancestor.is_focused(window)));
    visual.simulate_keystrokes("enter");
    host.read_with(visual, |host, _| assert_eq!(*host.routes.borrow(), vec!["/home"]));
}
