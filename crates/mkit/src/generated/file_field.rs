//! Labeled file selection field with browse and per-file removal controls.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement, KeyBinding,
    PathBuilder, PathPromptOptions, Render, Rgba, Window, actions, canvas, div, point, prelude::*,
    px,
};
use mkit_core::{
    a11y::AccessibilityExt,
    contrast::composite,
    theme::{ShadowToken, Theme},
};
use std::path::{Path, PathBuf};

/// Resolved button and row colours; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    high_contrast: bool,
    background: Rgba,
    text: Rgba,
    text_muted: Rgba,
    icon: Rgba,
    button_bg: Rgba,
    button_fg: Rgba,
    button_border: Rgba,
    button_hover_bg: Option<Rgba>,
    button_hover_border: Option<Rgba>,
    danger: Rgba,
    danger_hover: Option<Rgba>,
    focus: Rgba,
    ring: Rgba,
    disabled: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            high_contrast: true,
            background: c.background,
            text: c.text,
            text_muted: c.text_muted,
            icon: c.text,
            button_bg: c.accent,
            button_fg: c.accent_text,
            button_border: c.accent,
            button_hover_bg: None,
            button_hover_border: Some(c.text),
            danger: c.danger,
            danger_hover: None,
            focus: c.focus,
            ring: c.focus,
            disabled: c.disabled,
        };
    }
    // The shadcn roles used here (primary, muted-foreground, destructive) map to the same
    // tokens in light and dark themes, so no luminance split is needed.
    Look {
        high_contrast: false,
        background: c.background,
        text: c.text,
        text_muted: c.text_muted,
        icon: c.text_muted,
        button_bg: c.accent,
        button_fg: c.accent_text,
        button_border: c.background.opacity(0.),
        button_hover_bg: Some(mix(c.accent, c.background, 0.9)),
        button_hover_border: None,
        danger: c.danger,
        danger_hover: Some(mix(c.danger, c.background, 0.1)),
        focus: c.focus,
        ring: c.focus.opacity(0.5),
        disabled: c.disabled,
    }
}
/// The web preview's `opacity: .5` applied as one layer: composite over `base`, then mix 50%.
fn dim(color: Rgba, base: Rgba) -> Rgba {
    mix(composite(color, base), base, 0.5)
}
fn box_shadow(shadow: ShadowToken, alpha: f32) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: Rgba { a: shadow.color.a * alpha, ..shadow.color }.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}
/// shadcn/ui focus ring width, drawn outside the control.
const FOCUS_RING_WIDTH: f32 = 3.0;
fn focus_ring(color: Rgba) -> gpui_pre::BoxShadow {
    gpui_pre::BoxShadow {
        color: color.into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(FOCUS_RING_WIDTH),
        inset: false,
    }
}
/// One Lucide path command on the 24-unit icon grid.
#[derive(Clone, Copy)]
enum Seg {
    Move(f32, f32),
    Line(f32, f32),
    /// Small counter-clockwise arc of the given radius (SVG `a r r 0 0 0`) to an absolute point.
    Arc(f32, f32, f32),
    Close,
}
/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Paths use a
/// 24-unit grid; the stroke is 2 units, Lucide's default, and rounded corners are arcs.
fn icon(size: f32, segs: &'static [Seg], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let at = |x: f32, y: f32| origin + point(unit * x, unit * y);
            let mut path = PathBuilder::stroke(unit * 2.0);
            for seg in segs {
                match *seg {
                    Seg::Move(x, y) => path.move_to(at(x, y)),
                    Seg::Line(x, y) => path.line_to(at(x, y)),
                    Seg::Arc(r, x, y) => {
                        path.arc_to(point(unit * r, unit * r), px(0.), false, false, at(x, y))
                    }
                    Seg::Close => path.close(),
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `file`, shown before each selected file name.
const FILE: &[Seg] = &[
    Seg::Move(15., 2.),
    Seg::Line(6., 2.),
    Seg::Arc(2., 4., 4.),
    Seg::Line(4., 20.),
    Seg::Arc(2., 6., 22.),
    Seg::Line(18., 22.),
    Seg::Arc(2., 20., 20.),
    Seg::Line(20., 7.),
    Seg::Close,
    Seg::Move(14., 2.),
    Seg::Line(14., 6.),
    Seg::Arc(2., 16., 8.),
    Seg::Line(20., 8.),
];

pub const KEY_CONTEXT: &str = "FileField";
pub const REMOVE_KEY_CONTEXT: &str = "FileFieldRemove";
actions!(file_field, [Browse, Remove]);

pub fn default_key_bindings() -> [KeyBinding; 4] {
    [
        KeyBinding::new("enter", Browse, Some(KEY_CONTEXT)),
        KeyBinding::new("space", Browse, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", Remove, Some(REMOVE_KEY_CONTEXT)),
        KeyBinding::new("space", Remove, Some(REMOVE_KEY_CONTEXT)),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedFile {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: Option<u64>,
}
impl SelectedFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let name = path
            .file_name()
            .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned());
        Self { path, name, size_bytes: None }
    }

    pub fn size_bytes(mut self, size: u64) -> Self {
        self.size_bytes = Some(size);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionSource {
    Browse,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilesChange {
    pub files: Vec<SelectedFile>,
    pub source: SelectionSource,
}
impl EventEmitter<FilesChange> for FileField {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRejected {
    pub path: PathBuf,
    pub reason: String,
}
impl EventEmitter<FileRejected> for FileField {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrowseRequested;
impl EventEmitter<BrowseRequested> for FileField {}

pub struct FileField {
    label: String,
    description: String,
    accept: Vec<String>,
    multiple: bool,
    disabled: bool,
    files: Vec<SelectedFile>,
    controlled: bool,
    focus: Option<FocusHandle>,
    remove_focus: Vec<FocusHandle>,
    feedback: Option<String>,
}

impl FileField {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: String::new(),
            accept: Vec::new(),
            multiple: false,
            disabled: false,
            files: Vec::new(),
            controlled: false,
            focus: None,
            remove_focus: Vec::new(),
            feedback: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    pub fn accept(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.accept = values.into_iter().map(Into::into).collect();
        self
    }
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Set controlled files. The parent must apply emitted `FilesChange` values.
    pub fn files(mut self, files: Vec<SelectedFile>) -> Self {
        self.files = files;
        self.controlled = true;
        self
    }
    /// Seed an uncontrolled selection. Later changes are stored internally.
    pub fn default_files(mut self, files: Vec<SelectedFile>) -> Self {
        self.files = files;
        self
    }
    pub fn set_files(&mut self, files: Vec<SelectedFile>) {
        self.files = files;
    }
    pub fn selected_files(&self) -> &[SelectedFile] {
        &self.files
    }

    fn accepts(&self, path: &Path) -> bool {
        accept_path(&self.accept, path)
    }

    fn apply_paths(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for path in paths {
            if self.accepts(&path) {
                if self.multiple || accepted.is_empty() {
                    accepted.push(SelectedFile::new(path));
                } else {
                    rejected.push(FileRejected {
                        path,
                        reason: "Only one file can be selected".into(),
                    });
                }
            } else {
                rejected.push(FileRejected { path, reason: "File type is not accepted".into() });
            }
        }
        self.feedback = if rejected.is_empty() {
            Some("Files selected".into())
        } else {
            Some(format!("{} file(s) rejected", rejected.len()))
        };
        let mut proposed = if self.multiple { self.files.clone() } else { Vec::new() };
        for file in accepted {
            if !proposed.iter().any(|selected| selected.path == file.path) {
                proposed.push(file);
            }
        }
        if !self.controlled {
            self.files = proposed.clone();
        }
        for rejected in rejected {
            cx.emit(rejected);
        }
        cx.emit(FilesChange { files: proposed, source: SelectionSource::Browse });
        cx.notify();
    }

    fn remove_from_control(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.controlled {
            self.remove(index, cx);
            return;
        }
        let target = self
            .remove_focus
            .get(index + 1)
            .or_else(|| index.checked_sub(1).and_then(|previous| self.remove_focus.get(previous)))
            .cloned()
            .or_else(|| self.focus.clone());
        self.remove(index, cx);
        if let Some(target) = target {
            target.focus(window, cx);
        }
    }

    fn open_picker(&mut self, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        cx.emit(BrowseRequested);
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: self.multiple,
            prompt: Some(self.label.clone().into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await {
                let _ = this.update(cx, |this, cx| this.apply_paths(paths, cx));
            }
        })
        .detach();
    }

    fn browse(&mut self, _: &Browse, _: &mut Window, cx: &mut Context<Self>) {
        self.open_picker(cx);
    }

    fn remove(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.disabled || index >= self.files.len() {
            return;
        }
        let mut proposed = self.files.clone();
        proposed.remove(index);
        if !self.controlled {
            self.files = proposed.clone();
            if index < self.remove_focus.len() {
                self.remove_focus.remove(index);
            }
        }
        self.feedback = Some("File removed".into());
        cx.emit(FilesChange { files: proposed, source: SelectionSource::Remove });
        cx.notify();
    }
}

impl Focusable for FileField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("FileField focus initialized during render")
    }
}

impl Render for FileField {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let label = self.label.clone();
        let description = self.description.clone();
        let files = self.files.clone();
        let disabled = self.disabled;
        let feedback = self.feedback.clone();
        self.remove_focus.resize_with(files.len(), || cx.focus_handle().tab_index(0));
        self.remove_focus.truncate(files.len());
        let remove_focus = self.remove_focus.clone();
        let browse_label = if files.is_empty() { "Browse files" } else { "Change files" };
        let look = look(&theme);
        let bg = look.background;
        // Disabled controls: solid colours in high contrast, otherwise one 50% layer.
        let (button_bg, button_fg, button_border, danger) = if !disabled {
            (look.button_bg, look.button_fg, look.button_border, look.danger)
        } else if look.high_contrast {
            (bg, look.disabled, look.disabled, look.disabled)
        } else {
            (
                dim(look.button_bg, bg),
                dim(look.button_fg, bg),
                look.button_border,
                dim(look.danger, bg),
            )
        };
        let button_shadow = (!look.high_contrast)
            .then(|| box_shadow(theme.shadows.small, if disabled { 0.5 } else { 1.0 }));
        div()
            .id("mkit-file-field")
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(label.clone())
            .when(disabled, |el| {
                el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .flex()
            .flex_col()
            .gap(px(theme.spacing.small))
            .text_size(px(theme.typography.body))
            .child(div().text_color(look.text).font_weight(FontWeight::MEDIUM).child(label))
            .when(!description.is_empty(), |el| {
                el.child(div().text_color(look.text_muted).child(description))
            })
            .child(
                div()
                    .id("mkit-file-field-browse")
                    .key_context(KEY_CONTEXT)
                    .on_action(cx.listener(Self::browse))
                    .tab_index(0)
                    .tab_stop(!disabled)
                    .track_focus(&focus)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(browse_label)
                    .when(disabled, |el| {
                        el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                    })
                    .when(!disabled, |el| {
                        el.on_click(cx.listener(|this, _, _, cx| this.open_picker(cx)))
                    })
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(theme.controls.medium))
                    .px(px(theme.spacing.large))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.regular))
                    .border_color(button_border)
                    .bg(button_bg)
                    .when_some(button_shadow, |el, shadow| el.shadow(vec![shadow]))
                    .text_color(button_fg)
                    .font_weight(FontWeight::MEDIUM)
                    .whitespace_nowrap()
                    .when(!disabled, |el| {
                        el.hover(move |style| {
                            let style = match look.button_hover_bg {
                                Some(color) => style.bg(color),
                                None => style,
                            };
                            match look.button_hover_border {
                                Some(color) => style.border_color(color),
                                None => style,
                            }
                        })
                    })
                    .focus_visible(move |style| {
                        style.border_color(look.focus).shadow(vec![focus_ring(look.ring)])
                    })
                    .child(browse_label),
            )
            .when_some(feedback, |el, feedback| {
                el.child(
                    div()
                        .id("mkit-file-field-feedback")
                        .role(gpui_pre::accesskit::Role::Status)
                        .a11y_live_region(mkit_core::a11y::LiveRegionPriority::Polite)
                        .text_color(look.text_muted)
                        .child(feedback),
                )
            })
            .children(files.into_iter().enumerate().map(|(index, file)| {
                let name = file.name;
                let size = file.size_bytes.map(format_size);
                div()
                    .id(format!("mkit-file-field-row-{index}"))
                    .min_h(px(theme.controls.xsmall))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.small))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(theme.spacing.small))
                            .child(icon(theme.spacing.large, FILE, look.icon))
                            .child(div().text_color(look.text).child(name.clone()))
                            .when_some(size, |el, size| {
                                el.child(
                                    div()
                                        .text_color(look.text_muted)
                                        .text_size(px(theme.typography.caption))
                                        .child(size),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id(format!("mkit-file-field-remove-{index}"))
                            .key_context(REMOVE_KEY_CONTEXT)
                            .on_action(cx.listener(move |this, _: &Remove, window, cx| {
                                this.remove_from_control(index, window, cx)
                            }))
                            .tab_index(0)
                            .tab_stop(!disabled)
                            .track_focus(&remove_focus[index])
                            .role(gpui_pre::accesskit::Role::Button)
                            .aria_label(format!("Remove {name}"))
                            .when(disabled, |el| {
                                el.a11y_synthetic_children(|builder| {
                                    builder.parent_node().set_disabled()
                                })
                            })
                            .when(!disabled, |el| {
                                el.on_click(cx.listener(move |this, _, window, cx| {
                                    this.remove_from_control(index, window, cx)
                                }))
                            })
                            .flex_none()
                            .px(px(theme.spacing.small))
                            .py(px(theme.spacing.xsmall))
                            .rounded(px(theme.radii.small))
                            .text_size(px(theme.typography.caption))
                            .text_color(danger)
                            .when_some(look.danger_hover.filter(|_| !disabled), |el, fill| {
                                el.hover(move |style| style.bg(fill))
                            })
                            .focus_visible(move |style| {
                                style.bg(bg).shadow(vec![focus_ring(look.ring)])
                            })
                            .child("Remove"),
                    )
            }))
    }
}

fn accept_path(accept: &[String], path: &Path) -> bool {
    if accept.is_empty() {
        return true;
    }
    let extension =
        path.extension().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
    accept.iter().any(|filter| {
        let filter = filter.trim().to_ascii_lowercase();
        if let Some(filter) = filter.strip_prefix('.') {
            extension == filter
        } else if filter == "image/*" {
            matches!(
                extension.as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "avif" | "heic"
            )
        } else if filter == "audio/*" {
            matches!(extension.as_str(), "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac")
        } else if filter == "video/*" {
            matches!(extension.as_str(), "mp4" | "mov" | "webm" | "mkv" | "avi")
        } else {
            filter.rsplit_once('/').is_some_and(|(_, suffix)| suffix == extension)
        }
    })
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_suffixes_without_claiming_content_validation() {
        assert!(accept_path(&[".PDF".into()], Path::new("/tmp/report.pdf")));
        assert!(!accept_path(&[".pdf".into()], Path::new("/tmp/report.exe")));
        assert!(accept_path(&["image/*".into()], Path::new("/tmp/photo.webp")));
    }

    #[gpui_pre::test]
    async fn browse_opens_native_file_picker(cx: &mut gpui_pre::TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (view, visual) =
            cx.add_window_view(|_, _| FileField::new("Attachments").multiple(true));
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            view.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter");
        assert!(cx.did_prompt_for_paths());
    }

    #[gpui_pre::test]
    async fn selected_files_filter_rejections_and_controlled_removal(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        let uncontrolled = cx.new(|_| FileField::new("Files").accept([".txt"]).multiple(true));
        uncontrolled.update(cx, |this, cx| {
            this.apply_paths(vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.exe")], cx);
        });
        cx.update(|cx| {
            assert_eq!(uncontrolled.read(cx).selected_files().len(), 1);
            assert_eq!(uncontrolled.read(cx).feedback.as_deref(), Some("1 file(s) rejected"));
        });

        let single = cx.new(|_| FileField::new("Single").multiple(false));
        single.update(cx, |this, cx| {
            this.apply_paths(
                vec![PathBuf::from("/tmp/first.pdf"), PathBuf::from("/tmp/second.pdf")],
                cx,
            );
        });
        cx.update(|cx| {
            assert_eq!(single.read(cx).selected_files().len(), 1);
            assert_eq!(single.read(cx).feedback.as_deref(), Some("1 file(s) rejected"));
        });

        let initial = vec![SelectedFile::new("/tmp/a.txt")];
        let controlled = cx.new(|_| FileField::new("Files").files(initial.clone()));
        controlled.update(cx, |this, cx| this.remove(0, cx));
        cx.update(|cx| assert_eq!(controlled.read(cx).selected_files(), initial.as_slice()));
    }

    #[test]
    fn formats_file_sizes_compactly() {
        assert_eq!(format_size(12), "12 B");
        assert_eq!(format_size(2048), "2.0 KB");
    }
}
