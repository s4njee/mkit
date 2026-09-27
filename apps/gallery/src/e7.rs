//! Visual preview of the everyday component drafts in the neutral shadcn palette.

use gpui_pre::{Context, Entity, Render, Window, div, prelude::*, px};
use mkit::breadcrumbs::{Breadcrumbs, Crumb};
use mkit::button::{Button, Size as ButtonSize, Variant as ButtonVariant};
use mkit::checkbox::Checkbox;
use mkit::combobox::{Combobox, OptionItem as ComboboxOption};
use mkit::context_menu::{ContextMenu, MenuItem as ContextItem};
use mkit::core::theme::{SHADCN_DARK, SHADCN_LIGHT, Theme};
use mkit::data_table::{Column as TableColumn, DataRow, DataTable, SortDirection};
use mkit::dialog::Dialog;
use mkit::dropdown_menu::{DropdownMenu, MenuItem as CommandItem};
use mkit::form_layout::{FormLayout, FormRow};
use mkit::icon_button::{IconButton, Variant as IconButtonVariant};
use mkit::multi_select::{MultiSelect, OptionItem as MultiSelectOption};
use mkit::popover::Popover;
use mkit::progress::Progress;
use mkit::radio_group::{OptionItem, RadioGroup};
use mkit::scroll_area::ScrollArea;
use mkit::segmented_control::{Item as SegmentItem, SegmentedControl};
use mkit::select::{OptionItem as SelectOption, Select};
use mkit::separator::Separator;
use mkit::sheet::Sheet;
use mkit::sidebar::{Item as SidebarItem, Sidebar};
use mkit::slider::Slider;
use mkit::split_pane::SplitPane;
use mkit::switch::Switch;
use mkit::tabs::{Item as TabItem, Tabs};
use mkit::text_area::TextArea;
use mkit::text_field::TextField;
use mkit::toast::Toast;
use mkit::toggle_button::ToggleButton;
use mkit::toggle_group::{Item as ToggleItem, ToggleGroup};
use mkit::tooltip::Tooltip;
use mkit::tree::{Tree, TreeNode};
use mkit::virtual_list::{ListItem, VirtualList};
use std::collections::HashSet;

pub struct EverydayGallery {
    inputs_only: bool,
    scene: GalleryScene,
    checkbox: Option<Entity<Checkbox>>,
    switch: Option<Entity<Switch>>,
    slider: Option<Entity<Slider>>,
    text_field_empty: Option<Entity<TextField>>,
    text_field_filled: Option<Entity<TextField>>,
    text_field_invalid: Option<Entity<TextField>>,
    text_field_disabled: Option<Entity<TextField>>,
    text_area_empty: Option<Entity<TextArea>>,
    text_area_filled: Option<Entity<TextArea>>,
    text_area_invalid: Option<Entity<TextArea>>,
    text_area_disabled: Option<Entity<TextArea>>,
    select: Option<Entity<Select>>,
    select_disabled: Option<Entity<Select>>,
    select_open: Option<Entity<Select>>,
    multi_select: Option<Entity<MultiSelect>>,
    multi_select_disabled: Option<Entity<MultiSelect>>,
    multi_select_open: Option<Entity<MultiSelect>>,
    combobox: Option<Entity<Combobox>>,
    combobox_disabled: Option<Entity<Combobox>>,
    toggle_idle: Option<Entity<ToggleButton>>,
    toggle_pressed: Option<Entity<ToggleButton>>,
    toggle_disabled: Option<Entity<ToggleButton>>,
    toggle_group: Option<Entity<ToggleGroup>>,
    toggle_group_disabled: Option<Entity<ToggleGroup>>,
    checkbox_unchecked: Option<Entity<Checkbox>>,
    checkbox_mixed: Option<Entity<Checkbox>>,
    checkbox_disabled: Option<Entity<Checkbox>>,
    switch_off: Option<Entity<Switch>>,
    switch_disabled: Option<Entity<Switch>>,
    slider_range: Option<Entity<Slider>>,
    slider_disabled: Option<Entity<Slider>>,
    radio_selected: Option<Entity<RadioGroup>>,
    radio_disabled: Option<Entity<RadioGroup>>,
    dialog: Option<Entity<Dialog>>,
    popover: Option<Entity<Popover>>,
    sheet: Option<Entity<Sheet>>,
}

