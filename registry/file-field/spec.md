---
spec_version: 1
component: file-field
states:
  - id: empty
    description: No files are selected and the field offers Browse.
    fixture: empty_fixture
  - id: focused
    description: The Browse button has keyboard focus and shows the focus-visible ring.
    fixture: focused_fixture
  - id: selected
    description: Selected file names and sizes are shown with remove actions.
    fixture: selected_fixture
  - id: multiple
    description: Multiple selected files are shown when multiple selection is enabled.
    fixture: multiple_fixture
  - id: disabled
    description: Browse and remove actions are disabled.
    fixture: disabled_fixture
  - id: invalid
    description: Rejected files are reported while accepted files remain selected.
    fixture: invalid_fixture
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: Requires a real picker result or a dedicated public validation-state API; the screenshot matrix does not fabricate picker behavior.
keys:
  - key: Enter
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker.
    initial_state: empty
    expect: { event: browse_requested, focus_target: same_button }
  - key: Space
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker.
    initial_state: empty
    expect: { event: browse_requested, focus_target: same_button }
  - key: Enter
    modifiers: []
    when: A focused Remove file button is enabled.
    action: Remove that file from the selection proposal.
    initial_state: selected
    expect: { event: files_change, focus_target: browse_or_next_remove }
  - key: Space
    modifiers: []
    when: A focused Remove file button is enabled.
    action: Remove that file from the selection proposal.
    initial_state: selected
    expect: { event: files_change, focus_target: browse_or_next_remove }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided field label
    - name: description
      value: accepted file types and selection limit
    - name: disabled
      value: true
      when: disabled
    - name: live
      value: polite for validation and selection updates
---

# FileField

## Purpose

Represent a file selection as a labeled form field, with a platform Browse button and removable selected-file rows. It may be used independently or alongside `FileDropZone`.

## Anatomy

The field contains a label, accepted-type hint, Browse button, zero or more file rows, and a polite validation/status message. Each file row contains a name, optional size, and a Remove button.

## States

`empty` has no rows. `selected` contains one accepted path. `multiple` contains several accepted paths. `invalid` announces rejected paths while preserving accepted selection. `disabled` disables Browse and row removal.

## Props and events

The stateful `FileField` accepts label, `accept` suffix filters, `multiple`, `disabled`, and optional `files`. Omitting `.files(...)` selects uncontrolled mode and mutates internal selection; `.default_files(paths)` seeds its initial value. `.files(paths)` selects controlled mode: `FilesChange` emits a proposed complete selection and the component waits for the parent to call `set_files`. A second typed event, `BrowseRequested`, reports that the user asked to browse; the component invokes GPUI's platform picker and reports its result as `FilesChange`. `FileRejected` reports the rejected paths and reason. Canceling the picker changes no files. Each item carries a path, display name, and optional caller-provided size; file contents are never opened by the component.

## Keyboard map

Tab traverses Browse followed by enabled Remove buttons in row order. Enter and Space activate Browse; Enter and Space activate a focused Remove button. After removing a row, focus moves to the next row's Remove button, otherwise the previous row, otherwise Browse. Escape has no field-specific behavior.

## Pointer behaviour

Browse opens the platform path picker. Remove actions remove one path and emit a full selection proposal. In controlled mode the visual selection remains parent-owned until new `files` are supplied. File type acceptance is suffix based; the field does not inspect file contents or trust extensions as a security boundary.

## Accessibility role and properties

