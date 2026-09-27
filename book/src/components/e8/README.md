# E8 pro-app component guide

E8 covers controls for creative, data, and developer tools. Work is underway. The [viewport](viewport.md), [precision slider](precision-slider.md), and [histogram](histogram.md) have source drafts and a shared rendered preview. The [curve editor](curve-editor.md), [colour tools](colour-tools.md), [gradient editor](gradient-editor.md), [property inspector](property-inspector.md), [layer panel](layer-panel.md), [command palette](command-palette.md), [shortcut editor](shortcut-editor.md), [timeline tracks and ruler](timeline-t1.md), [timeline playhead and selection](timeline-t2.md), [node editor graph surface](node-editor-n1.md), [node movement and box selection](node-editor-n2.md), [node connection editing](node-editor-n3.md), and [node minimap navigation](node-editor-n4.md) each have their own guide. The [scrubbable number field pilot](../number-field-pilot.md) now has a 36-case state/theme/scale screenshot matrix and a focused disabled-input regression check. These components still have `implementation_in_progress` status; their public APIs and full conformance are not approved.

![Viewport, histogram, and precision slider in the dark theme](../../images/e8-preview-dark-2x.png)

The preview comes from a compiling example and a macOS harness capture. Run `cargo run -p mkit-example-e8-components --locked` to try the pan-and-zoom surface. The [same preview in the light theme](../../images/e8-preview-light-2x.png) uses the GPUI Global theme tokens.

## What is available now

| E8 story | Current draft | Remaining work |
| --- | --- | --- |
| E8.1 Viewport | App painter callback, transform and coordinate methods, drag, trackpad, and keyboard pan, wheel and pinch zoom, Fit/100%, optional rulers and guides; 24 state/theme/scale captures. | Physical trackpad and accessibility verification, public review. |
| E8.2 Scrubbable number field | Typing, keyboard steps, pointer scrubbing, bounds, controlled requests, optional unit suffix, and a 36-case state/theme/scale matrix. | Native IME and screen-reader checks, public API and visual review. |
| E8.3 Precision slider | Drag and keyboard adjustment, Shift fine adjustment, double-click reset, bipolar fill, value tooltip; 30 state/theme/scale captures. | Accessibility matrix and public API/visual review. |
| E8.4 Curve editor | Named channels, linear and smooth interpolation, pointer add/move/delete, keyboard nudge, controlled-event tests, and 30 state/theme/scale captures. | Accessibility matrix and public API review. |
| E8.5 Colour tools | Hue/saturation picker, editable sRGB/HSL/OKLCH values, three grading wheels, and 18 state/theme/scale captures. | Eyedropper backend, accessibility matrix, gamut handling, public API/visual review. |
| E8.6 Gradient editor | Editable sRGB stops, sampled live preview, pointer and keyboard changes, and 30 state/theme/scale captures. | Accessibility matrix and public API/visual review. |
| E8.7 Histogram | Luminance or RGB bins, clipping indicators, supplied accessible summary, and 24 state/theme/scale captures. | Platform accessibility and maintainer visual review. |
| E8.8 Property inspector | Grouped typed editors, mixed values, reset requests, real edited state, and five states across three themes and two scales. | Native IME/text selection, platform AX snapshots, and public API/keyboard/visual review. |
| E8.9 Timeline [T1](timeline-t1.md), [T2](timeline-t2.md), and [T3–T4](timeline-t3-t4.md) | Readable tracks and ruler, playhead and selection, reversible pointer/keyboard clip and keyframe edits, snapping, stable keyframe navigation, and 42 state/theme/scale captures. | Native AX, dense/invalid interaction cases, and maintainer public API/visual review. |
| E8.10 Node editor [N1](node-editor-n1.md), [N2](node-editor-n2.md), [N3](node-editor-n3.md), and [N4](node-editor-n4.md) | App-supplied graph, selection and move proposals, typed connection editing requests, optional minimap navigation, and 78 state/theme/scale captures. | Native AX, trackpad/outside-canvas interaction checks, and public API/visual review. |
| E8.11 Layer panel | Nested groups, image or swatch thumbnails, visibility/lock, selected row, keyboard/button/drag sibling reorder, and five states across three themes and two scales. | Native accessibility snapshots and public API/visual review. |
| E8.12 Command palette | Search over host actions, active result navigation, Enter activation, Escape dismissal, displayed keybindings, focus containment, and 30 state/theme/scale captures. | Accessibility and host focus/dialog integration review. |
| E8.13 Shortcut editor | Action binding list, capture, conflict resolution, versioned JSON save API, typed host adapter, and 36 state/theme/scale captures. | Accessibility, native host keymap integration, and public API review. |
| E8.14 Laika seeding | An isolated Laika worktree now mounts mkit histogram, controlled slider entities, and a controlled segmented view selector on the aligned GPUI version. Its app build and 36 tests pass. | Interactive multi-photo, undo/reset, keyboard, visual, and accessibility checks; maintainer API and spec review. |

Other E8 stories are still future work. The source workspace's `plan.md` is the authoritative story list.
