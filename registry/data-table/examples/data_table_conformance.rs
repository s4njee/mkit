extern crate gpui_pre as gpui;
use gpui_pre::{
    AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Subscription,
    TestApp, Window, div,
};
use mkit_registry_data_table::{
    ActiveCellChanged, ActiveChanged, Column, DataRow, DataTable, SelectionChanged, SortChanged,
    default_key_bindings,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    io::{self, Read},
    rc::Rc,
};

#[derive(Default)]
struct Events {
    names: Vec<String>,
    cell: Option<String>,
}
struct Host {
    table: Option<Entity<DataTable>>,
    events: Rc<RefCell<Events>>,
    subscriptions: Vec<Subscription>,
}
impl Host {
    fn new() -> Self {
        Self {
            table: None,
            events: Rc::new(RefCell::new(Events::default())),
            subscriptions: Vec::new(),
        }
    }
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.table.is_none() {
            let table = cx.new(|_| {
                DataTable::new(
                    "Conformance files",
                    vec![Column::new("name", "Name"), Column::new("kind", "Kind")],
                    vec![
                        DataRow::new("row-b", vec!["Beta".into(), "File".into()]),
                        DataRow::new("row-a", vec!["Alpha".into(), "Folder".into()]),
                    ],
                )
            });
            let e = self.events.clone();
            self.subscriptions.push(cx.subscribe(&table, move |_, _, _: &ActiveChanged, _| {
                e.borrow_mut().names.push("ActiveChanged".into())
            }));
            let e = self.events.clone();
            self.subscriptions.push(cx.subscribe(&table, move |_, _, v: &ActiveCellChanged, _| {
                let mut e = e.borrow_mut();
                e.names.push("ActiveCellChanged".into());
                e.cell = Some(format!("{}:{}", v.row.as_deref().unwrap_or("header"), v.column));
            }));
            let e = self.events.clone();
            self.subscriptions.push(cx.subscribe(&table, move |_, _, _: &SelectionChanged, _| {
                e.borrow_mut().names.push("SelectionChanged".into())
            }));
            let e = self.events.clone();
            self.subscriptions.push(cx.subscribe(&table, move |_, _, _: &SortChanged, _| {
                e.borrow_mut().names.push("SortChanged".into())
            }));
            self.table = Some(table);
        }
        div().child(self.table.as_ref().unwrap().clone())
    }
}
fn key_name(key: &str) -> &'static str {
    match key {
        "ArrowDown" => "down",
        "ArrowUp" => "up",
        "ArrowRight" => "right",
        "ArrowLeft" => "left",
        "Home" => "home",
        "End" => "end",
        "Enter" => "enter",
        "Space" => "space",
        _ => panic!("unsupported key {key}"),
    }
}
fn run(case: &Value) -> Value {
    let mut app = TestApp::new();
    app.update(|cx| {
        mkit_core::theme::set_light_theme(cx);
        cx.bind_keys(default_key_bindings());
    });
    let mut window = app.open_window(|_, _| Host::new());
    let (table, events) = window.update(|host, window, cx| {
        let t = host.table.as_ref().unwrap().clone();
        t.focus_handle(cx).focus(window, cx);
        (t, host.events.clone())
    });
    if let Some(keys) = case["pre_dispatch"].as_array() {
        for key in keys {
            window.simulate_keystroke(key_name(key.as_str().unwrap()));
        }
    }
    events.borrow_mut().names.clear();
    events.borrow_mut().cell = None;
    window.simulate_keystroke(key_name(case["dispatch"]["key"].as_str().expect("key")));
    let (sort, selected, event_names, active_cell) = window.update(|_, _, cx| {
        let t = table.read(cx);
        let (column, direction) = t
            .sort()
            .map(|(c, d)| (Some(c.to_owned()), Some(format!("{d:?}"))))
            .unwrap_or((None, None));
        (
            json!({"column":column,"direction":direction}),
            t.selection().iter().cloned().collect::<Vec<_>>(),
            events.borrow().names.clone(),
            events.borrow().cell.clone(),
        )
    });
    let event = if event_names.iter().any(|name| name == "ActiveCellChanged") {
        "ActiveCellChanged"
    } else {
        event_names.last().map(String::as_str).unwrap_or("none")
    };
    json!({"passed":true,"actual":{"sort":sort,"selection":selected,"events":event_names,"event":event,"active_cell":active_cell}})
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read case");
    let case: Value = serde_json::from_str(&input).expect("case JSON");
    println!("{}", run(&case));
}
