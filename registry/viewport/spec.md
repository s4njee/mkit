---
spec_version: 1
component: viewport
states:
  - id: actual-size
    description: App content shown at one world unit per logical pixel.
    fixture: viewport_actual_size
  - id: fitted
    description: Entire declared content extent centered within the viewport.
    fixture: viewport_fitted
  - id: panned
    description: Pointer, trackpad, or keyboard movement has changed the view offset.
    fixture: viewport_panned
  - id: zoomed
    description: A cursor-anchored wheel, pinch, or keyboard zoom has changed the scale.
    fixture: viewport_zoomed
keys:
  - key: ArrowRight
    modifiers: []
    when: Viewport has focus.
    action: Pan content left by one theme spacing step so the view moves right.
    initial_state: actual-size
    expect: { state: panned, event: transform_changed }
  - key: ArrowLeft
    modifiers: []
    when: Viewport has focus.
    action: Pan content right by one theme spacing step.
    initial_state: actual-size
    expect: { state: panned, event: transform_changed }
  - key: ArrowUp
    modifiers: []
    when: Viewport has focus.
    action: Pan content down by one theme spacing step.
    initial_state: actual-size
    expect: { state: panned, event: transform_changed }
  - key: ArrowDown
    modifiers: []
    when: Viewport has focus.
    action: Pan content up by one theme spacing step.
    initial_state: actual-size
    expect: { state: panned, event: transform_changed }
  - key: Equal
    modifiers: []
    when: Viewport has focus.
    action: Zoom in around the viewport center.
    initial_state: actual-size
    expect: { state: zoomed, event: transform_changed }
  - key: Minus
    modifiers: []
    when: Viewport has focus.
    action: Zoom out around the viewport center.
    initial_state: actual-size
    expect: { state: zoomed, event: transform_changed }
  - key: F
    modifiers: []
    when: Viewport has focus and content has a positive extent.
    action: Fit all content with theme-token padding.
    initial_state: actual-size
    expect: { state: fitted, event: transform_changed }
  - key: "1"
    modifiers: []
    when: Viewport has focus.
    action: Show content at 100 percent, centered.
    initial_state: zoomed
    expect: { state: actual-size, event: transform_changed }
accessibility:
  role: group
  properties:
    - name: name
      value: Canvas viewport
    - name: description
      value: Arrow keys pan; plus and minus zoom; F fits; 1 shows actual size.
---

# Viewport (E8.1 draft)

## Purpose

Provide a reusable spatial surface for an app-drawn canvas, image, or editor. The app supplies a paint callback and a finite world-space content extent. The component owns the view transform and exposes world-to-screen and screen-to-world mapping; it does not own the app's scene data. The first slice treats the content origin as world `(0, 0)` and accepts positive width and height.

## Anatomy

One clipped app-painted canvas sits below a compact toolbar. The toolbar shows current zoom and Fit, 100%, Zoom out, and Zoom in buttons. Optional horizontal and vertical rulers overlay the canvas edges with adaptive world-coordinate ticks. Optional app-supplied horizontal and vertical guide lines overlay the content. The app supplies the painter and content extent; it owns the scene's semantic objects.

## States

The declared actual-size, fitted, panned, and zoomed fixtures capture the transform at useful points. Pointer drag adds a transient gesture state without changing the public transform shape. Scale stays positive and finite, up to 16; Fit may use a scale below 0.1 for very large content.

## Conformance fixtures

The dedicated screenshot matrix renders a fixed 500 × 320 world-space artwork in a 440 × 320
logical-pixel viewport at light, dark, and high-contrast themes, each at 1× and 2×. Each fixture
focuses the real viewport and dispatches its rebindable keyboard actions: `1` for actual size,
`1` then `F` for fit, `1` then Right for pan, and `1` then `=` for zoom. The larger-than-window
artwork makes Fit visibly distinct from 100% in every theme. The harness asserts the resulting
transform before comparing the screenshot. Rulers and two static guides remain visible in every
case. This matrix covers visual states and action dispatch; active-platform accessibility and
physical trackpad behavior require separate checks.

## Props and events

`Entity<Viewport>` is stateful and emits typed `TransformChanged(ViewTransform)` after a user-originated pan, zoom, fit, or actual-size command. In uncontrolled mode the component applies the transform before emission. In controlled mode it emits a request and waits for the owner to call `set_transform`; the owner-supplied transform remains the displayed state. Reapplying an equal transform is a no-op. Scale must be positive and finite, capped at 16; offsets must be finite. Allowing scales below 0.1 lets Fit contain very large world extents. The app reads `transform()`, `world_to_screen`, and `screen_to_world` for overlays and hit testing.

The component uses a single focusable group named by `label`. The app must provide a separate semantic representation or keyboard-reachable controls for important content drawn into pixels; a labeled canvas alone cannot expose scene objects. The label and description are not hard-coded into the painter.

`show_rulers(true)` enables visual coordinate rulers; `guides(Vec<Guide>)` supplies static world-coordinate guide positions. `Guide::Horizontal(y)` and `Guide::Vertical(x)` use the same world coordinates as the painter and therefore move with the transform. Invalid non-finite guide positions are ignored. Rulers and guides are visual overlays; if guides are meaningful editable scene objects, the app supplies their semantic controls separately. The ruler strip height/width uses the theme's extra-small control size. Tick spacing adapts to keep labels roughly 50 logical pixels apart, choosing a 1/2/5 × 10ⁿ world increment. These overlays do not change the painter's bounds or coordinate transforms.

