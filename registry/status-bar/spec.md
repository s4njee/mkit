---
spec_version: 1
component: status-bar
states:
  - id: full
    description: Leading/trailing status text, buttons, and progress are visible.
    fixture: full_fixture
  - id: compact
    description: Low-priority progress and status items collapse to fit the supplied width.
    fixture: compact_fixture
  - id: narrow
    description: The smallest supported width keeps never-collapse status content visible.
    fixture: narrow_fixture
  - id: disabled
    description: Disabled action buttons retain disabled semantics and leave tab order.
    fixture: disabled_fixture
  - id: updated
    description: Updated status text requests polite announcement without moving focus.
    fixture: updated_fixture
keys:
  - key: Enter
    modifiers: []
    when: A supplied enabled button in the status bar is focused.
    action: Activate the supplied button using its normal GPUI behavior.
    initial_state: full
    expect: { focus_target: same_button }
  - key: Space
    modifiers: []
    when: A supplied enabled button in the status bar is focused.
    action: Activate the supplied button using its normal GPUI behavior.
    initial_state: full
    expect: { focus_target: same_button }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided status bar label
    - name: status_role
      value: status
    - name: live
      value: polite
    - name: role
      value: progress_indicator
      when: full
    - name: disabled
      value: true
      when: disabled
---

# Status bar

## Purpose

Place concise status information and quick controls along a window's lower edge. Status updates are
polite and must not move keyboard focus.

## Anatomy

A horizontal bar has leading and trailing ordered item regions. Items may be polite status text,
caller-supplied buttons, or labeled determinate/indeterminate progress. A quiet border separates the
bar from content. Buttons remain ordinary independent controls; the bar does not add a composite
focus model.

## States

`full` shows all items. `compact` omits low-priority items when the caller-supplied available width
is insufficient. `narrow` retains `Never` items and then the highest priorities that fit. `disabled`
uses caller-provided disabled buttons. `updated` changes status text while focus remains on its
existing control. Width is explicit because RenderOnce has no stable child-measurement callback;
without `available_width`, no items collapse.

## Props and events

`StatusBar::new(label)` accepts ordered leading/trailing `StatusItem`s and an optional logical
`available_width`. Items have a stable key, content, estimated width, and collapse priority
(`Never`, `Low`, `Normal`, or `High`). Text width is estimated from the body typography token;
callers supply `estimated_width` for arbitrary elements and may override estimates for text.
Collapsing proceeds from lowest priority, preserving insertion order among equal priorities. If
items marked `Never` still exceed the available width, the host must provide a wider slot or accept
clipping. This is a stateless `RenderOnce` builder: the application owns text/progress updates and
button actions. There are no component events or controlled/uncontrolled modes.

## Keyboard map

The component defines no key context or actions. Supplied buttons preserve their normal Enter/Space
activation and native focus behavior. Tab traverses enabled controls in leading-to-trailing order;
disabled controls are omitted by the supplied button component. Updating text never focuses a
button or the status bar.

## Pointer behaviour

Buttons receive pointer input through their supplied elements. Text and progress are noninteractive.
Collapsing removes an item from layout and the accessibility tree; mark important controls
`Never` so they remain available at narrow widths.

## Accessibility role and properties

The outer bar requests a labeled group. Each status text item uses Status role and polite live-region
priority. Progress items expose ProgressIndicator role, label, min/max/value when determinate, and an
in-progress description when indeterminate. Supplied buttons keep their own names, disabled state,
and focus semantics. Live updates do not request focus. Headless semantics can verify roles and
properties, but announcement timing needs native assistive-technology review.

## Theme tokens used

The look follows the docs-site web preview (`site/src/demos/e7_expansion.ts`, styled by
`.e7-status-bar` in `site/src/demos/e7_expansion.css`, with the shadcn token mapping in
`site/src/ui/tokens.ts`): a compact strip on the window background with a top border and muted,
small text. Colours are resolved from the installed `Theme` in three variants. `high-contrast` is
selected by theme name (the convention other registry components use); every other theme is
treated as dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise
light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. Colour is never the only
status cue: status is text and progress has a labelled role.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Bar fill | `background` | `background` | `background` |
| Top border | `border` | `text` at 10% over `background`, composited opaque | `border` |
| Status text | `text_muted` | `text_muted` | `text` |
| Progress track | `accent` at 20% over `background` (Progress's `bg-primary/20`) | same | `background` with a `borders.hairline` `border` |
| Progress fill | `accent` | `accent` | `accent` |

- **Geometry.** The bar is `controls.medium` (36px, the preview's `min-height`) tall with a
  `borders.hairline` top border. Horizontal padding stays `spacing.medium`, items within a region
  are separated by `spacing.small`, and the regions by at least `spacing.medium`, because the
  collapse calculation uses exactly these tokens; the preview's 8px padding and 14px gaps are not
  used so that the documented collapse behaviour is unchanged.
- **Text.** Status text uses `typography.caption` (12px, the nearest token to the preview's 11px)
  and does not wrap. Width estimates for text items still use `typography.body`, so estimates are
  slightly generous and collapse decisions are unchanged.
- **Progress.** Progress's pill (`radii.pill`) track, `spacing.small` (8px) thick, in a
  `controls.small` slot. It keeps Progress's thickness so the high-contrast track border (2px on
  each side) still leaves the fill visible.
- **Buttons.** Buttons are caller-supplied and keep their own look, disabled treatment and focus
  ring. The preview's buttons are small outline buttons, so the screenshot fixture passes
  `Button` with `Variant::Outline` and `Size::Small` (`controls.small`, 32px), which fits inside the
  36px bar. The bar reserves no height for larger controls; callers should use small controls.
- **Focus.** The bar itself is not focusable. Keyboard focus belongs to the supplied buttons, whose
  own screenshot matrix covers the focus ring, so this matrix has no separate focused state.

## WAI-ARIA pattern reference

Status text follows the [WAI-ARIA Status role] and polite live-region guidance. Actions remain
ordinary buttons, not toolbar items.

## Platform notes

GPUI/AccessKit maps Status and ProgressIndicator roles and live priority. Actual polite announcement
timing varies by platform and requires native screen-reader testing. Available width is a logical
width supplied by the host layout.

## Open questions

- Maintainers should review the explicit `available_width` and estimated-width collapse API.
- Confirm whether a future version should expose a menu for collapsed controls instead of omitting
  low-priority items.
- Verify polite announcement timing and exact Status role mapping on supported native platforms.

[WAI-ARIA Status role]: https://www.w3.org/TR/wai-aria-1.2/#status
