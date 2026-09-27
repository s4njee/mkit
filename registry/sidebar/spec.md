---
spec_version: 1
component: sidebar
states:
  - id: selection
    description: Enabled navigation links with one current page.
    fixture: sidebar_selection
    screenshot_status: captured_by_e7_gallery_matrix
  - id: disabled
    description: Disabled navigation container and disabled links.
    fixture: sidebar_disabled
    screenshot_status: captured_by_e7_gallery_matrix
keys:
  - key: Enter
    modifiers: []
    when: an enabled navigation link is focused
    action: Request navigation to the focused link; controlled state changes only when the owner applies the requested current value.
    initial_state: selection
    expect: { event: value_changed }
accessibility:
  role: navigation
  properties:
    - name: label
      value: Component label
    - name: child_role
      value: link
    - name: current_page_description
      value: 'Current page on the enabled link whose value is selected'
---

# Sidebar

## Purpose

Provide a labeled navigation landmark containing links to app destinations.

## Anatomy

A labeled navigation landmark containing an ordered list of caller-provided links. Items have a stable value used as the application route key and a visible label. The application owns route resolution.

## States

Enabled links, one current-page link or no current page, disabled links, and a disabled navigation container. Selection can be controlled or uncontrolled; uncontrolled mode begins at the supplied default value.

## Props and events

The builder accepts an accessible label, items, current/default value, orientation, and disabled state. `SelectionChanged(Option<String>)` requests a current-page value when an enabled link is activated. Uncontrolled mode applies the request. Controlled mode emits the request without changing its current value until the owner calls `set_value`.

## Keyboard map

Each enabled link participates in normal Tab order. Enter activates the focused link. Disabled links are excluded from Tab order and ignore pointer and keyboard activation. The component does not add arrow-key navigation to a navigation landmark.

## Pointer behaviour

Click an enabled link to request navigation using the same event contract as Enter. Disabled links and a disabled container ignore clicks.

## Accessibility role and properties

Navigation landmark with accessible label, containing links with accessible names. The current enabled link has a “Current page” description; GPUI does not expose AccessKit `aria-current` for this element yet. Disabled links are described as unavailable and are not keyboard focusable.

## Theme tokens used

Theme surface, text, muted text, border, accent, focus, disabled; spacing, borders, radii, controls, typography tokens.

## WAI-ARIA pattern reference

[Navigation landmark](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/). Links use normal document Tab order.

## Platform notes

The application owns destinations and route changes. GPUI's current element API does not expose AccessKit's `aria-current` or link URL properties; current-page state is represented with selected state and a visible current marker until that API is available. Verify platform accessibility output when active-platform capture is available.

## Gallery screenshot fixtures

`sidebar_selection` renders the enabled vertical navigation list with Projects current and Home/Settings enabled. The matrix starts with Home current, requests navigation by clicking Projects, verifies that the uncontrolled value changes to Projects, then captures that selected state in light, dark, and high-contrast themes at 1× and 2×. `sidebar_disabled` renders both the navigation landmark and its links disabled; the matrix checks that its current value remains Home after a Projects click and captures the disabled appearance. Headless pixels do not substitute for active-platform accessibility snapshots.

## Open questions

Maintainer review is needed for whether to add a GPUI-level `aria-current`/link-destination API before claiming full navigation conformance.
