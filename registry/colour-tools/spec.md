---
spec_version: 1
component: colour-tools
states:
  - id: selected
    description: A colour is selected and editable through the wheel and numeric fields.
    fixture: selected_fixture
  - id: eyedropper-unavailable
    description: No native screen-colour sampling backend is available; the picker remains usable through its wheel and numeric controls.
    fixture: eyedropper_unavailable_fixture
  - id: grading
    description: Shadows, midtones, and highlights have independent grading offsets.
    fixture: grading_fixture
  - id: focused
    description: The hue-saturation wheel has keyboard focus; its handle shows the focus ring.
    fixture: focused_fixture
keys:
  - key: ArrowRight
    modifiers: []
    when: The hue-saturation wheel has focus.
    action: Increase saturation by one percent.
    initial_state: selected
    expect: { event: colour_changed }
  - key: ArrowLeft
    modifiers: []
    when: The hue-saturation wheel has focus.
    action: Decrease saturation by one percent.
    initial_state: selected
    expect: { event: colour_changed }
  - key: ArrowUp
    modifiers: []
    when: The hue-saturation wheel has focus.
    action: Increase hue by one degree.
    initial_state: selected
    expect: { event: colour_changed }
  - key: ArrowDown
    modifiers: []
    when: The hue-saturation wheel has focus.
    action: Decrease hue by one degree.
    initial_state: selected
    expect: { event: colour_changed }
  - key: Shift-ArrowUp
    modifiers: [shift]
    when: The hue-saturation wheel has focus.
    action: Increase lightness by one percent.
    initial_state: selected
    expect: { event: colour_changed }
  - key: Shift-ArrowDown
    modifiers: [shift]
    when: The hue-saturation wheel has focus.
    action: Decrease lightness by one percent.
    initial_state: selected
    expect: { event: colour_changed }
accessibility:
  role: group
  properties:
    - name: name
      value: caller-provided colour tools label
    - name: description
      value: Hue and saturation wheel with lightness control, sRGB, HSL and OKLCH values, and shadows, midtones and highlights grading wheels.
---

# Colour tools (E8.5 draft)

## Purpose

Select a working colour and adjust the shadows, midtones, and highlights of an image-grade operation. The component provides a hue/saturation surface, lightness adjustment, equivalent sRGB/HSL/OKLCH numeric values, and three grading wheels. The owner supplies the resulting values to its own image pipeline.

## Anatomy

A compact, labelled card contains a hue/saturation wheel with a round handle and editable lightness field; numeric spinbuttons for sRGB, HSL, and OKLCH; three smaller grading wheels; and an eyedropper-unavailable status. Each grading wheel shows a neutral centre, colour by hue around its edge, and a round handle for its current offset. Hue zero is at the top and increases clockwise in every wheel so the painted colour matches pointer input and marker position. The panel follows shadcn's restrained field and panel styling while drawing all chrome from the active mkit theme and wheel colours from edited data.

## States

`selected` is the regular editing state. `eyedropper-unavailable` reports that this build has no native screen-sampling API, without disabling the wheel or numeric alternatives. `grading` exposes independent shadows, midtones, and highlights offsets.

## Conformance fixtures

The dedicated screenshot matrix covers these three states in shadcn light, shadcn dark, and high
contrast at 1× and 2×. The selected fixture uses a blue sRGB working colour. The unavailable
fixture uses a warm working colour to show that numeric and wheel controls remain usable while
the status is present; this build shows the unavailable status in every state because no native
sampler exists. The grading fixture applies distinct nonzero offsets to all three tonal wheels
through the public setter before capture. The focused fixture uses the selected fixture's colour,
dispatches an unbound Tab keystroke so the last input is the keyboard, focuses the main wheel through
the public `Focusable` handle, and asserts focus before capture. Every fixture uses the same panel
width and window size. Accessibility cases still require an active platform tree.

## Props and events

`ColourTools` is an `Entity` view. `ColourTools::new(label, colour)` creates an uncontrolled editor. `ColourTools::controlled(label, colour)` leaves displayed colour at the owner's value and emits a `ColourChanged` proposal; the owner applies accepted values with `set_colour`. The same controlled contract applies to `GradingChanged` and `set_grading`: input emits the full proposed value while the displayed value remains unchanged until the owner echoes an accepted value through the setter. Reapplying an equal value or proposing a value equal to the current state emits no event. All nine numeric spinbuttons are editable: focus a field and press Enter (or click it), type a number, then press Enter to finish. Escape restores the value from before editing. sRGB fields accept integer byte values; HSL fields accept degrees and percentages; OKLCH fields accept lightness, chroma, and hue. Input conversion is finite and clamped at the model boundary. For OKLCH values outside sRGB, conversion clips each encoded sRGB channel independently to [0, 1]; it does not preserve OKLCH hue/lightness by chroma reduction. Subsequent OKLCH display values are derived from the clipped sRGB colour. The GPUI interface does not expose an OS screen sampler here, so the panel reports that eyedropper sampling is unavailable and emits no false sample. Events fire only when a proposed value differs from current state.

## Keyboard map

The hue/saturation wheel and grading wheels register the `MkitColourTools` key context. Arrow keys adjust hue and saturation; Shift+Up/Down adjusts lightness on the main wheel. Numeric spinbuttons are tab stops: Enter starts editing, digits, decimal point, minus, and Backspace edit the draft, Enter finishes, and Escape restores the starting colour. Grading wheels expose their own focus and arrow-key adjustments. Host apps can rebind the named actions.