The field is a labeled Group. Browse and Remove are ordinary named buttons, with Remove names containing the file name. The accepted-type hint and selection limit are associated description text. Rejections and changes are exposed in a polite status region. Disabled state applies to Browse and all Remove buttons. Native screen-reader output must be verified on supported platforms.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts` key `file-field`,
styled by `.e7-file-button`, `.e7-file-row*`, and `.e7-remove` in `site/src/demos/e7_expansion.css`
and `.ui-btn--primary`, `.ui-label`, `.ui-description` in `site/src/ui/ui.css`, with the shadcn
token mapping in `site/src/ui/tokens.ts`). It is resolved from the installed `Theme` the same way
Button, Select, Text Field, and Tabs do it. `high-contrast` is selected by theme name and keeps
solid colours. The shadcn roles this field uses (primary, muted-foreground, destructive) map to the
same tokens in light and dark themes, so no luminance split is needed. Derived colours use a
crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or tokens
are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Label | `text`, medium weight | same | same |
| Description | `text_muted` | `text_muted` | `text_muted` |
| Browse button (primary) | `accent` fill, `accent_text` label, `shadows.small` | same | `accent` fill and border, `accent_text` label, no shadow |
| Browse button hover | `accent` mixed 90% over `background` | same | border `text` |
| Status feedback | `text_muted` | `text_muted` | `text_muted` |
| File row icon and size | `text_muted` | `text_muted` | `text` icon, `text_muted` size |
| File name | `text` | `text` | `text` |
| Remove (text button) | `danger` label; hover fill `danger` mixed 10% over `background` | same | `danger` label, no hover fill |
| Keyboard focus (Browse) | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Keyboard focus (Remove) | a 3px ring of `focus` at 50% over a `background` fill | same | a 3px ring of opaque `focus` |
| Disabled | Browse and Remove colours composited and mixed 50% over `background`; button shadow alpha halved; no hover | same | Browse `background` fill with `disabled` border and label; `disabled` Remove label |

- **Focus** follows `:focus-visible`: GPUI's `focus_visible` style applies while a control's focus
  handle is focused and the last input was from the keyboard. The ring replaces the button's resting
  shadow. GPUI paints drop shadows as filled shapes that are not clipped to the element's outside,
  so a focused control always has an opaque fill (the Remove button's transparent fill composites to
  `background`). Ring corners use the control radius rather than CSS's radius-plus-spread.
- **Disabled** matches the web's `opacity: .5` on the disabled controls as one layer, the way Select
  and Text Field do it: each colour is composited over `background` and mixed 50% with it. GPUI
  element opacity is not used because it dims each painted part separately. The label,
  description, and file rows are not controls and stay at full strength, as on the web. High
  contrast keeps solid colours instead.
- **Geometry**: the field is a column with a `spacing.small` (8px) gap, matching the preview's
  `.ui-col`. The label is `typography.body` (14px) at medium weight; the description is
  `typography.body` (the preview's 13px description has no token). The Browse button matches
  Button's default primary size and fills the field width (`.e7-file-button`): height
  `controls.medium` (36px, shadcn `h-9`), padding `spacing.large` (16px, `px-4`), radius
  `radii.medium`, a transparent `borders.regular` border, and a medium-weight `typography.body`
  label. File rows have no fill (the preview's `.e7-file-row`), are at least `controls.xsmall`
  (28px, nearest to the preview's 30px) tall, and put the name group and Remove at opposite ends.
  The name group is a Lucide `file` icon in a `spacing.large` (16px, nearest to 15px) square, then
  the name in `typography.body` (13px has no token), then the size in `typography.caption` (12px,
  nearest to 11px), separated by `spacing.small` gaps (nearest to 6px). The icon is drawn as vector
  paths on a 24-unit grid with a 2-unit stroke and arcs for its rounded corners; there are no icon
  assets, and strokes use butt caps because GPUI does not expose lyon line caps. Remove is a text
  button with `typography.caption` (12px) text, `spacing.small` × `spacing.xsmall` padding
  (nearest to the preview's 6px × 3px), and radius `radii.small`. The 3px focus ring is the shadcn/ui
  ring width, a fixed component value.
- Rejections are reported as text in the polite status region, so colour is not the only signal.

## WAI-ARIA pattern reference

Use standard labeled-group, button, and polite-status semantics. File rows are not a listbox and do not claim selection-widget semantics.

## Platform notes

The pinned GPUI `0.3.5` API exposes `App::prompt_for_paths(PathPromptOptions)` with files/multiple configuration and an asynchronous oneshot result. It does not expose extension filters in the picker options, so accepted suffixes are checked after selection. Headless tests may simulate picker completion; a native OS picker should not be invoked by screenshot fixtures. Native assistive-technology behavior remains a platform check.

## Open questions

- Review file metadata exposed publicly and whether selection limits belong on this component.
- Decide whether future fields need an async loading state for file reads; this draft only presents paths.
- Verify focus restoration following removal and polite announcement timing with native assistive technology.