fn render_breadth_scene(
    scene: GalleryScene,
    t: Theme,
    owner: &mut EverydayGallery,
    cx: &mut Context<EverydayGallery>,
) -> gpui_pre::AnyElement {
    let card = |title: &'static str| {
        div()
            .w_full()
            .p(px(t.spacing.large))
            .flex()
            .flex_col()
            .gap(px(t.spacing.medium))
            .rounded(px(t.radii.medium))
            .border(px(t.borders.hairline))
            .border_color(t.colors.border)
            .bg(t.colors.surface)
            .child(div().text_size(px(t.typography.heading_small)).child(title))
    };
    let heading = match scene {
        GalleryScene::Menus => "Menus",
        GalleryScene::Overlays => "Overlays",
        GalleryScene::Navigation => "Navigation",
        GalleryScene::Collections => "Collections",
        _ => "Layout helpers",
    };
    let body = match scene {
        GalleryScene::Menus => {
            // The screenshot harness has no keyboard script yet, so submenu
            // items are shown with their affordance but not expanded.
            let items = vec![
                CommandItem::new("new", "New file").shortcut("⌘ N"),
                CommandItem::new("duplicate", "Duplicate").shortcut("⌘ D"),
                CommandItem::new("view", "View options").submenu(vec![
                    CommandItem::new("grid", "Grid").checked(true),
                    CommandItem::new("list", "List").checked(false),
                ]),
                CommandItem::new("archive", "Archive").disabled(true),
            ];
            // The host supplies a window point for each deferred menu. Fixed
            // scene bounds reserve visible space for the screenshot fixtures.
            let menu = cx.new(|_| {
                DropdownMenu::controlled(items, true)
                    .anchor_at(gpui_pre::point(px(42.0), px(186.0)))
            });
            let context_items = vec![
                ContextItem::new("rename", "Rename").shortcut("F2"),
                ContextItem::new("pin", "Pin to top").checked(true),
                ContextItem::new("remove", "Remove").disabled(true),
            ];
            let context = cx.new(|_| {
                ContextMenu::controlled(context_items, true)
                    .anchor_at(gpui_pre::point(px(42.0), px(412.0)))
            });
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card("Dropdown menu · open").h(px(210.0)).child(menu))
                .child(card("Context menu · open").h(px(178.0)).child(context))
        }
        GalleryScene::Overlays => {
            // The harness cannot synthesize hover, so the tooltip's trigger is
            // visible while the transient hover label is not part of baseline.
            let overlay_muted = t.colors.text_muted;
            if owner.dialog.is_none() {
                let dialog = cx.new(|_| {
                    Dialog::with_content("Publish changes", move || {
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .child("Your changes will be visible to everyone in the workspace.")
                            .child(
                                div()
                                    .text_color(overlay_muted)
                                    .child("You can change access later in project settings."),
                            )
                    })
                });
                dialog.update(cx, |dialog, cx| dialog.set_open(false, cx));
                owner.dialog = Some(dialog);
            }
            let dialog = owner.dialog.as_ref().expect("dialog initialized").clone();
            if owner.popover.is_none() {
                owner.popover = Some(cx.new(|_| {
                    Popover::controlled("Quick settings", "Choose how this project appears.", true)
                        .trigger("Settings")
                        .anchor_at(gpui_pre::point(px(600.0), px(322.0)))
                }));
            }
            let popover = owner.popover.as_ref().expect("popover initialized").clone();
            if owner.sheet.is_none() {
                let sheet = cx.new(|_| {
                    Sheet::with_content("Edit profile", move || {
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .child("Update your name, role, and profile details.")
                            .child(
                                div()
                                    .text_color(overlay_muted)
                                    .child("Changes are saved automatically."),
                            )
                    })
                });
                sheet.update(cx, |sheet, cx| sheet.set_open(false, cx));
                owner.sheet = Some(sheet);
            }
            let sheet = owner.sheet.as_ref().expect("sheet initialized").clone();
            let toast = cx
                .new(|_| Toast::new("Changes saved", "Your workspace settings have been updated."));
            let dialog_trigger = dialog.clone();
            let sheet_trigger = sheet.clone();
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(
                    card("Dialog").child(
                        div()
                            .id("e7-dialog-trigger")
                            .debug_selector(|| "e7-dialog-trigger".into())
                            .child(
                                Button::new("Open dialog")
                                    .id(7_001)
                                    .variant(ButtonVariant::Outline)
                                    .on_activate(move |_, cx| {
                                        dialog_trigger
                                            .update(cx, |dialog, cx| dialog.set_open(true, cx));
                                    }),
                            ),
                    ),
                )
                .child(card("Popover · open").child(popover))
                .child(
                    card("Sheet").child(
                        div()
                            .id("e7-sheet-trigger")
                            .debug_selector(|| "e7-sheet-trigger".into())
                            .child(
                                Button::new("Open sheet")
                                    .id(7_002)
                                    .variant(ButtonVariant::Outline)
                                    .on_activate(move |_, cx| {
                                        sheet_trigger
                                            .update(cx, |sheet, cx| sheet.set_open(true, cx));
                                    }),
                            ),
                    ),
                )
                .child(card("Toast · open").child(toast))
                .child(
                    card("Tooltip").child(Tooltip::new(
                        "More information",
                        div()
                            .px(px(12.0))
                            .py(px(8.0))
                            .rounded(px(t.radii.small))
                            .border(px(t.borders.hairline))
                            .border_color(t.colors.border)
                            .bg(t.colors.elevated_surface)
                            .child("Hover for details"),
                    )),
                )
                .child(dialog)
                .child(sheet)
        }
        GalleryScene::Navigation => {
            let tabs = cx.new(|_| {
                Tabs::new(
                    "Project sections",
                    vec![
                        TabItem::new("overview", "Overview"),
                        TabItem::new("activity", "Activity"),
                        TabItem::new("settings", "Settings"),
                    ],
                    Some("overview".into()),
                )
            });
            let sidebar = cx.new(|_| {
                Sidebar::new(
                    "Workspace",
                    vec![
                        SidebarItem::new("home", "Home"),
                        SidebarItem::new("projects", "Projects"),
                        SidebarItem::new("settings", "Settings"),
                    ],
                    Some("projects".into()),
                )
            });
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card("Breadcrumbs").child(Breadcrumbs::new(
                    "Project location",
                    vec![
                        Crumb::new("Workspace"),
                        Crumb::new("Projects"),
                        Crumb::new("Website refresh"),
                    ],
                )))
                .child(card("Tabs").child(tabs))
                .child(card("Sidebar").child(sidebar))
                .child(card("Segmented control").child(cx.new(|_| {
                    SegmentedControl::new(
                        "View",
                        vec![
                            SegmentItem::new("board", "Board"),
                            SegmentItem::new("list", "List"),
                            SegmentItem::new("calendar", "Calendar"),
                        ],
                        Some("board".into()),
                    )
                })))
        }
        GalleryScene::Collections => {
            let list = cx.new(|_| {
                VirtualList::new(
                    "Recent projects",
                    vec![
                        ListItem::new("web", "Website refresh"),
                        ListItem::new("mobile", "Mobile app"),
                        ListItem::new("brand", "Brand system"),
                    ],
                )
                .viewport_height(160.0)
            });
            let tree = cx.new(|_| {
                Tree::new(
                    "Project files",
                    vec![
                        TreeNode::branch(
                            "src",
                            "src",
                            vec![
                                TreeNode::leaf("app", "App.rs"),
                                TreeNode::leaf("theme", "Theme.rs"),
                            ],
                        ),
                        TreeNode::leaf("readme", "README.md"),
                    ],
                )
            });
            tree.update(cx, |tree, cx| tree.set_expanded(HashSet::from(["src".to_owned()]), cx));
            let table = cx.new(|_| {
                DataTable::new(
                    "Project activity",
                    vec![
                        TableColumn::new("name", "Name").width(260),
                        TableColumn::new("status", "Status").width(150),
                        TableColumn::new("updated", "Updated").width(190),
                    ],
                    vec![
                        DataRow::new(
                            "website",
                            vec![
                                "Website refresh".into(),
                                "In progress".into(),
                                "2 hours ago".into(),
                            ],
                        ),
                        DataRow::new(
                            "mobile",
                            vec!["Mobile app".into(), "Review".into(), "Yesterday".into()],
                        ),
                        DataRow::new(
                            "brand",
                            vec!["Brand system".into(), "Complete".into(), "Sep 18".into()],
                        ),
                    ],
                )
                .multi_select()
            });
            table.update(cx, |table, cx| {
                table.set_selection(HashSet::from(["website".to_owned()]), cx);
                table.set_sort(Some(("updated".into(), SortDirection::Descending)), cx);
            });
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card("Virtual list").child(list))
                .child(card("Tree").child(tree))
                .child(card("Data table · selected row, sorted column").child(table))
        }
        _ => {
            let name = cx.new(|_| TextField::new_for_demo().controlled("Alex Morgan"));
            let bio = cx.new(|_| TextField::new_for_demo().controlled("Product designer"));
            let form = FormLayout::new()
                .label("Profile")
                .label_width(120.0)
                .row(FormRow::new("Name", name).description("Shown to your teammates."))
                .row(FormRow::new("Bio", bio));
            let scroll = ScrollArea::new(
                150.0,
                div().flex().flex_col().gap(px(t.spacing.small)).children(
                    (1..=7).map(|n| div().py(px(8.0)).child(format!("Scrollable row {n}"))),
                ),
            )
            .width(320.0)
            .label("Scrollable settings");
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card("Form layout").child(form))
                .child(card("Separator").child(Separator::new().label("Advanced settings")))
                .child(card("Split pane").child(cx.new(|cx| {
                    let left = cx.new(|_| PaneContents("Editor".into()));
                    let right = cx.new(|_| PaneContents("Inspector".into()));
                    SplitPane::new(left, right, 0.62)
                })))
                .child(card("Scroll area").child(scroll))
        }
    };
    div()
        .id("mkit-e7-breadth-scene")
        .size_full()
        .overflow_y_scroll()
        .bg(t.colors.background)
        .text_color(t.colors.text)
        .p(px(t.spacing.xlarge))
        .flex()
        .flex_col()
        .gap(px(t.spacing.large))
        .child(div().text_size(px(t.typography.heading_large)).child(heading))
        .child(
            div()
                .text_color(t.colors.text_muted)
                .child("E7.5–E7.9 components in the shadcn theme."),
        )
        .child(body)
        .into_any_element()
}

