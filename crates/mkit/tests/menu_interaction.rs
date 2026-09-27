extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, Modifiers, ParentElement, Render,
    Subscription, Window, div,
};
use mkit::{
    context_menu::{self, ContextMenu},
    core::theme,
    dropdown_menu::{self, DropdownMenu, MenuItem},
};
use std::{cell::RefCell, rc::Rc};

struct Host {
    menu: Option<Entity<DropdownMenu>>,
    events: Rc<RefCell<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.menu.is_none() {
            let items = vec![
                MenuItem::new("more", "More").submenu(vec![MenuItem::new("nested", "Nested")]),
                MenuItem::new("pin", "Pin").checked(false),
                MenuItem::new("save", "Save").shortcut("⌘S"),
            ];
            let menu = cx.new(|_| DropdownMenu::controlled(items, true));
            let events = self.events.clone();
            let open = cx.subscribe(&menu, move |_, _, event: &dropdown_menu::OpenChanged, _| {
                events.borrow_mut().push(format!("open:{}", event.0));
            });
            let events = self.events.clone();
            let checked =
                cx.subscribe(&menu, move |_, _, event: &dropdown_menu::CheckedChanged, _| {
                    events.borrow_mut().push(format!("checked:{}:{}", event.0, event.1));
                });
            let events = self.events.clone();
            let selected =
                cx.subscribe(&menu, move |_, _, event: &dropdown_menu::ItemSelected, _| {
                    events.borrow_mut().push(format!("selected:{}", event.0));
                });
            self._subscriptions = vec![open, checked, selected];
            self.menu = Some(menu);
        }
        div().child(self.menu.as_ref().unwrap().clone())
    }
}

#[gpui_pre::test]
fn dropdown_menu_keyboard_submenu_check_and_leaf_activation(cx: &mut gpui_pre::TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|cx| cx.bind_keys(dropdown_menu::default_key_bindings()));
    let events = Rc::new(RefCell::new(Vec::new()));
    let (host, visual) = cx.add_window_view(|_, _| Host {
        menu: None,
        events: events.clone(),
        _subscriptions: Vec::new(),
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let menu = host.read_with(visual, |host, _| host.menu.as_ref().unwrap().clone());
    visual.update(|window, cx| {
        menu.update(cx, |menu, cx| menu.focus_handle(cx).focus(window, cx));
    });
    visual.simulate_keystrokes("right");
    assert!(visual.debug_bounds("menu-item-1-0").is_some(), "Right opens the active submenu");
    visual.simulate_keystrokes("left");
    assert!(visual.debug_bounds("menu-item-1-0").is_none(), "Left returns to the parent pane");

    let parent = visual.debug_bounds("menu-item-0-0").expect("submenu row").center();
    visual.simulate_click(parent, Modifiers::default());
    assert!(visual.debug_bounds("menu-item-1-0").is_some(), "click opens the active submenu");

    let pin = visual.debug_bounds("menu-item-0-1").expect("checkable row").center();
    visual.simulate_click(pin, Modifiers::default());
    assert!(events.borrow().contains(&"checked:pin:true".to_owned()));
    assert!(
        visual.debug_bounds("menu-item-0-2").is_some(),
        "checkable activation leaves menu open"
    );
    assert!(visual.debug_bounds("menu-shortcut-0-2").is_some(), "shortcut is visibly rendered");
    visual.simulate_keystrokes("cmd-s");
    assert!(!events.borrow().contains(&"selected:save".to_owned()));

    let save = visual.debug_bounds("menu-item-0-2").expect("shortcut row").center();
    visual.simulate_click(save, Modifiers::default());
    assert!(events.borrow().contains(&"selected:save".to_owned()));
    assert!(events.borrow().contains(&"open:false".to_owned()));
    host.read_with(visual, |host, _| assert!(!host.events.borrow().is_empty()));
}

struct ContextHost {
    menu: Option<Entity<ContextMenu>>,
    events: Rc<RefCell<Vec<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl Render for ContextHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.menu.is_none() {
            let items = vec![
                context_menu::MenuItem::new("more", "More")
                    .submenu(vec![context_menu::MenuItem::new("copy", "Copy")]),
                context_menu::MenuItem::new("pin", "Pin").checked(false),
            ];
            let menu = cx.new(|_| ContextMenu::controlled(items, true));
            let events = self.events.clone();
            let open = cx.subscribe(&menu, move |_, _, event: &context_menu::OpenChanged, _| {
                events.borrow_mut().push(format!("open:{}", event.0));
            });
            let events = self.events.clone();
            let checked =
                cx.subscribe(&menu, move |_, _, event: &context_menu::CheckedChanged, _| {
                    events.borrow_mut().push(format!("checked:{}:{}", event.0, event.1));
                });
            self._subscriptions = vec![open, checked];
            self.menu = Some(menu);
        }
        div().child(self.menu.as_ref().unwrap().clone())
    }
}

#[gpui_pre::test]
fn context_menu_pointer_submenu_and_checkable_item(cx: &mut gpui_pre::TestAppContext) {
    cx.update(theme::set_light_theme);
    cx.update(|cx| cx.bind_keys(context_menu::default_key_bindings()));
    let events = Rc::new(RefCell::new(Vec::new()));
    let (host, visual) = cx.add_window_view(|_, _| ContextHost {
        menu: None,
        events: events.clone(),
        _subscriptions: Vec::new(),
    });
    visual.update(|window, cx| window.draw(cx).clear(cx));
    let menu = host.read_with(visual, |host, _| host.menu.as_ref().unwrap().clone());
    visual.update(|window, cx| {
        menu.update(cx, |menu, cx| menu.focus_handle(cx).focus(window, cx));
    });
    visual.simulate_keystrokes("right");
    assert!(visual.debug_bounds("menu-item-1-0").is_some(), "Right opens nested pane");
    visual.simulate_keystrokes("left");
    assert!(visual.debug_bounds("menu-item-1-0").is_none(), "Left closes nested pane");
    let parent = visual.debug_bounds("menu-item-0-0").expect("submenu row").center();
    visual.simulate_click(parent, Modifiers::default());
    assert!(visual.debug_bounds("menu-item-1-0").is_some(), "click opens nested pane");
    let pin = visual.debug_bounds("menu-item-0-1").expect("checkable row").center();
    visual.simulate_click(pin, Modifiers::default());
    assert!(events.borrow().contains(&"checked:pin:true".to_owned()));
}
