# Everyday component guide

This is mkit's draft collection of everyday desktop controls. These pages help you choose a control, understand its interaction contract, and inspect available gallery screenshots. They describe the current workspace source; **the components are still drafts**. The registry marks their implementations as `implementation_in_progress`, so they are not released through `cargo mkit add`. Keyboard tests and many screenshot comparisons have passed, while full platform accessibility checks and maintainer approval remain open. The detailed progress audit is in `docs/E7_AUDIT.md` in the source workspace.

## Find a component

| Need | Components |
| --- | --- |
| Start an action or set a pressed choice | [Button](button.md), [IconButton](icon-button.md), [ToggleButton](toggle-button.md), [ToggleGroup](toggle-group.md) |
| Choose or show a value | [Checkbox](checkbox.md), [RadioGroup](radio-group.md), [Switch](switch.md), [Slider](slider.md), [Progress](progress.md) |
| Enter text or select from options | [TextField](text-field.md), [TextArea](text-area.md), [Select](select.md), [Combobox](combobox.md), [MultiSelect](multi-select.md) |
| Offer commands | [DropdownMenu](dropdown-menu.md), [ContextMenu](context-menu.md) |
| Explain, overlay, or notify | [Tooltip](tooltip.md), [Popover](popover.md), [Dialog](dialog.md), [Sheet](sheet.md), [Toast](toast.md) |
| Move through an app | [Tabs](tabs.md), [Sidebar](sidebar.md), [Breadcrumbs](breadcrumbs.md), [SegmentedControl](segmented-control.md) |
| Browse many items | [VirtualList](virtual-list.md), [Tree](tree.md), [DataTable](data-table.md) |
| Arrange content | [SplitPane](split-pane.md), [ScrollArea](scroll-area.md), [FormLayout](form-layout.md), [Separator](separator.md) |
| Show optional content | [Disclosure and Accordion](disclosure.md) |
| Pick a date | [Calendar](calendar.md) |
| Type or pick a date | [Date picker and date range picker](date-picker.md) |
| Enter a time or date and time | [TimeField](time-field.md), [DateTimeField](date-time-field.md) |
| Search and filter results | [SearchField](search-field.md) |
| Show window status and quick controls | [StatusBar](status-bar.md) |
| Show compact identity, state, shortcuts, and navigation | [Badge](badge.md), [Avatar](avatar.md), [KeyHint](key-hint.md), [Link](link.md) |
| Group frequent commands | [Toolbar](toolbar.md) |
| Explain local state | [Inline alert and empty state](inline-alert.md) |
| Enter labels or recipients | [Tag input](tag-input.md) |
| Select files by drop or browse | [File drop zone](file-drop-zone.md), [File field](file-field.md) |
| Guide an ordered workflow | [Stepper](stepper.md) |
| Reorder a short list | [Reorderable list](reorderable-list.md) |
| Show application menus inside a window | [In-window menu bar](menu-bar.md) |

The [everyday component overview](../everyday-components.md) explains how the gallery and conformance work fit together. Each page below uses the checked-in spec and implementation as its source. Examples are usage situations, not Rust code to copy. The public APIs, accessibility contracts, and visual baselines still need maintainer review.

The inputs gallery scene includes real open-popup previews for Select and MultiSelect alongside their closed and disabled controls. These captures use the components' keyboard actions to open the retained fixtures.
