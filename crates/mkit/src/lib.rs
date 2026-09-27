//! Components and foundations for GPUI applications.

#[cfg(test)]
extern crate gpui_pre as gpui;

pub use mkit_core as core;

#[cfg(feature = "combobox")]
#[path = "generated/combobox.rs"]
pub mod combobox;

#[cfg(feature = "scrubbable-number-field")]
#[path = "generated/scrubbable_number_field.rs"]
pub mod scrubbable_number_field;

#[cfg(feature = "button")]
#[path = "generated/button.rs"]
pub mod button;

#[cfg(feature = "icon-button")]
#[path = "generated/icon_button.rs"]
pub mod icon_button;

#[cfg(feature = "toggle-button")]
#[path = "generated/toggle_button.rs"]
pub mod toggle_button;

#[cfg(feature = "toggle-group")]
#[path = "generated/toggle_group.rs"]
pub mod toggle_group;

#[cfg(feature = "checkbox")]
#[path = "generated/checkbox.rs"]
pub mod checkbox;

#[cfg(feature = "radio-group")]
#[path = "generated/radio_group.rs"]
pub mod radio_group;

#[cfg(feature = "switch")]
#[path = "generated/switch.rs"]
pub mod switch;

#[cfg(feature = "slider")]
#[path = "generated/slider.rs"]
pub mod slider;

#[cfg(feature = "progress")]
#[path = "generated/progress.rs"]
pub mod progress;

#[cfg(feature = "text-field")]
#[path = "generated/text_field.rs"]
pub mod text_field;

#[cfg(feature = "text-area")]
#[path = "generated/text_area.rs"]
pub mod text_area;

#[cfg(feature = "breadcrumbs")]
#[path = "generated/breadcrumbs.rs"]
pub mod breadcrumbs;

#[cfg(feature = "context-menu")]
#[path = "generated/context_menu.rs"]
pub mod context_menu;

#[cfg(feature = "data-table")]
#[path = "generated/data_table.rs"]
pub mod data_table;

#[cfg(feature = "dialog")]
#[path = "generated/dialog.rs"]
pub mod dialog;

#[cfg(feature = "dropdown-menu")]
#[path = "generated/dropdown_menu.rs"]
pub mod dropdown_menu;

#[cfg(feature = "form-layout")]
#[path = "generated/form_layout.rs"]
pub mod form_layout;

#[cfg(feature = "multi-select")]
#[path = "generated/multi_select.rs"]
pub mod multi_select;

#[cfg(feature = "popover")]
#[path = "generated/popover.rs"]
pub mod popover;

#[cfg(feature = "scroll-area")]
#[path = "generated/scroll_area.rs"]
pub mod scroll_area;

#[cfg(feature = "segmented-control")]
#[path = "generated/segmented_control.rs"]
pub mod segmented_control;

#[cfg(feature = "select")]
#[path = "generated/select.rs"]
pub mod select;

#[cfg(feature = "separator")]
#[path = "generated/separator.rs"]
pub mod separator;

#[cfg(feature = "sheet")]
#[path = "generated/sheet.rs"]
pub mod sheet;

#[cfg(feature = "sidebar")]
#[path = "generated/sidebar.rs"]
pub mod sidebar;

#[cfg(feature = "split-pane")]
#[path = "generated/split_pane.rs"]
pub mod split_pane;

#[cfg(feature = "tabs")]
#[path = "generated/tabs.rs"]
pub mod tabs;

#[cfg(feature = "toast")]
#[path = "generated/toast.rs"]
pub mod toast;

#[cfg(feature = "tooltip")]
#[path = "generated/tooltip.rs"]
pub mod tooltip;

#[cfg(feature = "tree")]
#[path = "generated/tree.rs"]
pub mod tree;

#[cfg(feature = "virtual-list")]
#[path = "generated/virtual_list.rs"]
pub mod virtual_list;

#[cfg(feature = "viewport")]
#[path = "generated/viewport.rs"]
pub mod viewport;

#[cfg(feature = "histogram")]
#[path = "generated/histogram.rs"]
pub mod histogram;

#[cfg(feature = "precision-slider")]
#[path = "generated/precision_slider.rs"]
pub mod precision_slider;

#[cfg(feature = "command-palette")]
#[path = "generated/command_palette.rs"]
pub mod command_palette;

#[cfg(feature = "curve-editor")]
#[path = "generated/curve_editor.rs"]
pub mod curve_editor;

#[cfg(feature = "colour-tools")]
#[path = "generated/colour_tools.rs"]
pub mod colour_tools;

#[cfg(feature = "property-inspector")]
#[path = "generated/property_inspector.rs"]
pub mod property_inspector;

#[cfg(feature = "layer-panel")]
#[path = "generated/layer_panel.rs"]
pub mod layer_panel;

#[cfg(feature = "gradient-editor")]
#[path = "generated/gradient_editor.rs"]
pub mod gradient_editor;

#[cfg(feature = "shortcut-editor")]
#[path = "generated/shortcut_editor.rs"]
pub mod shortcut_editor;

#[cfg(feature = "timeline")]
#[path = "generated/timeline.rs"]
pub mod timeline;

#[cfg(feature = "node-editor")]
#[path = "generated/node_editor.rs"]
pub mod node_editor;

#[cfg(feature = "calendar")]
#[path = "generated/calendar.rs"]
pub mod calendar;

#[cfg(feature = "disclosure")]
#[path = "generated/disclosure.rs"]
pub mod disclosure;

#[cfg(feature = "accordion")]
#[path = "generated/accordion.rs"]
pub mod accordion;

#[cfg(feature = "toolbar")]
#[path = "generated/toolbar.rs"]
pub mod toolbar;

#[cfg(feature = "inline-alert")]
#[path = "generated/inline_alert.rs"]
pub mod inline_alert;

#[cfg(feature = "empty-state")]
#[path = "generated/empty_state.rs"]
pub mod empty_state;

#[cfg(feature = "date-picker")]
#[path = "generated/date_picker.rs"]
pub mod date_picker;

#[cfg(feature = "time-field")]
#[path = "generated/time_field.rs"]
pub mod time_field;

#[cfg(feature = "date-time-field")]
#[path = "generated/date_time_field.rs"]
pub mod date_time_field;

#[cfg(feature = "search-field")]
#[path = "generated/search_field.rs"]
pub mod search_field;

#[cfg(feature = "status-bar")]
#[path = "generated/status_bar.rs"]
pub mod status_bar;

#[cfg(feature = "badge")]
#[path = "generated/badge.rs"]
pub mod badge;

#[cfg(feature = "avatar")]
#[path = "generated/avatar.rs"]
pub mod avatar;

#[cfg(feature = "key-hint")]
#[path = "generated/key_hint.rs"]
pub mod key_hint;

#[cfg(feature = "link")]
#[path = "generated/link.rs"]
pub mod link;

#[cfg(feature = "tag-input")]
#[path = "generated/tag_input.rs"]
pub mod tag_input;

#[cfg(feature = "file-drop-zone")]
#[path = "generated/file_drop_zone.rs"]
pub mod file_drop_zone;

#[cfg(feature = "file-field")]
#[path = "generated/file_field.rs"]
pub mod file_field;

#[cfg(feature = "stepper")]
#[path = "generated/stepper.rs"]
pub mod stepper;

#[cfg(feature = "menu-bar")]
#[path = "generated/menu_bar.rs"]
pub mod menu_bar;

#[cfg(feature = "reorderable-list")]
#[path = "generated/reorderable_list.rs"]
pub mod reorderable_list;
