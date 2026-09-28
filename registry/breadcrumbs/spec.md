---
spec_version: 1
component: breadcrumbs
states:
  - id: selection
    description: Linked ancestor locations and a current page.
    fixture: breadcrumbs_selection
  - id: disabled
    description: A trail with navigation disabled by its owner.
    fixture: breadcrumbs_disabled
keys:
  - key: Enter
    modifiers: []
    when: An enabled ancestor link owns focus.
    action: Request navigation to that link's caller-owned target.
    initial_state: selection
    expect:
      event: navigate
accessibility:
  role: navigation
  properties:
    - name: label
      value: Component label
    - name: child_role
      value: list
    - name: current_page_description
      value: 'Current page description on final non-link item'
---

# Breadcrumbs

## Purpose

Show the current location within a hierarchy.

## Anatomy

A labeled navigation landmark containing an ordered list of ancestor links and a final current-page item.

## States

Intermediate linked locations, a current non-link location, and an owner-disabled trail. The current item is the final item. Disabled trails use the theme's disabled text token on every crumb. Empty trails render an empty navigation landmark.

## Props and events

The stateless builder accepts an accessible label and ordered crumbs. Linked crumbs provide a caller-owned route target and can request navigation through the optional `on_navigate` callback. There is no current selection event; the caller supplies the current location.

## Keyboard map

Ancestor links participate in normal Tab order and activate with Enter through the rebindable `MkitBreadcrumbs` action context. The current page is not focusable. When the whole trail is disabled by its owner, links are not focusable and do not invoke navigation. Linked nodes use GPUI's tab index directly, so a stateless rerender should retain focus on the same ancestor when its index remains stable. A real GPUI rerender test verifies this for the unchanged trail; active-platform focus verification remains pending.

## Pointer behaviour

Click an enabled ancestor link to focus it and invoke `on_navigate` with its target. The current page is not interactive. A disabled trail ignores pointer activation.

## Accessibility role and properties

Navigation landmark with accessible label containing an ordered list. Ancestors are named links; final current page is plain text with a “Current page” description. GPUI does not expose AccessKit `aria-current` for this element yet.

## Theme tokens used

Theme text, muted text, focus, disabled; spacing, radii, borders and typography tokens.

### Visual design

The look follows the shadcn breadcrumb used by the docs-site web preview (`site/src/demos/e7.ts`).

- **Text.** Every crumb uses `typography.body` (14px). Ancestors use `text_muted`; hovering an enabled ancestor link changes it to `text`. The current page uses `text`. An owner-disabled trail uses `disabled` for every crumb and separator.
- **Separators.** A chevron-right drawn with GPUI vector paths (`canvas` + `PathBuilder`), not a glyph or icon asset, so it stays crisp at 1× and 2×. It sits in a square box of `typography.body` (14px, the web icon size) using the Lucide `chevron-right` geometry (points 9,6 → 15,12 → 9,18 on a 24-unit grid), stroked with `borders.hairline` in `text_muted`. The hairline token gives 1px in the shadcn themes (Lucide's 2/24 stroke at 14px is about 1.2px) and 2px in high contrast. Separators are decorative and add no accessibility node.
- **Spacing.** Crumbs and separators are separated by `spacing.small` (8px). The web preview uses 6px, which is not a token and is equally far from `spacing.xsmall` (4px). `spacing.small` was chosen because the chevron box already has about 5px of internal space on each side and shadcn's desktop breadcrumb uses a 10px gap.
- **Focus.** Linked ancestors reserve a transparent `borders.hairline` border with `radii.small` corners, offset by an equal negative margin so the reserved border does not move text. Keyboard focus (`focus_visible`) colours that border with `focus`.

The web preview also shows a collapsed "…" ghost button (24px tall, 4px padding) for elided middle crumbs. Breadcrumbs has no elision API, so this is not rendered. It is recorded under open questions and no API was added.

## WAI-ARIA pattern reference

[Breadcrumb](https://www.w3.org/WAI/ARIA/apg/patterns/breadcrumb/).

## Platform notes

GPUI's current element API does not expose AccessKit's `aria-current` or link URL properties. The current page is plain text and described as the current page; active-platform accessibility verification remains pending.

## Open questions

Elision (a collapsed "…" button for middle crumbs, as in the web preview) is a proposal that needs an API decision. It is not implemented.


Review focus behavior when ancestors are inserted, removed, or reordered while the parent rerenders the stateless builder. Maintainer review is needed for whether to add a GPUI-level `aria-current`/link-destination API before claiming full navigation conformance.
