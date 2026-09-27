# Button

Trigger a command such as saving or submitting. This E7 component is a draft with an `implementation_in_progress` registry entry. Its public contract needs maintainer review.

![Button dark theme baseline](../../images/e7/button.png)

*Dark theme, 2× baseline capture: `idle` state.*

## API and state

`Button::new(label)` is a stateless `RenderOnce` builder. Choose variant, size, optional leading/trailing element, disabled, loading, and accessible label. `on_activate` runs for primary click or keyboard activation. `on_click` runs only for a primary click, before `on_activate` when both exist. Disabled and loading suppress both.

## Keyboard behavior

The component's named actions can be rebound by the host where applicable.

| Key or action | Current behavior |
| --- | --- |
| Enter, Space | Activate when enabled and focused; disabled and loading do nothing. |

## Accessibility

Requests button role. Visible text supplies the name unless `aria_label` overrides it. Disabled and loading leave Tab order and request disabled state, with “Unavailable” or “Loading” descriptions. Busy-state announcement is unverified.

## Theme tokens

Colors: surface, text, border, accent, focus, danger, disabled. Spacing: small/medium. Radius: medium. Border: regular. Typography: body. Control sizes: xsmall/medium/large.

## Usage scenarios

- Save changes in a settings form; handle `on_activate` to persist the form.
- Submit a checkout step with loading enabled while the request runs.
- Offer a destructive Delete action using the destructive variant and an explicit confirmation flow.

## Verification and limits

Enter/Space adapter cases and focused pointer tests passed. 18/18 declared screenshot comparisons passed. Generated accessibility cases are pending. These are focused draft checks, not release approval. The full E5 conformance matrix and maintainer review are outstanding. The current contract and evidence are in `registry/button/spec.md` and `docs/E7_AUDIT.md` in the repository.
