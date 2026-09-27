extern crate gpui_pre as gpui;

use gpui_pre::{
    AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Subscription, TestApp, Window, div,
};
use mkit::{
    core::theme,
    dialog::{self, Dialog},
    sheet::{self, Sheet},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

#[derive(Clone)]
enum Surface {
    Dialog(Entity<Dialog>),
    Sheet(Entity<Sheet>),
}

struct Host {
    component: String,
    interactive: bool,
    surface: Option<Surface>,
    stops: Option<(FocusHandle, FocusHandle)>,
    events: Rc<RefCell<Vec<bool>>>,
    subscription: Option<Subscription>,
}

impl Host {
    fn new(component: &str, interactive: bool) -> Self {
        Self {
            component: component.to_owned(),
            interactive,
            surface: None,
            stops: None,
            events: Rc::new(RefCell::new(Vec::new())),
            subscription: None,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.surface.is_none() {
            let events = self.events.clone();
            match self.component.as_str() {
                "dialog" => {
                    let dialog = if self.interactive {
                        let first = cx.focus_handle().tab_stop(true);
                        let second = cx.focus_handle().tab_stop(true);
                        self.stops = Some((first.clone(), second.clone()));
                        let content_first = first.clone();
                        let content_second = second.clone();
                        Dialog::with_content("Settings", move || {
                            div()
                                .child(
                                    div().track_focus(&content_first).tab_stop(true).child("First"),
                                )
                                .child(
                                    div()
                                        .track_focus(&content_second)
                                        .tab_stop(true)
                                        .child("Second"),
                                )
                        })
                        .focus_stops(vec![first, second])
                    } else {
                        Dialog::new("Settings", "Update your preferences")
                    };
                    let surface = cx.new(|_| dialog);
                    self.subscription = Some(cx.subscribe(
                        &surface,
                        move |_, _, event: &dialog::OpenChanged, _| {
                            events.borrow_mut().push(event.0)
                        },
                    ));
                    self.surface = Some(Surface::Dialog(surface));
                }
                "sheet" => {
                    let sheet = if self.interactive {
                        let first = cx.focus_handle().tab_stop(true);
                        let second = cx.focus_handle().tab_stop(true);
                        self.stops = Some((first.clone(), second.clone()));
                        let content_first = first.clone();
                        let content_second = second.clone();
                        Sheet::with_content("Details", move || {
                            div()
                                .child(
                                    div().track_focus(&content_first).tab_stop(true).child("First"),
                                )
                                .child(
                                    div()
                                        .track_focus(&content_second)
                                        .tab_stop(true)
                                        .child("Second"),
                                )
                        })
                        .focus_stops(vec![first, second])
                    } else {
                        Sheet::new("Details", "Review your changes")
                    };
                    let surface = cx.new(|_| sheet);
                    self.subscription =
                        Some(cx.subscribe(&surface, move |_, _, event: &sheet::OpenChanged, _| {
                            events.borrow_mut().push(event.0)
                        }));
                    self.surface = Some(Surface::Sheet(surface));
                }
                other => panic!("unsupported surface {other}"),
            }
        }
        let mut root = div();
        match self.surface.as_ref().expect("surface initialized") {
            Surface::Dialog(surface) => root = root.child(surface.clone()),
            Surface::Sheet(surface) => root = root.child(surface.clone()),
        }
        root
    }
}

fn run(case: &Value) -> Value {
    let component = case["component"].as_str().expect("component");
    let interactive = case["initial_state"] == "interactive";
    let key = case["dispatch"]["key"].as_str().expect("key");
    let keystroke = match (key, case["dispatch"]["modifiers"].as_array()) {
        ("Escape", _) => "escape",
        ("Tab", Some(modifiers)) if modifiers.iter().any(|modifier| modifier == "Shift") => {
            "shift-tab"
        }
        ("Tab", _) => "tab",
        _ => panic!("unsupported key {key}"),
    };
    let mut app = TestApp::new();
    app.update(|cx| {
        theme::set_light_theme(cx);
        cx.bind_keys(dialog::default_key_bindings());
        cx.bind_keys(sheet::default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new(component, interactive));
    let (surface, events) = window.update(|host, window, cx| {
        let surface = host.surface.as_ref().expect("surface rendered").clone();
        if let Some((first, _)) = &host.stops {
            first.focus(window, cx);
        } else {
            match &surface {
                Surface::Dialog(entity) => entity.focus_handle(cx).focus(window, cx),
                Surface::Sheet(entity) => entity.focus_handle(cx).focus(window, cx),
            }
        }
        (surface, host.events.clone())
    });
    window.simulate_keystroke(keystroke);
    let (open, focus_target) = window.update(|host, window, cx| {
        let (open, root_focused) = match &surface {
            Surface::Dialog(entity) => {
                (entity.read(cx).is_open(), entity.focus_handle(cx).is_focused(window))
            }
            Surface::Sheet(entity) => {
                (entity.read(cx).is_open(), entity.focus_handle(cx).is_focused(window))
            }
        };
        let target = if host.stops.as_ref().is_some_and(|(_, second)| second.is_focused(window)) {
            "second_control"
        } else if host.stops.as_ref().is_some_and(|(first, _)| first.is_focused(window)) {
            "first_control"
        } else if root_focused {
            "surface"
        } else {
            "none"
        };
        (open, target)
    });
    let events = events.borrow().clone();
    json!({
        "state": if open { "open" } else { "closed" },
        "event": if events.is_empty() { "none" } else { "OpenChanged(false)" },
        "focus_target": focus_target,
        "events": events,
    })
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read case");
    let case: Value = if input.trim().is_empty() {
        json!({"component": "dialog", "dispatch": {"key": "Escape", "modifiers": []}})
    } else {
        serde_json::from_str(&input).expect("case JSON")
    };
    println!("{}", json!({"passed": true, "actual": run(&case)}));
}
