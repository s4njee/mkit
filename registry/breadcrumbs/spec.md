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

Theme text, muted text, accent, focus, disabled; spacing and typography tokens.

## WAI-ARIA pattern reference

[Breadcrumb](https://www.w3.org/WAI/ARIA/apg/patterns/breadcrumb/).

## Platform notes

GPUI's current element API does not expose AccessKit's `aria-current` or link URL properties. The current page is plain text and described as the current page; active-platform accessibility verification remains pending.

## Open questions

Review focus behavior when ancestors are inserted, removed, or reordered while the parent rerenders the stateless builder. Maintainer review is needed for whether to add a GPUI-level `aria-current`/link-destination API before claiming full navigation conformance.
