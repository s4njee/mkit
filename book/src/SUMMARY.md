# Summary

- [Introduction](README.md)

# Part I: Getting started

- [Install](install.md)
- [Hello window](examples/hello.md)
- [How GPUI thinks](getting-started/how-gpui-thinks.md)
- [Counter, line by line](examples/counter.md)

# Part II: State and views

- [Entities and weak handles](state/entities.md)
- [Which GPUI context do you have?](state/contexts.md)
- [Reactivity: notifications and events](state/reactivity.md)
- [Views and components](state/views-and-components.md)
- [Controlled and uncontrolled component state](state/controlled-state.md)
- [Shared configuration with Global](state/globals.md)
- [Borrow-checker survival guide](state/borrow-checker.md)

# Part III: Elements, styling and layout

- [Build a layout with div](elements/div-and-layout.md)
- [Size and scroll a settings area](elements/size-and-scroll.md)
- [Style and fit text](elements/text.md)
- [Add an image and SVG icon](elements/images-and-svg.md)
- [Build conditional rows and lists](elements/conditional-lists.md)
- [Put the settings screen together](elements/settings-screen.md)

# Part IV: Interaction

- [Pointer input and hit testing](interaction/mouse.md)
- [Focus and tab order](interaction/focus.md)
- [Actions and key bindings](interaction/actions.md)
- [Drag and drop](interaction/drag-and-drop.md)
- [Clipboard text](interaction/clipboard.md)
- [Application and context menus](interaction/menus.md)
- [A keyboard-driven list](interaction/keyboard-list.md)

# Part V: Text input and IME

- [Connect a field to platform text input](text-input/handler-contract.md)
- [Marked text and IME candidates](text-input/ime-composition.md)
- [Selection, cursor movement, and undo](text-input/selection-and-undo.md)
- [Build a single-line text field](text-input/single-line-field.md)

# Part VI: Custom rendering

- [Write a custom Element](custom-rendering/element-lifecycle.md)
- [Paint paths, quads, shadows, and images](custom-rendering/canvas-primitives.md)
- [Place overlays above content](custom-rendering/overlays.md)
- [Render only visible list items](custom-rendering/virtualization.md)
- [Animate while respecting reduced motion](custom-rendering/animation.md)
- [What surface can show](custom-rendering/gpu-surfaces.md)
- [Build a pan-and-zoom canvas](custom-rendering/pan-and-zoom.md)

# Part VII: Async, data and persistence

- [Keep long work off the UI thread](async/executors-and-tasks.md)
- [Return async results to an entity](async/entity-updates.md)
- [Keep settings, files, and SQLite separate](async/settings-files-sqlite.md)
- [Build an async file browser](async/file-browser.md)

# Part VIII: Windows, platform and shipping

- [Windows and appearance](windows-platform-shipping/windows-and-appearance.md)
- [Accessibility across the window](windows-platform-shipping/accessibility.md)
- [Package an app for users](windows-platform-shipping/packaging.md)
- [Logs and crash diagnostics](windows-platform-shipping/diagnostics.md)

# Part IX: Testing GPUI apps

- [Choose a test context](testing/test-contexts.md)
- [Simulate input and assert state](testing/simulate-and-assert.md)
- [Capture a harness screenshot](testing/harness-screenshots.md)

# Part X: Components

- [Own component source from the registry](components/source-registry.md)
- [Everyday component drafts](components/everyday-components.md)
- [Everyday component guide](components/e7/README.md)
  - [Button](components/e7/button.md)
  - [IconButton](components/e7/icon-button.md)
  - [ToggleButton](components/e7/toggle-button.md)
  - [ToggleGroup](components/e7/toggle-group.md)
  - [Checkbox](components/e7/checkbox.md)
  - [RadioGroup](components/e7/radio-group.md)
  - [Switch](components/e7/switch.md)
  - [Slider](components/e7/slider.md)
  - [Progress](components/e7/progress.md)
  - [TextField](components/e7/text-field.md)
  - [TextArea](components/e7/text-area.md)
  - [Select](components/e7/select.md)
  - [Combobox](components/e7/combobox.md)
  - [MultiSelect](components/e7/multi-select.md)
  - [DropdownMenu](components/e7/dropdown-menu.md)
  - [ContextMenu](components/e7/context-menu.md)
  - [Tooltip](components/e7/tooltip.md)
  - [Popover](components/e7/popover.md)
  - [Dialog](components/e7/dialog.md)
  - [Sheet](components/e7/sheet.md)
  - [Toast](components/e7/toast.md)
  - [Tabs](components/e7/tabs.md)
  - [Sidebar](components/e7/sidebar.md)
  - [Breadcrumbs](components/e7/breadcrumbs.md)
  - [SegmentedControl](components/e7/segmented-control.md)
  - [VirtualList](components/e7/virtual-list.md)
  - [Tree](components/e7/tree.md)
  - [DataTable](components/e7/data-table.md)
  - [SplitPane](components/e7/split-pane.md)
  - [ScrollArea](components/e7/scroll-area.md)
  - [FormLayout](components/e7/form-layout.md)
  - [Separator](components/e7/separator.md)
  - [Calendar](components/e7/calendar.md)
  - [Disclosure and Accordion](components/e7/disclosure.md)
  - [Toolbar](components/e7/toolbar.md)
  - [Date picker and date range picker](components/e7/date-picker.md)
  - [Time field](components/e7/time-field.md)
  - [Date and time field](components/e7/date-time-field.md)
  - [Search field](components/e7/search-field.md)
  - [Badge](components/e7/badge.md)
  - [Avatar](components/e7/avatar.md)
  - [Key hint](components/e7/key-hint.md)
  - [Link](components/e7/link.md)
  - [Inline alert and empty state](components/e7/inline-alert.md)
  - [Tag input](components/e7/tag-input.md)
  - [File drop zone](components/e7/file-drop-zone.md)
  - [File field](components/e7/file-field.md)
  - [Reorderable list](components/e7/reorderable-list.md)
  - [In-window menu bar](components/e7/menu-bar.md)
  - [Status bar](components/e7/status-bar.md)
  - [Stepper](components/e7/stepper.md)