## Keyboard map

Arrow keys pan; `=` and `-` zoom around the center; `F` fits content; `1` shows actual size. These are named actions under `MkitViewport` and can be rebound by the host. Fit requires a positive content extent. The toolbar buttons use standard button activation.

## Pointer behaviour

Primary-button drag pans by pointer delta. A held Space key also allows the same pan gesture when the host uses primary drag for scene tools; the host can turn off ordinary drag pan. Line-based wheel deltas zoom around the pointer and keep the world point under it stationary. Pixel scroll deltas pan by their x/y displacement, matching a trackpad scroll gesture. GPUI `PinchEvent` zooms by `1 + delta` around the pinch center; non-positive factors do nothing. Keyboard pan/zoom and Fit/100% actions are registered in `MkitViewport`, so the app can rebind them. The toolbar offers mouse-accessible Fit, 100%, zoom out, and zoom in commands; those are also keyboard-reachable. Content is clipped to the viewport.

The cursor, keyboard, and toolbar use the same transform math. `fit` computes the smaller width/height scale after theme spacing padding; `actual_size` sets scale to 1 and centers the content. Panning does not clamp: apps may choose to show work outside the declared content extent. Zoom stays within its configured range.

## Accessibility role and properties

The focusable root requests group role, a caller-provided name, and a keyboard description. Each toolbar control requests button role and name. App-painted scene items still need an accessible alternative supplied by the app; the viewport cannot infer it from pixels. Live platform relationships and focus output remain unverified.

## Theme tokens used

The chrome follows the restyled everyday components (Button, Toolbar, Slider, Tree) and is resolved
from the installed `Theme` in three variants. `high-contrast` is selected by theme name; every other
theme is dark when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise
light. Derived colours use a crate-local `color-mix` helper built on
`mkit_core::contrast::composite`; no mkit-core API or tokens are added. "Muted" is `text` mixed 4%
(light) or 12% (dark) into `background`. "Divider" is `border` in light and `text` at 10% alpha in
dark.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Frame | `background` fill, divider border, radius `radii.large` | same | `background`, `border` |
| Frame keyboard focus | `focus` border plus a 3px ring of `focus` at 50% | same | `focus` border plus a 3px ring of opaque `focus` |
| Toolbar strip | `background`, divider under it | same | `background`, `border` |
| Zoom readout | `text_muted`, `typography.caption` | same | same |
| Toolbar buttons | the Button `outline` variant: `background`, `text`, `border` outline, `shadows.small`; hover muted fill | same with `text` at 10% over `background` as the outline | `background`, `text`, `border`; hover border `accent` |
| Toolbar button focus | `focus` border, opaque `background`, 3px ring of `focus` at 50% in place of the shadow | same | same ring in opaque `focus` |
| Ruler strips and corner | muted fill, divider along the canvas edge | same | `background`, `border` |
| Ruler ticks | divider | divider | `border` |
| Ruler labels | `text_muted`, `typography.caption` (12px) | same | same |
| Guide lines | `focus`, `borders.hairline` | same | same |
| Guide handles | the Slider thumb look: opaque `background` fill, `borders.hairline` border in the guide colour, `shadows.small` | same | same (the shadow is transparent) |

- **Density (maintainer decision, provisional).** Pro components keep their existing dense
  geometry: the toolbar stays `controls.small` (32px) tall with `controls.xsmall` (28px) buttons
  (`controls.small` minimum width, `spacing.small` padding), and the rulers stay
  `controls.xsmall` wide. Colours, borders, radii, shadows, focus, hover and typography follow the
  everyday components. Button labels use `typography.body` (14px) at medium weight (500), like
  Button; there is no font-weight token yet. The 3px focus ring is the shadcn/ui ring width.
- **Guide handles.** Each visible guide draws a `spacing.medium` (12px) circle in the ruler it
  crosses, against the canvas edge of the strip so it clears most tick labels. Handles are painted
  with vector quads, never glyphs. They are decorative: guides remain static, app-supplied overlays
  and expose no accessibility nodes.
- GPUI paints drop shadows as filled shapes, so the frame, buttons and handles keep opaque fills
  under their shadows and rings.
- The viewport has no disabled state. Every matrix state is captured with keyboard focus on the
  viewport (each fixture focuses it and dispatches its key actions), so the focus ring is covered in
  all 24 cases.

## WAI-ARIA pattern reference

No APG pattern describes a spatial canvas. The [component pattern map](../../docs/component-pattern-map.md) calls for keyboard pan/zoom and a semantic alternative for important drawn content. The toolbar buttons follow the Button pattern.

## Platform notes

GPUI 0.3.5 exposes `PinchEvent` and distinguishes line from pixel scroll deltas. The interaction harness dispatches both; physical trackpad behavior on each platform still needs manual verification. The macOS headless harness checks a gallery preview; other platforms and native accessibility still need verification.

## Open questions

Public names, key bindings, accessibility representation, and visual baselines need maintainer review. Ruler labels are visual and are not currently individually exposed as accessibility nodes.
