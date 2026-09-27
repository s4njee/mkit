//! OS file drop target with a keyboard-accessible platform picker.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Context, EventEmitter, ExternalPaths, FocusHandle, Focusable, IntoElement, KeyBinding,
    PathPromptOptions, Render, Window, actions, div, prelude::*, px,
};
use mkit_core::{
    a11y::{AccessibilityExt, LiveRegionPriority},
    theme::Theme,
};
use std::path::{Path, PathBuf};

pub const KEY_CONTEXT: &str = "FileDropZone";
pub const REMOVE_KEY_CONTEXT: &str = "FileDropZoneRemove";
actions!(file_drop_zone, [Browse, Remove]);

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
}

impl SelectedFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let name = path
            .file_name()
            .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned());
        Self { path, name }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionSource {
    Drop,
    Browse,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedFile {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilesChange {
    pub files: Vec<SelectedFile>,
    pub rejected: Vec<RejectedFile>,
    pub source: SelectionSource,
}
impl EventEmitter<FilesChange> for FileDropZone {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrowseRequested;
impl EventEmitter<BrowseRequested> for FileDropZone {}

pub struct FileDropZone {
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

impl FileDropZone {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: "Drop files here or browse to choose files".into(),
            accept: Vec::new(),
            multiple: true,
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
        if self.accept.is_empty() {
            return true;
        }
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();
        let extension = extension.to_ascii_lowercase();
        self.accept.iter().any(|filter| {
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

    fn request_paths(
        &mut self,
        paths: Vec<PathBuf>,
        source: SelectionSource,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let previous_len = self.files.len();
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for path in paths {
            if !self.accepts(&path) {
                rejected.push(RejectedFile { path, reason: "File type is not accepted".into() });
            } else if !self.multiple && !accepted.is_empty() {
                rejected
                    .push(RejectedFile { path, reason: "Only one file can be selected".into() });
            } else {
                accepted.push(SelectedFile::new(path));
            }
        }
        let mut proposed = if self.multiple { self.files.clone() } else { Vec::new() };
        for file in accepted {
            if !proposed.iter().any(|selected| selected.path == file.path) {
                proposed.push(file);
            }
        }
        if !self.controlled {
            self.files = proposed.clone();
        }
        self.feedback = if rejected.is_empty() {
            (proposed.len() > previous_len).then(|| "Files added".into())
        } else {
            Some(format!("{} file(s) rejected", rejected.len()))
        };
        cx.emit(FilesChange { files: proposed, rejected, source });
        cx.notify();
    }

    fn browse(&mut self, _: &Browse, _: &mut Window, cx: &mut Context<Self>) {
        self.open_picker(cx);
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
            prompt: Some("Choose files".into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await {
                let _ = this
                    .update(cx, |this, cx| this.request_paths(paths, SelectionSource::Browse, cx));
            }
        })
        .detach();
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
        cx.emit(FilesChange {
            files: proposed,
            rejected: Vec::new(),
            source: SelectionSource::Remove,
        });
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
}

impl Focusable for FileDropZone {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone().expect("FileDropZone focus initialized during render")
    }
}

impl Render for FileDropZone {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let label = self.label.clone();
        let description = self.description.clone();
        let accept = self.accept.clone();
        let disabled = self.disabled;
        let files = self.files.clone();
        self.remove_focus.resize_with(files.len(), || cx.focus_handle().tab_index(0));
        self.remove_focus.truncate(files.len());
        let remove_focus = self.remove_focus.clone();
        let feedback = self.feedback.clone();
        let button_label = if files.is_empty() { "Browse files" } else { "Add files" };
        div()
            .id("mkit-file-drop-zone")
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::browse))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.request_paths(paths.paths().to_vec(), SelectionSource::Drop, cx);
            }))
            .drag_over::<ExternalPaths>(move |style, paths, _, _| {
                if disabled {
                    style
                } else if accept_paths(&accept, paths.paths()) {
                    style.bg(theme.colors.accent.opacity(0.14)).border_color(theme.colors.accent)
                } else {
                    style.bg(theme.colors.danger.opacity(0.14)).border_color(theme.colors.danger)
                }
            })
            .role(gpui_pre::accesskit::Role::Group)
            .aria_label(label.clone())
            .aria_description(description.clone())
            .when(disabled, |el| {
                el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
            })
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .p(px(theme.spacing.large))
            .border(px(theme.borders.regular))
            .border_color(theme.colors.border)
            .rounded(px(theme.radii.medium))
            .bg(theme.colors.surface)
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_size(px(theme.typography.body))
                    .child(description),
            )
            .child(
                div()
                    .id("mkit-file-drop-zone-browse")
                    .key_context(KEY_CONTEXT)
                    .tab_index(0)
                    .tab_stop(!disabled)
                    .track_focus(&focus)
                    .role(gpui_pre::accesskit::Role::Button)
                    .aria_label(button_label)
                    .when(disabled, |el| {
                        el.a11y_synthetic_children(|builder| builder.parent_node().set_disabled())
                    })
                    .when(!disabled, |el| {
                        el.on_click(cx.listener(|this, _, _, cx| this.open_picker(cx)))
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(theme.controls.large))
                    .px(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .bg(if disabled { theme.colors.disabled } else { theme.colors.accent })
                    .text_color(theme.colors.surface)
                    .child(button_label),
            )
            .when_some(feedback, |el, feedback| {
                el.child(
                    div()
                        .id("mkit-file-drop-zone-feedback")
                        .role(gpui_pre::accesskit::Role::Status)
                        .a11y_live_region(LiveRegionPriority::Polite)
                        .text_color(theme.colors.text_muted)
                        .child(feedback),
                )
            })
            .children(files.into_iter().enumerate().map(|(index, file)| {
                let name = file.name;
                div()
                    .id(format!("mkit-file-drop-zone-file-{index}"))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.small))
                    .child(div().text_color(theme.colors.text).child(name.clone()))
                    .child(
                        div()
                            .id(format!("mkit-file-drop-zone-remove-{index}"))
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
                            .text_color(if disabled {
                                theme.colors.disabled
                            } else {
                                theme.colors.danger
                            })
                            .child("Remove"),
                    )
            }))
    }
}

