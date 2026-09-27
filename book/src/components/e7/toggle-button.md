# Toggle button

Keep one persistent pressed or unpressed choice. This E7 component is a draft with an `implementation_in_progress` registry entry. Its public contract needs maintainer review.

![Toggle button dark theme baseline](../../images/e7/toggle-button.png)

*Dark theme, 2× baseline capture: `on` state.*

## API and state

`ToggleButton::new(label, default_pressed)` owns its value. `controlled(label, pressed)` emits `ValueChanged(bool)` requests without changing displayed state; the owner applies `set_pressed`. Uncontrolled activation updates state before emitting. Disabled activation is inert.

## Keyboard behavior

The component's named actions can be rebound by the host where applicable.

| Key or action | Current behavior |
| --- | --- |
| Enter, Space | Request the opposite pressed value when enabled. |

## Accessibility

Requests button role, accessible name, and pressed state. Disabled leaves Tab order and requests disabled state with an unavailable description. Platform pressed-state announcements remain unverified.

## Theme tokens

Colors: surface, text, accent, focus, disabled. Spacing: small/medium. Radius: small. Border: regular.

## Usage scenarios

- Make a Bold formatting control that remains pressed while bold is active.
- Pin a side panel open until the user toggles it again.
- Let an owner control a Preview mode and apply accepted `ValueChanged` requests.

## Verification and limits

Enter/Space adapter and focused controlled, uncontrolled, and disabled tests passed. 18/18 declared screenshot comparisons passed. These are focused draft checks, not release approval. The full E5 conformance matrix and maintainer review are outstanding. The current contract and evidence are in `registry/toggle-button/spec.md` and `docs/E7_AUDIT.md` in the repository.