struct PaneContents(String);
impl Render for PaneContents {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        div()
            .p(px(t.spacing.medium))
            .rounded(px(t.radii.small))
            .bg(t.colors.surface)
            .child(self.0.clone())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GalleryScene {
    Main,
    Inputs,
    Menus,
    Overlays,
    Navigation,
    Collections,
    Layout,
}

impl EverydayGallery {
    pub fn new() -> Self {
        Self {
            inputs_only: false,
            scene: GalleryScene::Main,
            checkbox: None,
            switch: None,
            slider: None,
            text_field_empty: None,
            text_field_filled: None,
            text_field_invalid: None,
            text_field_disabled: None,
            text_area_empty: None,
            text_area_filled: None,
            text_area_invalid: None,
            text_area_disabled: None,
            select: None,
            select_disabled: None,
            select_open: None,
            multi_select: None,
            multi_select_disabled: None,
            multi_select_open: None,
            combobox: None,
            combobox_disabled: None,
            toggle_idle: None,
            toggle_pressed: None,
            toggle_disabled: None,
            toggle_group: None,
            toggle_group_disabled: None,
            checkbox_unchecked: None,
            checkbox_mixed: None,
            checkbox_disabled: None,
            switch_off: None,
            switch_disabled: None,
            slider_range: None,
            slider_disabled: None,
            radio_selected: None,
            radio_disabled: None,
            dialog: None,
            popover: None,
            sheet: None,
        }
    }

