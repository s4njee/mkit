use gpui_pre::{
    App, Context, Entity, Focusable, IntoElement, Keystroke, Render, Subscription, Window, div,
    prelude::*, px, size,
};
use image::RgbaImage;
use mkit::{
    core::theme::{self, HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme},
    data_table::{self, ActiveCellChanged, Column, DataRow, DataTable, SortDirection},
};
use mkit_harness::{HeadlessSession, PixelTolerance, ScreenshotError};
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    rc::Rc,
};

const SIZE: (f32, f32) = (620.0, 250.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum State {
    Idle,
    Sorted,
    Selected,
    ActiveCell,
    ActiveHeader,
}

impl State {
    fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Sorted => "sorted",
            Self::Selected => "selected",
            Self::ActiveCell => "active-cell",
            Self::ActiveHeader => "active-header",
        }
    }

    fn from_manifest(value: &str) -> Self {
        match value {
            "idle" => Self::Idle,
            "sorted" => Self::Sorted,
            "selected" => Self::Selected,
            "active-cell" => Self::ActiveCell,
            "active-header" => Self::ActiveHeader,
            other => panic!("unmapped DataTable screenshot state: {other}"),
        }
    }
}

fn columns() -> Vec<Column> {
    vec![
        Column::new("name", "Name").width(180),
        Column::new("type", "Type").width(130),
        Column::new("modified", "Modified").width(130),
    ]
}

fn rows() -> Vec<DataRow> {
    vec![
        DataRow::new("row-c", vec!["Meeting notes.md".into(), "Markdown".into(), "Today".into()]),
        DataRow::new("row-a", vec!["Brand guide.pdf".into(), "PDF".into(), "Yesterday".into()]),
        DataRow::new("row-b", vec!["Budget.xlsx".into(), "Spreadsheet".into(), "Sep 18".into()]),
        DataRow::new("row-d", vec!["Project plan.md".into(), "Markdown".into(), "Sep 17".into()]),
    ]
}

struct DataTableFixture {
    table: Option<Entity<DataTable>>,
    active_events: Rc<RefCell<Vec<ActiveCellChanged>>>,
    _active_subscription: Option<Subscription>,
}

impl DataTableFixture {
    fn new() -> Self {
        Self {
            table: None,
            active_events: Rc::new(RefCell::new(Vec::new())),
            _active_subscription: None,
        }
    }
}

impl Render for DataTableFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        if self.table.is_none() {
            self.table = Some(cx.new(|_| DataTable::new("Recent files", columns(), rows())));
        }
        if self._active_subscription.is_none() {
            let table = self.table.as_ref().expect("DataTable initialized");
            let events = self.active_events.clone();
            self._active_subscription =
                Some(cx.subscribe(table, move |_, _, event: &ActiveCellChanged, _| {
                    events.borrow_mut().push(event.clone())
                }));
        }
        div()
            .size_full()
            .bg(theme.colors.background)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.heading_small))
                    .child("Data Table"),
            )
            .child(div().w(px(440.0)).child(self.table.as_ref().unwrap().clone()))
    }
}

fn dispatch(window: &mut Window, key: &str, cx: &mut App) {
    window.dispatch_keystroke(
        Keystroke::parse(key).unwrap_or_else(|error| panic!("{key}: {error}")),
        cx,
    );
}

fn capture(state: State, theme_value: Theme, scale: u32) -> Result<RgbaImage, ScreenshotError> {
    let mut session = HeadlessSession::new(
        DataTableFixture::new(),
        size(px(SIZE.0), px(SIZE.1)),
        scale as f32,
        |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, theme_value);
            cx.bind_keys(data_table::default_key_bindings());
        },
    )?;

    if state == State::Idle {
        session.update(|root, _, cx| {
            let fixture = root.read(cx);
            let table = fixture.table.as_ref().expect("DataTable initialized").read(cx);
            assert_eq!(table.sort(), None);
            assert!(table.selection().is_empty());
            assert!(fixture.active_events.borrow().is_empty());
        })?;
    } else {
        session.update(|root, window, cx| {
            let table = root.read(cx).table.as_ref().expect("DataTable initialized").clone();
            table.update(cx, |table, cx| table.focus_handle(cx).focus(window, cx));
            match state {
                State::Sorted => dispatch(window, "enter", cx),
                State::Selected => {
                    dispatch(window, "down", cx);
                    dispatch(window, "enter", cx);
                }
                State::ActiveCell => {
                    dispatch(window, "down", cx);
                    dispatch(window, "right", cx);
                }
                State::ActiveHeader => dispatch(window, "end", cx),
                State::Idle => unreachable!(),
            }
        })?;

        session.update(|root, window, cx| {
            let table = root.read(cx).table.as_ref().expect("DataTable initialized").clone();
            let focused = table.update(cx, |table, cx| table.focus_handle(cx).is_focused(window));
            assert!(focused, "{} fixture must retain table keyboard focus", state.id());
            let table_state = table.read(cx);
            let events = root.read(cx).active_events.borrow();
            match state {
                State::Sorted => {
                    assert_eq!(table_state.sort(), Some(("name", SortDirection::Ascending)));
                    assert!(table_state.selection().is_empty());
                }
                State::Selected => {
                    assert_eq!(table_state.selection(), &HashSet::from(["row-c".to_owned()]));
                }
                State::ActiveCell => {
                    assert!(table_state.selection().is_empty());
                    let event = events.last().expect("ArrowDown/ArrowRight emits active cell");
                    assert_eq!(event.row.as_deref(), Some("row-c"));
                    assert_eq!(event.column, "type");
                }
                State::ActiveHeader => {
                    assert_eq!(table_state.sort(), None);
                    assert!(table_state.selection().is_empty());
                    let event = events.last().expect("End emits active header coordinate");
                    assert_eq!(event.row, None);
                    assert_eq!(event.column, "modified");
                }
                State::Idle => unreachable!(),
            }
        })?;
    }
    session.capture()
}

