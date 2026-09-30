---
spec_version: 1
component: file-drop-zone
states:
  - id: idle
    description: The zone accepts files and offers a keyboard reachable Browse action.
    fixture: idle_fixture
  - id: focused
    description: The Browse button has keyboard focus and shows the focus-visible ring.
    fixture: focused_fixture
  - id: drag-accepted
    description: A compatible OS file drag is over the zone.
    fixture: drag_accepted_fixture
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: Requires an OS-owned file drag session; a synthetic fixture would not prove native drag integration.
  - id: drag-rejected
    description: An incompatible OS file drag is over the zone.
    fixture: drag_rejected_fixture
    screenshot_status: unsupported_headless_capture
    screenshot_limitation: Requires an OS-owned file drag session; a synthetic fixture would not prove native drag integration.
  - id: selected
    description: Accepted file paths are represented by the controlled or internal selection.
    fixture: selected_fixture
  - id: disabled
    description: The zone rejects drops and Browse activation while disabled.
    fixture: disabled_fixture
keys:
  - key: Enter
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker configured by the accept and multiple props.
    initial_state: idle
    expect: { event: browse_requested, focus_target: same_browse_button }
  - key: Space
    modifiers: []
    when: The Browse button is focused and enabled.
    action: Open the platform file picker configured by the accept and multiple props.
    initial_state: idle
    expect: { event: browse_requested, focus_target: same_browse_button }
  - key: Enter
    modifiers: []
    when: A focused enabled Remove file button is selected.
    action: Propose removal of that file.
    initial_state: selected
    expect: { event: files_change, focus_target: next_remove_or_browse }
  - key: Space
    modifiers: []
    when: A focused enabled Remove file button is selected.
    action: Propose removal of that file.
    initial_state: selected
    expect: { event: files_change, focus_target: next_remove_or_browse }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided accessible label
    - name: description
      value: accepted file types and browse instruction
    - name: live
      value: polite for accepted/rejected feedback
    - name: disabled
      value: true
      when: disabled
---

# FileDropZone

## Purpose

Accept on-disk paths dragged from the operating system and provide a keyboard-accessible platform Browse action. This component does not read file contents; consumers own validation and loading.

## Anatomy

The zone contains an instruction/feedback region, a Browse button, and an optional selected-files presentation. The separate `FileField` component can render the same selected-file model with removable rows.

## States

`idle` is ready for files. `drag-accepted` and `drag-rejected` are based on real platform file paths and the configured extension/MIME-like suffix filter. `selected` presents current file paths. `disabled` makes the zone noninteractive. Programmatic path changes follow the same filtering rules as dropped or browsed paths.

## Props and events

The stateful `FileDropZone` accepts a label, optional description, `accept` filters (extensions such as `.png` and suffix/type tokens such as `image/*`), `multiple`, `disabled`, and an optional `files` value. Omitting `.files(...)` is uncontrolled and updates internal selected paths; `.default_files(paths)` can seed that uncontrolled selection. `.files(paths)` is controlled: the component emits a typed `FilesChange` proposal and waits for the parent to call `set_files` with the new paths. Rejected paths are included in the change event and are never silently added. When `multiple` is false, the first accepted path is proposed and further dropped/picker paths are rejected. Each selected item is a path and display name; actual file reads remain the caller's responsibility.

## Keyboard map

A labeled Group contains the Browse button followed by one Remove button per selected file. Tab reaches enabled controls in order; Enter and Space activate Browse or Remove through separate rebindable key contexts. Removing an item focuses the next row, previous row, or Browse. The Browse button opens GPUI's platform path picker. The picker supports file selection and the `multiple` setting. Escape belongs to the native picker. The component does not intercept unrelated shortcuts.

## Pointer behaviour

GPUI converts OS file drops to `ExternalPaths` and dispatches the typed drop payload to the target element. Drag-over styling is active only for compatible paths; incompatible paths show a reject cue and are reported, not selected. Leaving the target clears hover feedback. Clicking Browse opens `App::prompt_for_paths`; cancel leaves selection unchanged. No synthetic internal drag payload is treated as an OS file.

## Accessibility role and properties