## Pointer behaviour

Dragging or clicking the main wheel selects hue by angle and saturation by radius. The adjacent HSL lightness field edits the third channel. Each grading wheel updates its own hue and saturation offset. The controls clamp to their supported range and emit the typed event contract. Eyedropper use is platform-dependent and remains explicitly unavailable until GPUI exposes an OS-backed screen sampler in this workspace.

## Accessibility role and properties

The panel requests group role with a caller-provided label and description. The main wheel requests slider semantics and exposes hue and saturation in its value text. Numeric fields request spinbutton role with names, current numeric value, and model-specific min/max. Each grading wheel requests slider role and exposes its zone name and current hue/saturation. The eyedropper-unavailable message is a status description, not an actionable button. Native accessibility snapshots and platform announcements remain to be verified by the host harness.

## Theme tokens used

The look follows the everyday components restyled in E7 and E13.1 (Card, Slider and Text field) and
is resolved from the installed `Theme` in three variants. `high-contrast` is selected by theme name;
every other theme is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`,
otherwise light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Muted" is `text` mixed 4%
(light) or 12% (dark) into `background`; "input" is shadcn's `--input`: `border` in light themes and
`text` at 15% in dark themes.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Card fill | `background` | `background` | `background` |
| Card border and wheel outlines (the "border" role) | `border` | `text` at 10% over `background` | `border` |
| Card shadow | `shadows.small` | `shadows.small` | none (`shadows.small` is transparent) |
| Title and section heading | `text`, semibold | same | same |
| Hex value, "Numeric entry" and wheel names | `text_muted`, `typography.caption` | same | same |
| Wheel hues (colour data) | edited colour data | same | same |
| Wheel handle | `background` fill, `accent` border, `shadows.small` | same | `background` fill, `accent` border |
| Wheel with keyboard focus | its handle plus a 3px ring of `focus` at 50% over `background`, composited opaque so it reads over any hue | same | the same ring in opaque `focus` |
| Numeric field | Text field: fill `background`, input border, `shadows.small`; label `text_muted`, value `text` | fill `text` at 4.5% over `background`, input border | `background` fill, `border` border |
| Numeric field focused or editing | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Eyedropper status | muted fill, border role; message `text`, status `text_muted` | same | `background` fill, `border` border |

- **Handles.** The main wheel handle uses the restyled Slider thumb: a `spacing.large` (16px)
  circle with an opaque `background` fill, a `borders.hairline` `accent` border and `shadows.small`.
  Grading wheel handles are the same at `spacing.medium` (12px) to suit the smaller wheels. Each
  wheel is one focus target with one handle, so keyboard focus is shown on that handle: a 3px ring
  drawn as its own circle, because GPUI keeps an element's corner radius when spreading a ring
  shadow. The ring appears only while the wheel has focus from keyboard input
  (`Window::last_input_was_keyboard`), matching `:focus-visible`. Hue zero stays at the top and
  handles sit exactly where pointer input places them.
- **Wheel outlines.** Each wheel is outlined by a circle in the border role, `borders.hairline` for
  the main wheel and `borders.regular` for the grading wheels, so the wheel edge reads on the card
  in every theme. Wheel hues are edited colour data and are the only colours not sourced from the
  theme.
- **Numeric fields.** Each spinbutton adopts the restyled Text field chrome: radius `radii.medium`,
  a `borders.regular` border, `shadows.small`, a `typography.caption` label and a
  `typography.body` value. Editing and keyboard focus show the Text field focus look; a field that
  was clicked into for editing shows it too, as an input does. Fills under shadows and rings are
  opaque because GPUI paints them as filled shapes that are not clipped to the element's outside.
- **Card.** Radius `radii.large`, a `borders.hairline` border (2px in high contrast) and the existing
  `spacing.medium` padding, following the web preview's `.ui-card`. The eyedropper status row has
  radius `radii.medium` and stays a status description, not a button.
- **Density (provisional maintainer decision).** Pro components keep their existing control heights,
  plot sizes and hit targets; only colours, borders, radii, shadows, focus, hover and typography
  follow the everyday look. The primary wheel diameter stays 4.5 times `controls.large` and grading
  wheels stay 1.6 times `controls.large`; the whole wheel remains the pointer target. Numeric fields
  keep their `spacing.xsmall` padding and gap.
- The panel has no disabled state.

## WAI-ARIA pattern reference

The numeric values follow the [Spinbutton pattern](https://www.w3.org/WAI/ARIA/apg/patterns/spinbutton/) and continuous adjustments follow the [Slider pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider/). The wheel itself is a custom spatial control with keyboard and numeric alternatives.

## Platform notes

GPUI 0.3.5 in this workspace has no supported cross-platform eyedropper API. This draft reports the sampler unavailable and keeps numeric alternatives. A macOS-specific implementation must be added only after the platform permission flow, cancellation, and sampling semantics are reviewed. WCAG contrast and color-management assumptions for display-P3 are not currently included; conversions use sRGB D65.

## Open questions

- Should grading wheel state be stored as three offsets, or as absolute colors relative to the source image?
- Maintainer review is required for the current channel-clipping policy for out-of-sRGB OKLCH inputs.
- Which OS permission and cancellation flow should the eyedropper follow?
- Public type names, keyboard steps, accessibility value format, API behavior, and visuals require maintainer review.
- The density decision above (keep today's heights and hit targets) is provisional; confirm it, or
  adopt shadcn's 36px control heights across the pro components.
- Should wheel handles share one size token with the other pro components (new theme API)?