fn compare_or_update(actual: &RgbaImage, baseline: &str) {
    if let Some(directory) = std::env::var_os("DATA_TABLE_CAPTURE_DIR") {
        let path = PathBuf::from(directory).join(baseline);
        fs::create_dir_all(path.parent().expect("capture parent")).expect("create capture dir");
        actual.save(&path).expect("write candidate screenshot");
        return;
    }
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/data-table/tests/baselines")
        .join(baseline.strip_prefix("data-table/").expect("DataTable baseline prefix"));
    if std::env::var_os("UPDATE_SNAPSHOTS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        fs::create_dir_all(path.parent().expect("baseline parent")).expect("create baseline dir");
        actual.save(&path).expect("write baseline");
        println!("Updated DataTable screenshot: {}", path.display());
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| panic!("missing DataTable baseline {}: {error}", path.display()))
        .to_rgba8();
    let tolerance = PixelTolerance { channel_delta: 2, max_different_pixels: 32 };
    if !tolerance.matches(&expected, actual) {
        let diff = path.with_file_name(format!(
            "{}-diff.png",
            path.file_stem().expect("baseline stem").to_string_lossy()
        ));
        let mut diff_image = RgbaImage::new(actual.width(), actual.height());
        for (x, y, pixel) in diff_image.enumerate_pixels_mut() {
            let before = expected.get_pixel(x, y);
            let after = actual.get_pixel(x, y);
            *pixel = image::Rgba([
                before[0].abs_diff(after[0]).saturating_mul(4),
                before[1].abs_diff(after[1]).saturating_mul(4),
                before[2].abs_diff(after[2]).saturating_mul(4),
                255,
            ]);
        }
        diff_image.save(&diff).expect("write screenshot diff");
        panic!("DataTable screenshot differs: {}; diff: {}", path.display(), diff.display());
    }
    println!("Matched DataTable screenshot: {baseline}");
}

fn run() -> Result<(), ScreenshotError> {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../registry/data-table/tests/conformance.json"))
            .expect("DataTable manifest JSON");
    let cases = manifest["screenshot_cases"].as_array().expect("screenshot cases");
    assert_eq!(cases.len(), 5 * 3 * 2, "all state/theme/scale combinations must be declared");
    let mut seen = HashSet::new();
    let mut captures = HashMap::new();
    for case in cases {
        let state_name = case["state"].as_str().expect("state");
        let state = State::from_manifest(state_name);
        let theme_name = case["theme"].as_str().expect("theme");
        let theme_value = match theme_name {
            "light" => SHADCN_LIGHT,
            "dark" => SHADCN_DARK,
            "high-contrast" => HIGH_CONTRAST,
            other => panic!("unmapped DataTable screenshot theme: {other}"),
        };
        let scale = u32::try_from(case["scale"].as_u64().expect("scale")).expect("u32 scale");
        assert!(matches!(scale, 1 | 2));
        assert_eq!(case["load_fixture"], format!("{}_fixture", state.id().replace('-', "_")));
        assert_eq!(case["platform"], "macos");
        assert!(seen.insert((state_name, theme_name, scale)), "duplicate screenshot case");
        let baseline = case["baseline"].as_str().expect("baseline");
        let actual = capture(state, theme_value, scale)?;
        assert_eq!(actual.dimensions(), (SIZE.0 as u32 * scale, SIZE.1 as u32 * scale));
        compare_or_update(&actual, baseline);
        captures.insert((state, theme_name.to_owned(), scale), actual);
    }
    assert_eq!(seen.len(), 30);
    for theme in ["light", "dark", "high-contrast"] {
        for scale in [1, 2] {
            let idle = captures.get(&(State::Idle, theme.to_owned(), scale)).unwrap();
            let active_header =
                captures.get(&(State::ActiveHeader, theme.to_owned(), scale)).unwrap();
            assert_ne!(
                idle.as_raw(),
                active_header.as_raw(),
                "active-header must be visually distinct from idle for {theme} {scale}x"
            );
            let selected = captures.get(&(State::Selected, theme.to_owned(), scale)).unwrap();
            let active_cell = captures.get(&(State::ActiveCell, theme.to_owned(), scale)).unwrap();
            assert_ne!(
                selected.as_raw(),
                active_cell.as_raw(),
                "active-cell must be visually distinct from selected for {theme} {scale}x"
            );
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(ScreenshotError::UnsupportedPlatform) => {
            println!("Skipping DataTable screenshot matrix: macOS Metal is required.");
        }
        Err(error) => panic!("DataTable screenshot matrix failed: {error}"),
    }
}
