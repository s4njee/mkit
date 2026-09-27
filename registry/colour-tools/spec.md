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

A compact, labelled panel contains a hue/saturation wheel and editable lightness field; numeric spinbuttons for sRGB, HSL, and OKLCH; three smaller grading wheels; and an eyedropper-unavailable status. Each grading wheel shows a neutral centre, colour by hue around its edge, and a marker for its current offset. Hue zero is at the top and increases clockwise in every wheel so the painted colour matches pointer input and marker position. The panel follows shadcn's restrained field and panel styling while drawing all chrome from the active mkit theme and wheel colours from edited data.

## States

`selected` is the regular editing state. `eyedropper-unavailable` reports that this build has no native screen-sampling API, without disabling the wheel or numeric alternatives. `grading` exposes independent shadows, midtones, and highlights offsets.

## Conformance fixtures

The dedicated screenshot matrix covers these three states in shadcn light, shadcn dark, and high
contrast at 1× and 2×. The selected fixture uses a blue sRGB working colour. The unavailable
fixture uses a warm working colour to show that numeric and wheel controls remain usable while
the status is present; this build shows the unavailable status in every state because no native
sampler exists. The grading fixture applies distinct nonzero offsets to all three tonal wheels
through the public setter before capture. Every fixture uses the same panel width and window
size. Accessibility cases still require an active platform tree.

## Props and events

`ColourTools` is an `Entity` view. `ColourTools::new(label, colour)` creates an uncontrolled editor. `ColourTools::controlled(label, colour)` leaves displayed colour at the owner's value and emits a `ColourChanged` proposal; the owner applies accepted values with `set_colour`. The same controlled contract applies to `GradingChanged` and `set_grading`: input emits the full proposed value while the displayed value remains unchanged until the owner echoes an accepted value through the setter. Reapplying an equal value or proposing a value equal to the current state emits no event. All nine numeric spinbuttons are editable: focus a field and press Enter (or click it), type a number, then press Enter to finish. Escape restores the value from before editing. sRGB fields accept integer byte values; HSL fields accept degrees and percentages; OKLCH fields accept lightness, chroma, and hue. Input conversion is finite and clamped at the model boundary. For OKLCH values outside sRGB, conversion clips each encoded sRGB channel independently to [0, 1]; it does not preserve OKLCH hue/lightness by chroma reduction. Subsequent OKLCH display values are derived from the clipped sRGB colour. The GPUI interface does not expose an OS screen sampler here, so the panel reports that eyedropper sampling is unavailable and emits no false sample. Events fire only when a proposed value differs from current state.

## Keyboard map

The hue/saturation wheel and grading wheels register the `MkitColourTools` key context. Arrow keys adjust hue and saturation; Shift+Up/Down adjusts lightness on the main wheel. Numeric spinbuttons are tab stops: Enter starts editing, digits, decimal point, minus, and Backspace edit the draft, Enter finishes, and Escape restores the starting colour. Grading wheels expose their own focus and arrow-key adjustments. Host apps can rebind the named actions.

## Pointer behaviour

Dragging or clicking the main wheel selects hue by angle and saturation by radius. The adjacent HSL lightness field edits the third channel. Each grading wheel updates its own hue and saturation offset. The controls clamp to their supported range and emit the typed event contract. Eyedropper use is platform-dependent and remains explicitly unavailable until GPUI exposes an OS-backed screen sampler in this workspace.

## Accessibility role and properties

The panel requests group role with a caller-provided label and description. The main wheel requests slider semantics and exposes hue and saturation in its value text. Numeric fields request spinbutton role with names, current numeric value, and model-specific min/max. Each grading wheel requests slider role and exposes its zone name and current hue/saturation. The eyedropper-unavailable message is a status description, not an actionable button. Native accessibility snapshots and platform announcements remain to be verified by the host harness.

## Theme tokens used

Panel, fields, borders, text, focus, and disabled styling use the GPUI Global `Theme` surface, elevated surface, border, text, text-muted, focus, accent, and disabled colors. Padding and control geometry use spacing, controls, radii, and border-width tokens. The primary wheel diameter is 4.5 times `controls.large`; grading wheels are 1.6 times `controls.large`, keeping their relative scale across themes. Wheel hues represent edited color data and are the only colors not sourced from the theme; all wheel geometry and selection outlines use theme tokens.

## WAI-ARIA pattern reference

The numeric values follow the [Spinbutton pattern](https://www.w3.org/WAI/ARIA/apg/patterns/spinbutton/) and continuous adjustments follow the [Slider pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider/). The wheel itself is a custom spatial control with keyboard and numeric alternatives.

## Platform notes

GPUI 0.3.5 in this workspace has no supported cross-platform eyedropper API. This draft reports the sampler unavailable and keeps numeric alternatives. A macOS-specific implementation must be added only after the platform permission flow, cancellation, and sampling semantics are reviewed. WCAG contrast and color-management assumptions for display-P3 are not currently included; conversions use sRGB D65.

## Open questions

- Should grading wheel state be stored as three offsets, or as absolute colors relative to the source image?
- Maintainer review is required for the current channel-clipping policy for out-of-sRGB OKLCH inputs.
- Which OS permission and cancellation flow should the eyedropper follow?
- Public type names, keyboard steps, accessibility value format, API behavior, and visuals require maintainer review.