The zone is a labeled Group, with a visible instruction and a native button named Browse files. Accepted/rejected feedback is exposed through a polite status announcement. Selection rows expose file names and removal controls are named buttons. Disabled zones remove Browse from activation and report disabled semantics. Headless tests can exercise component-produced state and button actions; actual OS drag gestures and native assistive-technology announcements require platform testing.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts` key `file-drop-zone`,
styled by `.e7-drop` in `site/src/demos/e7_expansion.css` and `.ui-btn--outline.ui-btn--sm`,
`.ui-description` in `site/src/ui/ui.css`, with the shadcn token mapping in `site/src/ui/tokens.ts`).
It is resolved from the installed `Theme` in three variants, the same way Button, Select, Text Field,
and Tabs do it. `high-contrast` is selected by theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" below is shadcn's `accent`/`secondary`: `text` mixed 4% (light) or 12%
(dark) into `background`. "Border" is shadcn's `--border`: `border` in light themes and `text` at 10%
in dark themes.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Zone fill | `background` | `background` | `background` |
| Zone border (dashed) | `border` | `text` at 10% | `border` |
| Zone pointer hover and accepted drag | border `focus`, fill muted mixed 45% over `background` | same | border `focus`, fill unchanged |
| Rejected drag | border `danger`, fill `danger` mixed 10% over `background` | same | border `danger`, fill unchanged |
| Icon (Lucide `inbox`) | `text_muted` | `text_muted` | `text` |
| Instruction (description) | `text`, medium weight | `text`, medium weight | `text`, medium weight |
| Status feedback | `text_muted` | `text_muted` | `text_muted` |
| Browse button (outline) | `background` fill, border as zone, `text` label, `shadows.small` | same | `background` fill, `border` border, `text` label, no shadow |
| Browse button hover | muted fill | muted fill | border `accent` |
| File row icon and size | `text_muted` | `text_muted` | `text` |
| File name | `text` | `text` | `text` |
| Remove (text button) | `danger` label; hover fill `danger` mixed 10% over `background` | same | `danger` label, no hover fill |
| Keyboard focus (Browse) | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Keyboard focus (Remove) | a 3px ring of `focus` at 50% over a `background` fill | same | a 3px ring of opaque `focus` |
| Disabled zone | every colour composited and mixed 50% over `background`; button shadow alpha halved; no hover | same | `disabled` border, text, icon, button border and label, and Remove label |

- **Focus** follows `:focus-visible`: GPUI's `focus_visible` style applies while a control's focus
  handle is focused and the last input was from the keyboard. The ring replaces the button's resting
  shadow. GPUI paints drop shadows as filled shapes that are not clipped to the element's outside,
  so a focused control always has an opaque fill (the Remove button's transparent fill composites to
  `background`). Ring corners use the control radius rather than CSS's radius-plus-spread. The zone
  itself is not focusable; the web preview's `:hover` treatment is applied to pointer hover and to an
  accepted OS file drag, which is the state the dashed zone is meant to signal.
- **Disabled** matches the web's `opacity: .5` as one layer, the way Select and Text Field do it:
  each colour is composited over `background` and mixed 50% with it. GPUI element opacity is not
  used because it dims each painted part separately. High contrast keeps solid colours instead.
- **Geometry**: the zone is a `borders.regular` dashed border (GPUI `border_dashed`) with radius
  `radii.medium` (the preview's `--ui-radius-md`). Content is a centred column with a
  `spacing.small` (8px) gap; padding is `spacing.xlarge` (24px, nearest token to the preview's
  22px) vertically and `spacing.medium` (12px) horizontally. The icon is Lucide `inbox` (the icon
  the preview draws) as vector paths on a 24-unit grid with a 2-unit stroke, rounded corners drawn
  as arcs, in a `spacing.xlarge` (24px, nearest to the preview's 22px) square; there are no icon
  assets, and strokes use butt caps because GPUI does not expose lyon line caps. Text is
  `typography.body` (14px; the preview's 13px description has no token). The Browse button matches
  Button's outline small size: height `controls.small` (32px, shadcn `h-8`), padding
  `spacing.medium` (12px, `px-3`), radius `radii.medium`, a `borders.regular` border, and a
  medium-weight `typography.body` label. The web preview shows no selected files; file rows follow
  FileField's row look: at least `controls.xsmall` (28px, nearest to the preview's 30px) tall,
  full width, a Lucide `file` icon in a `spacing.large` (16px, nearest to 15px) square with a
  `spacing.small` gap before the name, and a Remove text button with `typography.caption` (12px)
  text, `spacing.small` × `spacing.xsmall` padding (nearest to the preview's 6px × 3px), and radius
  `radii.small`. The 3px focus ring is the shadcn/ui ring width, a fixed component value.
- The accepted/rejected distinction includes the polite status text and does not rely on colour
  alone.

## WAI-ARIA pattern reference

The group uses ordinary browse button semantics and a polite status region; there is no established APG composite pattern for desktop file drop targets. Follow platform file picker and drag/drop conventions.

## Platform notes

The pinned GPUI `0.3.5` API exposes `FileDropEvent` translated to `ExternalPaths`, element `on_drop`/`drag_over` handlers, and `App::prompt_for_paths(PathPromptOptions)`. The picker returns asynchronously through a oneshot receiver. GPUI headless tests simulate drops but do not represent OS integration; macOS/Windows/Linux native drag and picker paths need real-platform verification. GPUI's picker API does not expose extension filters in `PathPromptOptions`; this draft filters returned paths and dropped paths itself, so picker filtering is advisory/unavailable.

## Open questions

- Review public `accept` token syntax and whether MIME type detection beyond file suffixes belongs in this UI component or app code.
- Decide whether the component should accept dropped directories; this draft accepts files only.
- Native OS drag state, live announcement timing, and picker cancellation need platform integration review.
