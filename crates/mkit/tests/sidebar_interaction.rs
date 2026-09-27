extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, IntoElement, Modifiers, ParentElement, Render, Subscription,
    TestAppContext, Window, div,
};
use mkit::{
    core::theme,
    sidebar::{Item, Sidebar, ValueChanged},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    sidebar: Option<Entity<Sidebar>>,
    changes: Rc<RefCell<Vec<Option<String>>>>,
    _sub: Option<Subscription>,
    controlled: bool,
    disabled: bool,
}
impl Host {
    fn new(controlled: bool, disabled: bool) -> Self {
        Self {
            sidebar: None,
            changes: Rc::new(RefCell::new(Vec::new())),
            _sub: None,
            controlled,
            disabled,
        }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.sidebar.is_none() {
            let items = vec![
                Item::new("home", "Home"),
                Item::new("blocked", "Blocked").disabled(true),
                Item::new("settings", "Settings"),
            ];
            let sidebar = cx.new(|_| {
                let s = if self.controlled {
                    Sidebar::controlled("Primary", items, Some("home".into()))
                } else {
                    Sidebar::new("Primary", items, Some("home".into()))
                };
                s.disabled(self.disabled)
            });
            let changes = self.changes.clone();
            self._sub = Some(cx.subscribe(&sidebar, move |_, _, e: &ValueChanged, _| {
                changes.borrow_mut().push(e.0.clone())
            }));
            self.sidebar = Some(sidebar);
        }
        div().child(self.sidebar.as_ref().unwrap().clone())
    }
}
#[gpui_pre::test]
fn pointer_requests_enabled_navigation_and_ignores_disabled_links(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| Host::new(false, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = visual.debug_bounds("sidebar-item-2").expect("settings link bounds").center();
    visual.simulate_click(target, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(host.sidebar.as_ref().unwrap().read(cx).value(), Some("settings"));
        assert_eq!(*host.changes.borrow(), vec![Some("settings".into())]);
    });
    let blocked = visual.debug_bounds("sidebar-item-1").expect("disabled link bounds").center();
    visual.simulate_click(blocked, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(host.sidebar.as_ref().unwrap().read(cx).value(), Some("settings"));
        assert_eq!(*host.changes.borrow(), vec![Some("settings".into())]);
    });
}
#[gpui_pre::test]
fn controlled_and_container_disabled_requests(cx: &mut TestAppContext) {
    cx.update(theme::set_light_theme);
    let (host, visual) = cx.add_window_view(|_, _| Host::new(true, false));
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = visual.debug_bounds("sidebar-item-2").unwrap().center();
    visual.simulate_click(target, Modifiers::default());
    host.read_with(visual, |host, cx| {
        assert_eq!(
            host.sidebar.as_ref().unwrap().read(cx).value(),
            Some("home"),
            "controlled value stays with its owner"
        );
        assert_eq!(*host.changes.borrow(), vec![Some("settings".into())]);
    });
    let (disabled_host, disabled_visual) = cx.add_window_view(|_, _| Host::new(false, true));
    disabled_visual.update(|window, cx| window.draw(cx).clear(cx));
    let target = disabled_visual.debug_bounds("sidebar-item-2").unwrap().center();
    disabled_visual.simulate_click(target, Modifiers::default());
    disabled_host.read_with(disabled_visual, |host, cx| {
        assert_eq!(host.sidebar.as_ref().unwrap().read(cx).value(), Some("home"));
        assert!(host.changes.borrow().is_empty());
    });
}
