# Tooltip

> **Draft E7 component.** Its public API, accessibility contract, and visual baselines still need maintainer review. It is not marked `source_ready` for `cargo mkit add`.

A tooltip adds a short explanation when the pointer rests on a child control. The child remains responsible for its own accessible name and keyboard behavior.

## When to use it

- Explain an icon-only toolbar action while the pointer hovers over it.
- Give a compact chart marker a short description.
- Clarify an unfamiliar settings symbol without adding permanent text.

## API and state

`Tooltip::new(label, child)` is a stateless `RenderOnce` builder; `disabled` controls tooltip behavior. It emits no events.

## Keyboard and pointer behavior

| Key | Behavior |
| --- | --- |
| None | The wrapper registers no keys; its child keeps its own keyboard behavior. |
| Focus | Does not open the visual tooltip. |

No key context or action is registered. The child retains its keyboard behavior. The visual tooltip is not triggered by keyboard focus.

GPUI owns hover/leave timing and popup placement. The delay is fixed at 500 ms.

## Accessibility

The wrapper sets an accessible description with the label; the child keeps its own role and name. GPUI 0.3.5 exposes a description setter but no described-by node relationship builder. A description placed on a generic wrapper is not evidence that the focused descendant receives that description, so the current implementation does not claim a keyboard-focus description relationship.

## Theme

`elevated_surface`, `text`, `border`, `spacing.small`, `spacing.xsmall`, `radii.small`, `borders.hairline`, and `typography.caption` come from GPUI `Theme`.

## Current limits

The current GPUI API shows the visual popup on pointer hover or long press, not keyboard focus. A focused child is not proven to receive the wrapper description. Visible native popups cannot yet be captured by the headless screenshot harness.

For the exact state and event contract, see the checked-in `registry/tooltip/spec.md`. The [E7 overview](../everyday-components.md) tracks current test evidence; these pages are documentation drafts, not a claim that the full E5 conformance matrix has passed.
