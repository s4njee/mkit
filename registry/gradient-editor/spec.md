---
spec_version: 1
component: gradient-editor
states:
  - id: default
    description: A gradient with two protected endpoint stops and its rendered preview.
    fixture: default_fixture
  - id: selected-stop
    description: One stop is selected and its position and sRGB fields are editable.
    fixture: selected_stop_fixture
  - id: many-stops
    description: Three or more ordered stops are visible and may be edited or removed.
    fixture: many_stops_fixture
  - id: controlled
    description: Proposed edits emit typed events while displayed gradient remains owner-controlled.
    fixture: controlled_fixture
  - id: disabled
    description: Preview remains visible while editing controls ignore input.
    fixture: disabled_fixture
  - id: focused
    description: The editor has keyboard focus; its preview shows the focus ring.
    fixture: focused_fixture
keys:
  - key: ArrowLeft
    modifiers: []
    when: A stop is selected.
    action: Move selected stop left by one percent, clamped between neighbouring stops.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: ArrowRight
    modifiers: []
    when: A stop is selected.
    action: Move selected stop right by one percent, clamped between neighbouring stops.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Shift-ArrowLeft
    modifiers: [shift]
    when: A stop is selected.
    action: Move selected stop left by one tenth percent, clamped between neighbouring stops.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Shift-ArrowRight
    modifiers: [shift]
    when: A stop is selected.
    action: Move selected stop right by one tenth percent, clamped between neighbouring stops.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Delete
    modifiers: []
    when: A non-endpoint stop is selected.
    action: Delete the selected stop.
    initial_state: many-stops
    expect: { event: gradient_changed }
  - key: Home
    modifiers: []
    when: An interior stop is selected.
    action: Move the selected stop as far left as neighbouring-stop spacing allows.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: End
    modifiers: []
    when: An interior stop is selected.
    action: Move the selected stop as far right as neighbouring-stop spacing allows.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Tab
    modifiers: []
    when: The gradient editor has focus.
    action: Select the next stop.
    initial_state: selected-stop
    expect: { event: stop_selected }
  - key: r
    modifiers: []
    when: A stop is selected.
    action: Increase its red channel by one byte step.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Shift-R
    modifiers: [shift]
    when: A stop is selected.
    action: Decrease its red channel by one byte step.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: g
    modifiers: []
    when: A stop is selected.
    action: Increase its green channel by one byte step.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Shift-G
    modifiers: [shift]
    when: A stop is selected.
    action: Decrease its green channel by one byte step.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: b
    modifiers: []
    when: A stop is selected.
    action: Increase its blue channel by one byte step.
    initial_state: selected-stop
    expect: { event: gradient_changed }
  - key: Shift-B
    modifiers: [shift]
    when: A stop is selected.
    action: Decrease its blue channel by one byte step.
    initial_state: selected-stop
    expect: { event: gradient_changed }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided label
    - name: description
      value: Gradient preview with editable color stops. Select a stop, adjust its position and color, add or remove stops.
---

# Gradient editor

## Purpose

Edit a one-dimensional sRGB gradient and show the resulting gradient as a live preview. The editor
owns no persistence or paint pipeline; consumers receive a value object containing normalized
positions and colors.

## Anatomy

The bordered card contains a horizontal gradient preview with round stop handles placed at
normalized positions, a wrapping ordered stop list with theme-coloured labels beside small
caller-coloured swatches, grouped position increment fields, grouped RGB byte increment fields,
and Add/Remove buttons. Endpoints at 0 and 1 always exist and cannot
be moved or removed. Interior stops are ordered, may not cross their neighbours, and are limited to
16 total stops. Positions are normalized to `[0, 1]`. Preview sampling linearly interpolates each
sRGB channel between adjacent stops.

## States

The required states cover the default gradient, a selected stop, many stops, controlled edits, and
disabled editing. The selected stop's handle gets the focus ring and its list entry is outlined;
keyboard focus on the editor rings the preview. While disabled, keyboard and
pointer input do not change the selected stop or gradient and emit neither `StopSelected` nor
`GradientChanged`; owner updates through `set_gradient` still apply.