    /// A compact screenshot scene for the E7.3/E7.4 input and selection states.
    pub fn inputs_preview() -> Self {
        Self { inputs_only: true, scene: GalleryScene::Inputs, ..Self::new() }
    }

    /// Return retained selection fixtures for the gallery screenshot harness.
    pub fn selection_preview_entities(&self) -> (Entity<Select>, Entity<MultiSelect>) {
        (
            self.select_open.as_ref().expect("Select preview initialized").clone(),
            self.multi_select_open.as_ref().expect("MultiSelect preview initialized").clone(),
        )
    }

    pub fn menus_preview() -> Self {
        Self { scene: GalleryScene::Menus, ..Self::new() }
    }
    pub fn overlays_preview() -> Self {
        Self { scene: GalleryScene::Overlays, ..Self::new() }
    }
    pub fn navigation_preview() -> Self {
        Self { scene: GalleryScene::Navigation, ..Self::new() }
    }
    pub fn collections_preview() -> Self {
        Self { scene: GalleryScene::Collections, ..Self::new() }
    }
    pub fn layout_preview() -> Self {
        Self { scene: GalleryScene::Layout, ..Self::new() }
    }
}

impl Default for EverydayGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for EverydayGallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        if self.scene != GalleryScene::Main && self.scene != GalleryScene::Inputs {
            return render_breadth_scene(self.scene, t, self, cx);
        }
        let checkbox = self
            .checkbox
            .get_or_insert_with(|| cx.new(|_| Checkbox::new("Email updates", true)))
            .clone();
        let switch = self
            .switch
            .get_or_insert_with(|| cx.new(|_| Switch::new("Notifications", true)))
            .clone();
        let slider = self
            .slider
            .get_or_insert_with(|| cx.new(|_| Slider::new("Volume", 42.0, 0.0, 100.0, 1.0)))
            .clone();
        let text_field_empty = self
            .text_field_empty
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextField::new_for_demo()
                        .with_label("Project name")
                        .with_placeholder("Enter a name")
                })
            })
            .clone();
        let text_field_filled = self
            .text_field_filled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextField::new_for_demo().with_label("Workspace").controlled("Design system")
                })
            })
            .clone();
        let text_field_invalid = self
            .text_field_invalid
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextField::new_for_demo()
                        .with_label("Email")
                        .with_placeholder("you@example.com")
                        .with_validation_message("Enter a valid email address")
                })
            })
            .clone();
        let text_field_disabled = self
            .text_field_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextField::new_for_demo()
                        .with_label("Invite code")
                        .controlled("LAUNCH-2026")
                        .disabled(true)
                })
            })
            .clone();
        let text_area_empty = self
            .text_area_empty
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextArea::new_for_demo()
                        .with_label("Description")
                        .with_placeholder("What are you making?")
                })
            })
            .clone();
        let text_area_filled = self
            .text_area_filled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextArea::new_for_demo()
                        .with_label("About")
                        .controlled("A small toolkit for thoughtful interfaces.")
                })
            })
            .clone();
        let text_area_invalid = self
            .text_area_invalid
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextArea::new_for_demo()
                        .with_label("Summary")
                        .with_placeholder("Add a short summary")
                        .with_validation_message("A summary is required")
                })
            })
            .clone();
        let text_area_disabled = self
            .text_area_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    TextArea::new_for_demo()
                        .with_label("Release notes")
                        .controlled("Managed by your administrator.")
                        .disabled(true)
                })
            })
            .clone();
        let selection_options = || {
            vec![
                SelectOption::new("design", "Design"),
                SelectOption::new("engineering", "Engineering"),
                SelectOption::new("product", "Product"),
            ]
        };
        let select = self
            .select
            .get_or_insert_with(|| {
                cx.new(|_| Select::new("Team", selection_options(), Some("design".into())))
            })
            .clone();
        let select_disabled = self
            .select_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    Select::new("Team (disabled)", selection_options(), Some("engineering".into()))
                        .disabled(true)
                })
            })
            .clone();
        let select_open = self
            .select_open
            .get_or_insert_with(|| {
                cx.new(|_| Select::new("Team (open)", selection_options(), Some("design".into())))
            })
            .clone();
        let multi_options = || {
            vec![
                MultiSelectOption::new("design", "Design"),
                MultiSelectOption::new("engineering", "Engineering"),
                MultiSelectOption::new("product", "Product"),
            ]
        };
        let multi_select = self
            .multi_select
            .get_or_insert_with(|| {
                cx.new(|_| {
                    MultiSelect::new(
                        "Teams",
                        multi_options(),
                        vec!["design".into(), "product".into()],
                    )
                })
            })
            .clone();
        let multi_select_disabled = self
            .multi_select_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    MultiSelect::new(
                        "Teams (disabled)",
                        multi_options(),
                        vec!["engineering".into()],
                    )
                    .disabled(true)
                })
            })
            .clone();
        let multi_select_open = self
            .multi_select_open
            .get_or_insert_with(|| {
                cx.new(|_| {
                    MultiSelect::new(
                        "Teams (open)",
                        multi_options(),
                        vec!["design".into(), "product".into()],
                    )
                })
            })
            .clone();
        let combo_options = || {
            vec![
                ComboboxOption::new("san-francisco", "San Francisco"),
                ComboboxOption::new("san-jose", "San Jose"),
                ComboboxOption::new("seattle", "Seattle"),
            ]
        };
        let combobox = self
            .combobox
            .get_or_insert_with(|| {
                let combo = cx.new(|_| Combobox::new("City", combo_options(), None));
                combo.update(cx, |combo, cx| combo.set_query("San", cx));
                combo
            })
            .clone();
        let combobox_disabled = self
            .combobox_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    Combobox::new("City (disabled)", combo_options(), Some("san-francisco".into()))
                        .disabled(true)
                })
            })
            .clone();
        let checkbox_unchecked = self
            .checkbox_unchecked
            .get_or_insert_with(|| cx.new(|_| Checkbox::new("Unchecked", false)))
            .clone();
        let checkbox_mixed = self
            .checkbox_mixed
            .get_or_insert_with(|| cx.new(|_| Checkbox::new("Mixed", false).indeterminate(true)))
            .clone();
        let checkbox_disabled = self
            .checkbox_disabled
            .get_or_insert_with(|| cx.new(|_| Checkbox::new("Disabled", true).disabled(true)))
            .clone();
        let switch_off =
            self.switch_off.get_or_insert_with(|| cx.new(|_| Switch::new("Off", false))).clone();
        let switch_disabled = self
            .switch_disabled
            .get_or_insert_with(|| cx.new(|_| Switch::new("Disabled", true).disabled(true)))
            .clone();
        let slider_range = self
            .slider_range
            .get_or_insert_with(|| cx.new(|_| Slider::range("Range", 25.0, 72.0, 0.0, 100.0, 1.0)))
            .clone();
        let slider_disabled = self
            .slider_disabled
            .get_or_insert_with(|| {
                cx.new(|_| Slider::new("Disabled", 35.0, 0.0, 100.0, 1.0).disabled(true))
            })
            .clone();
        let radio_options = || {
            vec![
                OptionItem::new("small", "Small"),
                OptionItem::new("medium", "Medium"),
                OptionItem::new("large", "Large"),
            ]
        };
        let radio_selected = self
            .radio_selected
            .get_or_insert_with(|| {
                cx.new(|_| RadioGroup::new("Plan", radio_options(), Some("medium".into())))
            })
            .clone();
        let radio_disabled = self
            .radio_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    RadioGroup::new("Plan (disabled)", radio_options(), Some("small".into()))
                        .disabled(true)
                })
            })
            .clone();
        let toggle_idle = self
            .toggle_idle
            .get_or_insert_with(|| cx.new(|_| ToggleButton::new("Italic", false)))
            .clone();
        let toggle_pressed = self
            .toggle_pressed
            .get_or_insert_with(|| cx.new(|_| ToggleButton::new("Bold", true)))
            .clone();
        let toggle_disabled = self
            .toggle_disabled
            .get_or_insert_with(|| cx.new(|_| ToggleButton::new("Underline", true).disabled(true)))
            .clone();
        let group_items = || {
            vec![
                ToggleItem::new("left", "Left"),
                ToggleItem::new("center", "Center"),
                ToggleItem::new("right", "Right"),
            ]
        };
        let toggle_group = self
            .toggle_group
            .get_or_insert_with(|| {
                cx.new(|_| ToggleGroup::new("Text alignment", group_items(), Some("center".into())))
            })
            .clone();
        let toggle_group_disabled = self
            .toggle_group_disabled
            .get_or_insert_with(|| {
                cx.new(|_| {
                    ToggleGroup::new("Disabled text alignment", group_items(), Some("left".into()))
                        .disabled(true)
                })
            })
            .clone();
        let card = |title: &'static str| {
            div()
                .p(px(t.spacing.large))
                .flex()
                .flex_col()
                .gap(px(t.spacing.medium))
                .rounded(px(t.radii.medium))
                .border(px(t.borders.hairline))
                .border_color(t.colors.border)
                .bg(t.colors.surface)
                .child(div().text_size(px(t.typography.heading_small)).child(title))
        };
        if self.inputs_only {
            return div()
                .id("mkit-e7-inputs-preview")
                .size_full()
                .overflow_y_scroll()
                .bg(t.colors.background)
                .text_color(t.colors.text)
                .p(px(t.spacing.xlarge))
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(
                    div().text_size(px(t.typography.heading_large)).child("Text entry & selection"),
                )
                .child(
                    div()
                        .text_color(t.colors.text_muted)
                        .child("Everyday input states in the shadcn palette."),
                )
                .child(
                    card("Text fields")
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-text-field-empty")
                                        .debug_selector(|| "e7-text-field-empty".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(div().text_color(t.colors.text_muted).child("Empty"))
                                        .child(text_field_empty),
                                )
                                .child(
                                    div()
                                        .id("e7-text-field-filled")
                                        .debug_selector(|| "e7-text-field-filled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div().text_color(t.colors.text_muted).child("Filled"),
                                        )
                                        .child(text_field_filled),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-text-field-invalid")
                                        .debug_selector(|| "e7-text-field-invalid".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div().text_color(t.colors.text_muted).child("Invalid"),
                                        )
                                        .child(text_field_invalid),
                                )
                                .child(
                                    div()
                                        .id("e7-text-field-disabled")
                                        .debug_selector(|| "e7-text-field-disabled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div().text_color(t.colors.text_muted).child("Disabled"),
                                        )
                                        .child(text_field_disabled),
                                ),
                        ),
                )
                .child(
                    card("Text areas")
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-text-area-empty")
                                        .debug_selector(|| "e7-text-area-empty".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(div().text_color(t.colors.text_muted).child("Empty"))
                                        .child(text_area_empty),
                                )
                                .child(
                                    div()
                                        .id("e7-text-area-filled")
                                        .debug_selector(|| "e7-text-area-filled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div().text_color(t.colors.text_muted).child("Filled"),
                                        )
                                        .child(text_area_filled),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-text-area-invalid")
                                        .debug_selector(|| "e7-text-area-invalid".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div().text_color(t.colors.text_muted).child("Invalid"),
                                        )
                                        .child(text_area_invalid),
                                )
                                .child(
                                    div()
                                        .id("e7-text-area-disabled")
                                        .debug_selector(|| "e7-text-area-disabled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div().text_color(t.colors.text_muted).child("Disabled"),
                                        )
                                        .child(text_area_disabled),
                                ),
                        ),
                )
                .child(
                    card("Selection controls")
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-select-closed")
                                        .debug_selector(|| "e7-select-closed".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("Select · closed"),
                                        )
                                        .child(select),
                                )
                                .child(
                                    div()
                                        .id("e7-select-disabled")
                                        .debug_selector(|| "e7-select-disabled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("Select · disabled"),
                                        )
                                        .child(select_disabled),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-multi-select-closed")
                                        .debug_selector(|| "e7-multi-select-closed".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("MultiSelect · closed"),
                                        )
                                        .child(multi_select),
                                )
                                .child(
                                    div()
                                        .id("e7-multi-select-disabled")
                                        .debug_selector(|| "e7-multi-select-disabled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("MultiSelect · disabled"),
                                        )
                                        .child(multi_select_disabled),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-combobox-open")
                                        .debug_selector(|| "e7-combobox-open".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("Combobox · open, filtered"),
                                        )
                                        .child(combobox),
                                )
                                .child(
                                    div()
                                        .id("e7-combobox-disabled")
                                        .debug_selector(|| "e7-combobox-disabled".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("Combobox · disabled"),
                                        )
                                        .child(combobox_disabled),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(t.spacing.large))
                                .child(
                                    div()
                                        .id("e7-select-open")
                                        .debug_selector(|| "e7-select-open".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("Select · open"),
                                        )
                                        .child(select_open),
                                )
                                .child(
                                    div()
                                        .id("e7-multi-select-open")
                                        .debug_selector(|| "e7-multi-select-open".into())
                                        .w(px(390.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(t.spacing.small))
                                        .child(
                                            div()
                                                .text_color(t.colors.text_muted)
                                                .child("MultiSelect · open"),
                                        )
                                        .child(multi_select_open),
                                ),
                        ),
                )
                .into_any_element();
        }
        div()
            .id("mkit-e7-gallery")
            .size_full()
            .overflow_y_scroll()
            .bg(t.colors.background)
            .text_color(t.colors.text)
            .p(px(t.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(t.spacing.large))
            .child(div().text_size(px(t.typography.heading_large)).child("Everyday components"))
            .child(
                div()
                    .text_color(t.colors.text_muted)
                    .child("Neutral, compact GPUI controls with shared semantic tokens."),
            )
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.small))
                    .child(
                        Button::new("Light")
                            .variant(if t.name == SHADCN_LIGHT.name {
                                ButtonVariant::Default
                            } else {
                                ButtonVariant::Outline
                            })
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.set_global(SHADCN_LIGHT);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("Dark")
                            .variant(if t.name == SHADCN_DARK.name {
                                ButtonVariant::Default
                            } else {
                                ButtonVariant::Outline
                            })
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.set_global(SHADCN_DARK);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                card("Buttons").child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.small))
                        .child(Button::new("Save changes"))
                        .child(Button::new("Cancel").variant(ButtonVariant::Secondary))
                        .child(Button::new("Outline").variant(ButtonVariant::Outline))
                        .child(Button::new("Small").size(ButtonSize::Small))
                        .child(Button::new("Loading").loading(true)),
                ),
            )
            .child(
                card("Icon buttons").child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.large))
                        .child(
                            div()
                                .id("e7-icon-button-idle")
                                .debug_selector(|| "e7-icon-button-idle".into())
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.small))
                                .child(IconButton::new("Close", div().child("×")))
                                .child(div().text_color(t.colors.text_muted).child("Idle")),
                        )
                        .child(
                            div()
                                .id("e7-icon-button-disabled")
                                .debug_selector(|| "e7-icon-button-disabled".into())
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.small))
                                .child(
                                    IconButton::new("More options", div().child("···"))
                                        .variant(IconButtonVariant::Outline)
                                        .disabled(true),
                                )
                                .child(div().text_color(t.colors.text_muted).child("Disabled")),
                        ),
                ),
            )
            .child(
                card("Toggle buttons").child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.large))
                        .child(
                            div()
                                .id("e7-toggle-button-idle")
                                .debug_selector(|| "e7-toggle-button-idle".into())
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.small))
                                .child(toggle_idle)
                                .child(div().text_color(t.colors.text_muted).child("Idle")),
                        )
                        .child(
                            div()
                                .id("e7-toggle-button-pressed")
                                .debug_selector(|| "e7-toggle-button-pressed".into())
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.small))
                                .child(toggle_pressed)
                                .child(div().text_color(t.colors.text_muted).child("Pressed")),
                        )
                        .child(
                            div()
                                .id("e7-toggle-button-disabled")
                                .debug_selector(|| "e7-toggle-button-disabled".into())
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.small))
                                .child(toggle_disabled)
                                .child(div().text_color(t.colors.text_muted).child("Disabled")),
                        ),
                ),
            )
            .child(
                card("Toggle groups")
                    .child(
                        div()
                            .id("e7-toggle-group-selected")
                            .debug_selector(|| "e7-toggle-group-selected".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(toggle_group)
                            .child(div().text_color(t.colors.text_muted).child("Center selected")),
                    )
                    .child(
                        div()
                            .id("e7-toggle-group-disabled")
                            .debug_selector(|| "e7-toggle-group-disabled".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(toggle_group_disabled)
                            .child(div().text_color(t.colors.text_muted).child("Disabled")),
                    ),
            )
            .child(
                card("Checkboxes").child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.large))
                        .child(
                            div()
                                .id("e7-checkbox-unchecked")
                                .debug_selector(|| "e7-checkbox-unchecked".into())
                                .child(checkbox_unchecked),
                        )
                        .child(
                            div()
                                .id("e7-checkbox-checked")
                                .debug_selector(|| "e7-checkbox-checked".into())
                                .child(checkbox),
                        )
                        .child(
                            div()
                                .id("e7-checkbox-mixed")
                                .debug_selector(|| "e7-checkbox-mixed".into())
                                .child(checkbox_mixed),
                        )
                        .child(
                            div()
                                .id("e7-checkbox-disabled")
                                .debug_selector(|| "e7-checkbox-disabled".into())
                                .child(checkbox_disabled),
                        ),
                ),
            )
            .child(
                card("Radio groups")
                    .child(
                        div()
                            .id("e7-radio-selected")
                            .debug_selector(|| "e7-radio-selected".into())
                            .flex()
                            .items_start()
                            .gap(px(t.spacing.xlarge))
                            .child(radio_selected),
                    )
                    .child(
                        div()
                            .id("e7-radio-disabled")
                            .debug_selector(|| "e7-radio-disabled".into())
                            .flex()
                            .items_start()
                            .gap(px(t.spacing.xlarge))
                            .child(radio_disabled),
                    ),
            )
            .child(
                card("Switches").child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.large))
                        .child(
                            div()
                                .id("e7-switch-off")
                                .debug_selector(|| "e7-switch-off".into())
                                .child(switch_off),
                        )
                        .child(
                            div()
                                .id("e7-switch-on")
                                .debug_selector(|| "e7-switch-on".into())
                                .child(switch),
                        )
                        .child(
                            div()
                                .id("e7-switch-disabled")
                                .debug_selector(|| "e7-switch-disabled".into())
                                .child(switch_disabled),
                        ),
                ),
            )
            .child(
                card("Sliders")
                    .child(
                        div()
                            .id("e7-slider-single")
                            .debug_selector(|| "e7-slider-single".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(120.0)).child("Single value"))
                            .child(slider),
                    )
                    .child(
                        div()
                            .id("e7-slider-range")
                            .debug_selector(|| "e7-slider-range".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(120.0)).child("Range"))
                            .child(slider_range),
                    )
                    .child(
                        div()
                            .id("e7-slider-disabled")
                            .debug_selector(|| "e7-slider-disabled".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(120.0)).child("Disabled"))
                            .child(slider_disabled),
                    ),
            )
            .child(
                card("Progress")
                    .child(
                        div()
                            .id("e7-progress-zero")
                            .debug_selector(|| "e7-progress-zero".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(140.0)).child("Zero"))
                            .child(Progress::new("Zero progress", 0.0)),
                    )
                    .child(
                        div()
                            .id("e7-progress-partial")
                            .debug_selector(|| "e7-progress-partial".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(140.0)).child("Partial"))
                            .child(Progress::new("Partial progress", 64.0)),
                    )
                    .child(
                        div()
                            .id("e7-progress-complete")
                            .debug_selector(|| "e7-progress-complete".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(140.0)).child("Complete"))
                            .child(Progress::new("Complete progress", 100.0)),
                    )
                    .child(
                        div()
                            .id("e7-progress-indeterminate")
                            .debug_selector(|| "e7-progress-indeterminate".into())
                            .flex()
                            .items_center()
                            .gap(px(t.spacing.medium))
                            .child(div().w(px(140.0)).child("Indeterminate"))
                            .child(Progress::new("Indeterminate progress", 0.0).indeterminate()),
                    ),
            )
            .child(
                card("Text fields")
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-text-field-empty")
                                    .debug_selector(|| "e7-text-field-empty".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Empty"))
                                    .child(text_field_empty),
                            )
                            .child(
                                div()
                                    .id("e7-text-field-filled")
                                    .debug_selector(|| "e7-text-field-filled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Filled"))
                                    .child(text_field_filled),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-text-field-invalid")
                                    .debug_selector(|| "e7-text-field-invalid".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Invalid"))
                                    .child(text_field_invalid),
                            )
                            .child(
                                div()
                                    .id("e7-text-field-disabled")
                                    .debug_selector(|| "e7-text-field-disabled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Disabled"))
                                    .child(text_field_disabled),
                            ),
                    ),
            )
            .child(
                card("Text areas")
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-text-area-empty")
                                    .debug_selector(|| "e7-text-area-empty".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Empty"))
                                    .child(text_area_empty),
                            )
                            .child(
                                div()
                                    .id("e7-text-area-filled")
                                    .debug_selector(|| "e7-text-area-filled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Filled"))
                                    .child(text_area_filled),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-text-area-invalid")
                                    .debug_selector(|| "e7-text-area-invalid".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Invalid"))
                                    .child(text_area_invalid),
                            )
                            .child(
                                div()
                                    .id("e7-text-area-disabled")
                                    .debug_selector(|| "e7-text-area-disabled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(div().text_color(t.colors.text_muted).child("Disabled"))
                                    .child(text_area_disabled),
                            ),
                    ),
            )
            .child(
                card("Selection controls")
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-select-closed")
                                    .debug_selector(|| "e7-select-closed".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(
                                        div()
                                            .text_color(t.colors.text_muted)
                                            .child("Select · closed"),
                                    )
                                    .child(select),
                            )
                            .child(
                                div()
                                    .id("e7-select-disabled")
                                    .debug_selector(|| "e7-select-disabled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(
                                        div()
                                            .text_color(t.colors.text_muted)
                                            .child("Select · disabled"),
                                    )
                                    .child(select_disabled),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-multi-select-closed")
                                    .debug_selector(|| "e7-multi-select-closed".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(
                                        div()
                                            .text_color(t.colors.text_muted)
                                            .child("MultiSelect · closed"),
                                    )
                                    .child(multi_select),
                            )
                            .child(
                                div()
                                    .id("e7-multi-select-disabled")
                                    .debug_selector(|| "e7-multi-select-disabled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(
                                        div()
                                            .text_color(t.colors.text_muted)
                                            .child("MultiSelect · disabled"),
                                    )
                                    .child(multi_select_disabled),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.large))
                            .child(
                                div()
                                    .id("e7-combobox-open")
                                    .debug_selector(|| "e7-combobox-open".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(
                                        div()
                                            .text_color(t.colors.text_muted)
                                            .child("Combobox · open, filtered"),
                                    )
                                    .child(combobox),
                            )
                            .child(
                                div()
                                    .id("e7-combobox-disabled")
                                    .debug_selector(|| "e7-combobox-disabled".into())
                                    .w(px(390.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(t.spacing.small))
                                    .child(
                                        div()
                                            .text_color(t.colors.text_muted)
                                            .child("Combobox · disabled"),
                                    )
                                    .child(combobox_disabled),
                            ),
                    ),
            )
            .into_any_element()
    }
}
