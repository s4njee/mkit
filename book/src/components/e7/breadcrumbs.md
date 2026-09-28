# Breadcrumbs

> **Draft everyday component.** Its public API, accessibility contract, and visual baselines still need maintainer review. It is not marked `source_ready` for `cargo mkit add`.

Breadcrumbs show where the current page sits in a hierarchy. Earlier locations can link back; the final crumb names the current page.

## When to use it

- Show Workspace › Projects › Project details.
- Let someone return from a document to its parent folder.
- Orient a user inside a deeply nested settings area.

## Preview

![Breadcrumbs in its selection state](../../images/e7/breadcrumbs.png)

This is a checked-in dark-theme, 2× screenshot baseline of one state. The component spec lists the other captured states, themes, and scales.

## API and state

The stateless builder accepts an accessible label and ordered crumbs. Linked crumbs provide a caller-owned route target and can request navigation through the optional `on_navigate` callback. There is no current selection event; the caller supplies the current location.

## Keyboard and pointer behavior

| Key | Behavior |
| --- | --- |
| Tab / Shift+Tab | Move through enabled ancestor links in normal page order; skip the current page. |
| Enter | Invoke navigation for the focused ancestor. |
| Arrow keys | No special breadcrumb binding. |

Ancestor links participate in normal Tab order and activate with Enter through the rebindable `MkitBreadcrumbs` action context. The current page is not focusable. When the whole trail is disabled by its owner, links are not focusable and do not invoke navigation. Linked nodes use GPUI's tab index directly, so a stateless rerender should retain focus on the same ancestor when its index remains stable. A real GPUI rerender test verifies this for the unchanged trail; active-platform focus verification remains pending.

Click an enabled ancestor link to focus it and invoke `on_navigate` with its target. The current page is not interactive. A disabled trail ignores pointer activation.

## Accessibility

Navigation landmark with accessible label containing an ordered list. Ancestors are named links; final current page is plain text with a “Current page” description. GPUI does not expose AccessKit `aria-current` for this element yet.

## Theme

Theme text, muted text, focus, disabled; spacing, radii, borders and typography tokens.

Ancestors use muted text and change to full text colour on hover. The current page uses full text colour. Separators are chevrons drawn as vector paths, so they stay crisp at 1× and 2×. They use muted text and are sized from the body text token. Linked ancestors show a `focus`-coloured border when keyboard focused. Collapsing middle crumbs into a "…" button is not supported yet.

## Verification and limits

The generated Enter case passes 1/1 through a real GPUI adapter, and four GPUI interaction tests cover pointer activation, keyboard activation, disabled suppression, and focus retention when the parent rerenders an unchanged trail. A 12-case screenshot matrix passes baseline comparison for selection and disabled states across three themes and two scales.

The app supplies route targets and navigation callbacks. Native current-page and URL properties remain pending, as does focus behavior when the trail changes shape. The generated accessibility cases are pending because the headless test platform does not activate the accessibility tree. Broader visual coverage and maintainer API, spec, and visual review are outstanding.

For the exact state and event contract, see the checked-in `registry/breadcrumbs/spec.md`. The [everyday components overview](../everyday-components.md) tracks current test evidence.