fn accept_paths(accept: &[String], paths: &[PathBuf]) -> bool {
    if accept.is_empty() {
        return true;
    }
    paths.iter().all(|path| {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_filters_are_case_insensitive_and_reject_unknown_suffixes() {
        assert!(accept_paths(&[".PNG".into()], &[PathBuf::from("/tmp/photo.png")]));
        assert!(!accept_paths(&[".png".into()], &[PathBuf::from("/tmp/readme.txt")]));
        assert!(accept_paths(&["image/*".into()], &[PathBuf::from("/tmp/photo.jpeg")]));
        assert!(!accept_paths(&["image/*".into()], &[PathBuf::from("/tmp/movie.mov")]));
    }

    #[gpui_pre::test]
    async fn browse_opens_the_platform_picker(cx: &mut gpui_pre::TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (view, visual) = cx.add_window_view(|_, _| {
            FileDropZone::new("Attachments").accept([".pdf"]).multiple(false)
        });
        visual.update(|window, cx| {
            window.draw(cx).clear(cx);
            view.focus_handle(cx).focus(window, cx);
        });
        visual.simulate_keystrokes("enter");
        assert!(cx.did_prompt_for_paths());
    }

    #[gpui_pre::test]
    async fn remove_button_keyboard_action_updates_selection(cx: &mut gpui_pre::TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let files = vec![SelectedFile::new("/tmp/a.pdf"), SelectedFile::new("/tmp/b.pdf")];
        let (view, visual) =
            cx.add_window_view(|_, _| FileDropZone::new("Files").default_files(files));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| {
            let focus = view.read(cx).remove_focus[0].clone();
            focus.focus(window, cx);
            assert!(focus.is_focused(window));
        });
        visual.simulate_keystrokes("enter");
        assert_eq!(view.read_with(visual, |view, _| view.selected_files().len()), 1);
    }

    #[gpui_pre::test]
    async fn drop_proposes_only_accepted_files_and_controlled_values_stay_parent_owned(
        cx: &mut gpui_pre::TestAppContext,
    ) {
        let uncontrolled = cx.new(|_| FileDropZone::new("Attachments").accept([".pdf"]));
        uncontrolled.update(cx, |this, cx| {
            this.request_paths(
                vec![PathBuf::from("/tmp/ok.pdf"), PathBuf::from("/tmp/no.exe")],
                SelectionSource::Drop,
                cx,
            );
        });
        cx.update(|cx| {
            assert_eq!(uncontrolled.read(cx).selected_files().len(), 1);
            assert_eq!(uncontrolled.read(cx).feedback.as_deref(), Some("1 file(s) rejected"));
        });

        let single = cx.new(|_| FileDropZone::new("Single").multiple(false));
        single.update(cx, |this, cx| {
            this.request_paths(
                vec![PathBuf::from("/tmp/first.pdf"), PathBuf::from("/tmp/second.pdf")],
                SelectionSource::Drop,
                cx,
            );
        });
        cx.update(|cx| {
            assert_eq!(single.read(cx).selected_files().len(), 1);
            assert_eq!(single.read(cx).feedback.as_deref(), Some("1 file(s) rejected"));
        });

        let initial = vec![SelectedFile::new("/tmp/initial.pdf")];
        let controlled = cx.new(|_| FileDropZone::new("Controlled").files(initial.clone()));
        controlled.update(cx, |this, cx| {
            this.request_paths(vec![PathBuf::from("/tmp/next.pdf")], SelectionSource::Drop, cx);
        });
        cx.update(|cx| assert_eq!(controlled.read(cx).selected_files(), initial.as_slice()));
    }
}
