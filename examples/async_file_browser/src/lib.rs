//! A read-only file browser demonstrating executors, weak entity updates, and filesystem results.

use gpui_pre::{
    App, AppContext, AsyncApp, Context, Global, Render, Task, WeakEntity, Window, div, prelude::*,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

// ANCHOR: executors_and_tasks
/// Schedule trivial work on the main-thread executor.
pub fn foreground_example(cx: &App) -> Task<&'static str> {
    cx.foreground_executor().spawn(async { "foreground task finished" })
}

/// Schedule owned work on the background executor.
pub fn background_example(cx: &App) -> Task<&'static str> {
    cx.background_executor().spawn(async { "background task finished" })
}

/// A timer is also a task. Dropping the handle cancels its future; detach when completion is intentional.
pub fn example_timer(cx: &App, delay: Duration) -> Task<()> {
    cx.background_executor().timer(delay)
}
// ANCHOR_END: executors_and_tasks

// ANCHOR: file_browser_state
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserEntry {
    pub name: String,
    pub kind: EntryKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowserStatus {
    Loading,
    Loaded,
    Empty,
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadDelivery {
    Applied,
    Stale,
    Released,
}

/// Read-only model for one chosen directory. It never navigates outside that root.
pub struct FileBrowser {
    root: PathBuf,
    status: BrowserStatus,
    entries: Vec<BrowserEntry>,
    request_generation: u64,
}

impl FileBrowser {
    pub fn new(root: PathBuf) -> Self {
        Self { root, status: BrowserStatus::Loading, entries: Vec::new(), request_generation: 0 }
    }

    /// Construct a deterministic view state for screenshot previews.
    pub fn preview(root: PathBuf, result: Result<Vec<BrowserEntry>, String>) -> Self {
        let mut browser = Self::new(root);
        browser.apply_result(result);
        browser
    }

    pub fn status(&self) -> &BrowserStatus {
        &self.status
    }

    pub fn entries(&self) -> &[BrowserEntry] {
        &self.entries
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
// ANCHOR_END: file_browser_state

// ANCHOR: file_browser_fs
/// Read one selected directory, omit hidden entries when configured, skip symlinks,
/// and sort names because `read_dir` order is unspecified.
pub fn read_directory(root: &Path, show_hidden: bool) -> Result<Vec<BrowserEntry>, String> {
    let directory =
        fs::read_dir(root).map_err(|_| "Could not read the selected directory.".to_owned())?;
    let mut entries = Vec::new();
    for entry in directory {
        let entry = entry.map_err(|_| "Could not read a directory entry.".to_owned())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let file_type =
            entry.file_type().map_err(|_| "Could not inspect a directory entry.".to_owned())?;
        if file_type.is_symlink() {
            continue;
        }
        let kind = if file_type.is_dir() { EntryKind::Directory } else { EntryKind::File };
        entries.push(BrowserEntry { name, kind });
    }
    entries.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(entries)
}
// ANCHOR_END: file_browser_fs

// ANCHOR: file_browser_load
impl FileBrowser {
    /// Start a background directory read, then apply its owned result through a weak view handle.
    /// Request generations keep older completions from replacing a newer directory listing.
    pub fn reload(&mut self, cx: &mut Context<Self>) -> Task<LoadDelivery> {
        let root = self.root.clone();
        let settings = cx.read_global::<BrowserSettings, _>(|settings, _| *settings);
        let read_root = root.clone();
        let background_work =
            cx.background_spawn(async move { read_directory(&read_root, settings.show_hidden) });
        self.begin_request(cx, root, background_work)
    }

    fn begin_request(
        &mut self,
        cx: &mut Context<Self>,
        root: PathBuf,
        background_work: Task<Result<Vec<BrowserEntry>, String>>,
    ) -> Task<LoadDelivery> {
        self.request_generation += 1;
        let generation = self.request_generation;
        self.root = root;
        self.status = BrowserStatus::Loading;
        self.entries.clear();
        cx.notify();

        cx.spawn(async move |weak: WeakEntity<Self>, async_cx: &mut AsyncApp| {
            let result = background_work.await;
            match weak.update(async_cx, |browser, cx| {
                if browser.request_generation != generation {
                    return false;
                }
                browser.apply_result(result);
                cx.notify();
                true
            }) {
                Ok(true) => LoadDelivery::Applied,
                Ok(false) => LoadDelivery::Stale,
                Err(_) => LoadDelivery::Released,
            }
        })
    }

    fn apply_result(&mut self, result: Result<Vec<BrowserEntry>, String>) {
        match result {
            Ok(entries) if entries.is_empty() => {
                self.entries.clear();
                self.status = BrowserStatus::Empty;
            }
            Ok(entries) => {
                self.entries = entries;
                self.status = BrowserStatus::Loaded;
            }
            Err(error) => {
                self.entries.clear();
                self.status = BrowserStatus::Error(error);
            }
        }
    }
}
// ANCHOR_END: file_browser_load

// ANCHOR: file_browser_settings
/// In-memory GPUI settings shared by app contexts; this value is not disk persistence.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrowserSettings {
    pub show_hidden: bool,
}

impl Global for BrowserSettings {}

/// Install the example's in-memory settings before starting directory loads.
pub fn install_browser_settings(cx: &mut App) {
    cx.set_global(BrowserSettings::default());
}

/// A fixed, harmless directory shipped with this example. The runnable stays away from user paths.
pub fn sample_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sample")
}
// ANCHOR_END: file_browser_settings

// ANCHOR: file_browser_view
impl Render for FileBrowser {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &gpui_kit::component::ActiveTheme::theme(&**cx).colors;
        let refresh = cx.listener(|browser, _, _, cx| browser.reload(cx).detach());
        let content = match &self.status {
            BrowserStatus::Loading => div()
                .flex()
                .items_center()
                .gap_3()
                .text_color(colors.muted_foreground)
                .child("◌")
                .child("Reading this example directory…"),
            BrowserStatus::Loaded => {
                div().flex().flex_col().gap_2().children(self.entries.iter().map(|entry| {
                    let icon = match entry.kind {
                        EntryKind::Directory => "▰",
                        EntryKind::File => "▤",
                    };
                    div()
                        .id(format!("entry-{}", entry.name))
                        .flex()
                        .items_center()
                        .gap_3()
                        .rounded_md()
                        .border_1()
                        .border_color(colors.border)
                        .bg(colors.popover)
                        .px_4()
                        .py_3()
                        .child(div().text_color(colors.primary).child(icon))
                        .child(entry.name.clone())
                }))
            }
            BrowserStatus::Empty => div()
                .rounded_lg()
                .bg(colors.muted)
                .p_5()
                .text_color(colors.muted_foreground)
                .child("This directory is empty."),
            BrowserStatus::Error(message) => div()
                .rounded_lg()
                .bg(colors.danger)
                .p_5()
                .text_color(colors.danger_foreground)
                .child(message.clone()),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(colors.border)
                    .px_8()
                    .py_5()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(gpui_pre::FontWeight::BOLD)
                                    .child("Files"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.muted_foreground)
                                    .child("Sample workspace · read only"),
                            ),
                    )
                    .child(
                        div()
                            .id("reload-directory")
                            .on_click(refresh)
                            .rounded_md()
                            .bg(colors.primary)
                            .text_color(colors.primary_foreground)
                            .px_4()
                            .py_2()
                            .child("Reload"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_8()
                    .py_4()
                    .child(
                        div()
                            .font_weight(gpui_pre::FontWeight::SEMIBOLD)
                            .child("Directory contents"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.muted_foreground)
                            .child(format!("{} items", self.entries.len())),
                    ),
            )
            .child(
                div()
                    .id("file-browser-scroll-region")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_8()
                    .pb_8()
                    .child(content),
            )
    }
}
// ANCHOR_END: file_browser_view

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::oneshot;
    use gpui_pre::TestAppContext;
    use std::{
        fs,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };

    fn install(cx: &mut TestAppContext) {
        cx.update(install_browser_settings);
    }

    fn write_fixture(root: &Path) {
        fs::create_dir_all(root.join("Guides")).unwrap();
        fs::write(root.join("zeta.txt"), "fixture").unwrap();
        fs::write(root.join("Alpha.md"), "fixture").unwrap();
        fs::write(root.join(".hidden"), "fixture").unwrap();
    }

    #[gpui_pre::test]
    fn read_directory_sorts_filters_and_reports_missing_paths(cx: &mut TestAppContext) {
        let fixture = tempfile::tempdir().unwrap();
        write_fixture(fixture.path());
        let entries = read_directory(fixture.path(), false).unwrap();
        assert_eq!(
            entries.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>(),
            ["Alpha.md", "Guides", "zeta.txt"]
        );
        assert!(
            read_directory(fixture.path(), true)
                .unwrap()
                .iter()
                .any(|entry| entry.name == ".hidden")
        );
        let missing = fixture.path().join("missing");
        assert_eq!(
            read_directory(&missing, false).unwrap_err(),
            "Could not read the selected directory."
        );
        let empty = tempfile::tempdir().unwrap();
        assert!(read_directory(empty.path(), false).unwrap().is_empty());
        let _ = cx;
    }

    #[gpui_pre::test]
    async fn async_load_updates_sorted_state_and_handles_missing_path(cx: &mut TestAppContext) {
        install(cx);
        let fixture = tempfile::tempdir().unwrap();
        write_fixture(fixture.path());
        let browser = cx.update(|app| app.new(|_| FileBrowser::new(fixture.path().to_path_buf())));
        let task = browser.update(cx, |browser, cx| browser.reload(cx));
        cx.run_until_parked();
        assert!(task.is_ready());
        assert_eq!(
            browser.read_with(cx, |browser, _| browser.status().clone()),
            BrowserStatus::Loaded
        );
        assert_eq!(
            browser.read_with(cx, |browser, _| browser.entries()[0].name.clone()),
            "Alpha.md"
        );

        let missing = fixture.path().join("deleted");
        let task = browser.update(cx, |browser, cx| {
            browser.root = missing;
            browser.reload(cx)
        });
        cx.run_until_parked();
        assert!(task.is_ready());
        assert!(matches!(
            browser.read_with(cx, |browser, _| browser.status().clone()),
            BrowserStatus::Error(_)
        ));
    }

    #[gpui_pre::test]
    async fn weak_update_reports_released_view_after_background_work_completes(
        cx: &mut TestAppContext,
    ) {
        install(cx);
        let browser = cx.update(|app| app.new(|_| FileBrowser::new(PathBuf::from("fixture"))));
        let (send, receive) = oneshot::channel();
        let work = cx.update(|app| {
            app.background_executor().spawn(async move {
                receive.await.unwrap_or_else(|_| Err("controlled fixture was cancelled".into()))
            })
        });
        let delivery = browser
            .update(cx, |browser, cx| browser.begin_request(cx, PathBuf::from("fixture"), work));
        drop(browser);
        send.send(Ok(Vec::new())).unwrap();
        cx.run_until_parked();
        assert!(delivery.is_ready());
        assert_eq!(delivery.await, LoadDelivery::Released);
    }

    #[gpui_pre::test]
    async fn older_listing_cannot_replace_newer_result(cx: &mut TestAppContext) {
        install(cx);
        let browser = cx.update(|app| app.new(|_| FileBrowser::new(PathBuf::from("initial"))));
        let (send_old, receive_old) = oneshot::channel();
        let old_work = cx.update(|app| {
            app.background_executor().spawn(async move {
                receive_old.await.unwrap_or_else(|_| Err("old listing cancelled".into()))
            })
        });
        let old_delivery = browser
            .update(cx, |browser, cx| browser.begin_request(cx, PathBuf::from("older"), old_work));

        let (send_new, receive_new) = oneshot::channel();
        let new_work = cx.update(|app| {
            app.background_executor().spawn(async move {
                receive_new.await.unwrap_or_else(|_| Err("new listing cancelled".into()))
            })
        });
        let new_delivery = browser
            .update(cx, |browser, cx| browser.begin_request(cx, PathBuf::from("newer"), new_work));

        send_new
            .send(Ok(vec![BrowserEntry { name: "new.txt".into(), kind: EntryKind::File }]))
            .unwrap();
        cx.run_until_parked();
        send_old
            .send(Ok(vec![BrowserEntry { name: "old.txt".into(), kind: EntryKind::File }]))
            .unwrap();
        cx.run_until_parked();

        assert!(old_delivery.is_ready() && new_delivery.is_ready());
        assert_eq!(old_delivery.await, LoadDelivery::Stale);
        assert_eq!(new_delivery.await, LoadDelivery::Applied);
        assert_eq!(
            browser.read_with(cx, |browser, _| browser.root().to_path_buf()),
            PathBuf::from("newer")
        );
        assert_eq!(
            browser.read_with(cx, |browser, _| browser.entries()[0].name.clone()),
            "new.txt"
        );
    }

    #[gpui_pre::test]
    async fn executor_tasks_cancellation_detach_and_test_clock(cx: &mut TestAppContext) {
        let foreground = Arc::new(AtomicBool::new(false));
        let foreground_for_task = foreground.clone();
        let foreground_task = cx.update(|app| {
            app.foreground_executor().spawn(async move {
                foreground_for_task.store(true, Ordering::SeqCst);
            })
        });
        cx.run_until_parked();
        assert!(foreground_task.is_ready());
        assert!(foreground.load(Ordering::SeqCst));

        let background_task = cx.update(|app| background_example(app));
        cx.run_until_parked();
        assert!(background_task.is_ready());

        let cancelled_effect = Arc::new(AtomicBool::new(false));
        let cancelled_flag = cancelled_effect.clone();
        let (cancel_send, cancel_receive) = oneshot::channel::<()>();
        let (started_send, started_receive) = oneshot::channel::<()>();
        let cancelled = cx.update(|app| {
            app.background_executor().spawn(async move {
                let _ = started_send.send(());
                let _ = cancel_receive.await;
                cancelled_flag.store(true, Ordering::SeqCst);
            })
        });
        cx.run_until_parked();
        started_receive.await.expect("cancellable task reached its pending wait");
        drop(cancelled);
        let _ = cancel_send.send(());
        cx.run_until_parked();
        assert!(!cancelled_effect.load(Ordering::SeqCst));

        let detached_effect = Arc::new(AtomicBool::new(false));
        let detached_flag = detached_effect.clone();
        cx.update(|app| {
            app.background_executor().spawn(async move {
                detached_flag.store(true, Ordering::SeqCst);
            })
        })
        .detach();
        cx.run_until_parked();
        assert!(detached_effect.load(Ordering::SeqCst));

        let timer = cx.update(|app| example_timer(app, Duration::from_secs(5)));
        assert!(!timer.is_ready());
        cx.executor().advance_clock(Duration::from_secs(5));
        cx.run_until_parked();
        assert!(timer.is_ready());
    }
}