## Conformance fixtures

The dedicated screenshot matrix renders these five states at shadcn light, shadcn dark, and high
contrast, each at 1× and 2×. The default fixture uses two protected endpoints; selected-stop
adds an interior stop and selects it through the public setter; many-stops uses five ordered
coloured stops; controlled uses a distinct three-stop value with the selected interior stop;
disabled keeps the default gradient visible while disabling editing; focused dispatches an unbound
Tab keystroke so the last input is the keyboard, moves focus to the editor (its only tab stop), and
asserts focus before capture. The controlled proposal
contract is asserted in a separate real-GPUI interaction test. Every screenshot uses the same
panel width and window size. Active-platform accessibility remains a separate check.

## Props and events

`GradientEditor` is a stateful GPUI `Entity`. `new(label, gradient)` is uncontrolled: an accepted
edit updates the visible model before emitting `GradientChanged`. `controlled(label, gradient)` emits
the same proposed complete gradient while retaining the value supplied by the owner. The owner
applies an accepted proposal with `set_gradient`; that owner echo emits no interaction event.
`GradientChanged` carries the full ordered
`Gradient` value. Stop selection emits `StopSelected(index)`. `add_stop`, `move_stop`,
`set_stop_colour`, and `delete_stop` are available as typed model operations. Add inserts a stop at
the midpoint of the widest interval using the preview color at that position. A no-op emits no
event, including a move clamped to its current position or an equal colour proposal. Every color uses
a local finite, clamped linear channel representation (`0..=1`) to keep this
registry component dependent only on `mkit-core` and GPUI.

## Keyboard map

The editor registers the `MkitGradientEditor` key context with rebindable actions. The editor root
is a keyboard focus target; while it has focus, arrows move the selected stop by 1%, Shift+arrows
use 0.1%, Home and End set an interior stop to the nearest legal position, and Delete removes an
interior stop. Tab and Shift+Tab select adjacent stops without transferring focus out of the editor.
While disabled, these keys are ignored.

## Pointer behaviour

The preview stop swatches select a stop; clicking the preview near a marker selects and moves it, while clicking
away from a marker adds a stop at that point. The selected stop can be moved with keyboard actions,
position buttons, or the RGB increment buttons. Add and Remove buttons offer pointer access to the
model operations.

## Accessibility role and properties

The editor root requests group role with a caller-supplied name and instructions. The preview is a
group with a textual summary. Each stop swatch requests button semantics with a stop-specific name,
selected state, current position in its name, and a fixed-position description for endpoints. The
position and RGB increment controls request button semantics with channel or direction names. Add
and Remove use button semantics. Keyboard actions work while the labeled editor group is focused.
Platform snapshots and announcement quality remain to be verified with the active harness.

## WAI-ARIA pattern reference

Stop selection follows the [Button pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/);
position adjustments follow the [Slider pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider/)
through the focused editor's keyboard actions and named increment buttons.

## Theme tokens used