- [Pro-app component guide](components/e8/README.md)
  - [Viewport](components/e8/viewport.md)
  - [Precision slider](components/e8/precision-slider.md)
  - [Curve editor](components/e8/curve-editor.md)
  - [Colour tools](components/e8/colour-tools.md)
  - [Gradient editor](components/e8/gradient-editor.md)
  - [Histogram](components/e8/histogram.md)
  - [Property inspector](components/e8/property-inspector.md)
  - [Layer panel](components/e8/layer-panel.md)
  - [Command palette](components/e8/command-palette.md)
  - [Shortcut editor](components/e8/shortcut-editor.md)
  - [Timeline: tracks and ruler](components/e8/timeline-t1.md)
  - [Timeline: playhead and selection](components/e8/timeline-t2.md)
  - [Timeline: clip editing and keyframes](components/e8/timeline-t3-t4.md)
  - [Node editor: graph surface](components/e8/node-editor-n1.md)
  - [Node editor: movement and box selection](components/e8/node-editor-n2.md)
  - [Node editor: connection editing](components/e8/node-editor-n3.md)
  - [Node editor: minimap navigation](components/e8/node-editor-n4.md)
  - [Laika seeds](components/e8/laika-seeds.md)
- [Editable combobox pilot](components/combobox-pilot.md)
- [Scrubbable number field pilot](components/number-field-pilot.md)

# Part XI: Theming and design language

- [The mkit design language](theming/design-language.md)

# Part XII: Coming from another UI framework

- [Coming from React](coming-from/react.md)
- [Coming from Tauri](coming-from/tauri.md)
- [Coming from iced or egui](coming-from/iced-egui.md)

# Part XIII: Architecture internals

- [Trace a GPUI frame](architecture/frame-pipeline.md)
- [Trace text across platforms](architecture/text-systems.md)
- [Follow an entity's lifetime](architecture/entity-map.md)

# Part XIV: Cookbook

- [Increment an entity counter](cookbook/state/entity-counter.md)
- [Observe an entity change](cookbook/state/observe-entity.md)
- [Emit a typed counter event](cookbook/state/typed-event.md)
- [Install an app global](cookbook/state/install-global.md)
- [Read an app global](cookbook/state/read-global.md)
- [Update a row by stable ID](cookbook/state/stable-row-ids.md)
- [Compute a count from source state](cookbook/state/derived-count.md)
- [Run a short foreground task](cookbook/async/foreground-executor.md)
- [Run work on the background executor](cookbook/async/background-executor.md)
- [Wait with an executor timer](cookbook/async/timer.md)
- [Cancel a retained task handle](cookbook/async/cancel-task.md)
- [Reject an old generation](cookbook/async/debounce-generation.md)
- [Commit only the latest query](cookbook/async/debounce-timer.md)
- [Ignore an out-of-date result](cookbook/async/latest-result.md)
- [Queue and dismiss toast messages](cookbook/async/toast-queue.md)
- [Declare typed actions](cookbook/input/declare-actions.md)
- [Choose a navigation key at startup](cookbook/input/rebind-navigation.md)
- [Wrap focus inside a small modal view](cookbook/input/modal-focus.md)
- [Write text to the clipboard](cookbook/input/clipboard-write.md)
- [Reorder rows through GPUI drag and drop](cookbook/input/drag-reorder.md)
- [Receive selected file paths](cookbook/input/file-picker.md)
- [Register an application menu](cookbook/windows/application-menu.md)
- [Request a tagged system notification](cookbook/windows/system-notification.md)
- [Request centered window bounds](cookbook/windows/window-options.md)
- [Save the current window size](cookbook/windows/persist-window-size.md)
- [Toggle an in-app token palette](cookbook/windows/theme-preview.md)
- [Lay out a simple split pane](cookbook/rendering/split-pane.md)
- [Make a vertical scroll region](cookbook/rendering/scroll-region.md)
- [Style a span of text](cookbook/rendering/rich-text.md)
- [Arrange two grid cells](cookbook/rendering/grid-layout.md)
- [Render a panel conditionally](cookbook/rendering/conditional-panel.md)
- [Build a reusable div helper](cookbook/rendering/reusable-div.md)

- [Appendices](appendices/README.md)
  - [Versions](appendices/versions.md)
  - [Migration notes](appendices/migration.md)
  - [Glossary](appendices/glossary.md)
  - [Public API inventory](appendices/api-inventory.md)
  - [Concept index](appendices/concept-index.md)