The look follows the everyday components restyled in E7 and E13.1 (Card, Slider, Text field and
Button) and is resolved from the installed `Theme` in three variants. `high-contrast` is selected by
theme name; every other theme is dark when
`mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived colours
use a crate-local `color-mix` helper built on `mkit_core::contrast::composite`; no mkit-core API or
tokens are added. "Muted" is `text` mixed 4% (light) or 12% (dark) into `background`; "input" is
shadcn's `--input`: `border` in light themes and `text` at 15% in dark themes.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Card fill | `background` | `background` | `background` |
| Card and preview border (the "border" role) | `border` | `text` at 10% over `background` | `border` |
| Card shadow | `shadows.small` | `shadows.small` | none (`shadows.small` is transparent) |
| Preview fill | the gradient (data) | same | same |
| Stop handle | `background` fill, `accent` border, `shadows.small` | same | `background` fill, `accent` border |
| Stop guide line | `accent` | `accent` | `accent` |
| Selected stop handle | the resting handle plus a 3px ring of `focus` at 50% over `background`, composited opaque so it reads the same over any gradient | same | the same ring in opaque `focus` |
| Editor keyboard focus | preview border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Stop list entry | outline button: `background` fill, border `border`, `shadows.small`, `text` label | the same with border `text` at 10% | `background` fill, `border` border |
| Selected stop list entry | muted fill, `accent` border | same | `background` fill, `accent` border |
| Stop list hover (unselected) | muted fill | same | border `accent` |
| Position and RGB fields | Text field: fill `background`, input border, `shadows.small`; label `text_muted`, value `text` | fill `text` at 4.5% over `background`, input border | `background` fill, `border` border |
| Increment buttons | ghost icon buttons with vector minus and plus in `text`; hover muted | same | hover border `accent` |
| Add and Remove | outline buttons, as the stop list entries, medium-weight `typography.body` labels | same | same |
| Remove when unavailable, and every part while disabled | colours mixed 50% over `background`; shadow alpha halved | same | text, icons, borders and handle borders use `disabled`; fills stay `background` |

- **Handles.** Stop handles use the restyled Slider thumb: a `spacing.large` (16px) circle with an
  opaque `background` fill, a `borders.hairline` `accent` border and `shadows.small`, centred
  vertically on a `borders.hairline` guide line at the stop position. Handles sit above the preview,
  so the endpoint handles overhang its ends into the card padding instead of being clipped. The
  selected stop's handle adds a 3px focus ring, drawn as its own circle because GPUI keeps an
  element's corner radius when spreading a ring shadow. Keyboard focus is shown on the preview
  border instead, so selected and focused read differently; the focus look appears only while the
  editor has focus from keyboard input (`Window::last_input_was_keyboard`), matching
  `:focus-visible`.
- **Fields and buttons.** The position and RGB groups adopt the restyled Text field chrome (radius
  `radii.medium`, `borders.regular` border), with `typography.caption` labels and
  `typography.body` values that do not wrap. Increment buttons are ghost icon buttons with radius
  `radii.medium` and Lucide `minus` and `plus` drawn as 2-unit vector strokes in a
  `spacing.large` square. Stop list entries and Add/Remove use the restyled outline Button look with
  radius `radii.medium`, a `borders.regular` border and horizontal padding `spacing.medium`.
- **Density (provisional maintainer decision).** Pro components keep their existing control heights,
  plot sizes and hit targets; only colours, borders, radii, shadows, focus, hover and typography
  follow the everyday look. The preview stays `controls.large × 1.8` tall; stop list entries stay
  `controls.small` tall and `controls.large × 3.6` wide; increment buttons stay `controls.small`
  square; Add and Remove are `controls.small` (32px, the button small size) tall, close to their
  previous text-derived height; the pointer hit tolerance around a stop stays 0.035.
- **Card.** The card has radius `radii.large`, a `borders.hairline` border (2px in high contrast) and
  the existing `spacing.medium` padding. Fills under shadows and rings are opaque because GPUI paints
  them as filled shapes that are not clipped to the element's outside. The preview gradient is data
  and displays caller-selected colours; handles, rings and labels remain theme-derived.
- **Disabled.** The web look's `opacity: .5` applies as one layer. GPUI element opacity dims each
  painted part separately, so every theme colour is composited opaque over `background` and mixed
  50% with it instead; the preview gradient itself stays at full strength so the value remains
  readable. High contrast keeps solid colours.

## Platform notes

The native accessibility bridge and pointer capture beyond the preview bounds need active-platform
verification. The renderer samples the app-supplied sRGB stops rather than requesting a platform
gradient shader.

## Open questions

- Should the preview interpolation become selectable in a later version (sRGB, linear-light, or
  perceptual)?
- Does a consumer need alpha stops, radial gradients, or angle controls?
- Public API and accessibility mapping require maintainer review.
- The density decision above (keep today's heights and hit targets) is provisional; confirm it, or
  adopt shadcn's 36px control heights across the pro components.
- Should stop handles share one size token with the other pro components (new theme API)?
