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

Theme background, surface, text, border, accent, accent text, focus, disabled; spacing, borders, radii, controls, typography tokens.

### Visual design

The look follows the shadcn sidebar used by the docs-site web preview (`site/src/demos/e7.ts`, `.ui-menu__item`).

- **Container.** A flat pane, not a bordered card: no radius, `spacing.small` (8px) padding, and one `borders.hairline` divider on the trailing edge (right edge when vertical, bottom edge when horizontal). Items are separated by `spacing.xsmall` (4px), the shadcn `SidebarMenu` gap, with no borders between items. The component does not set a width; the owner sizes the pane and items stretch to fill it.
- **Items.** Full-width rows, `controls.small` (32px) tall, which equals the web row (14px text × 1.43 line height + 2 × 6px padding). Horizontal padding `spacing.small` (8px), radius `radii.small`, text `typography.body` (14px) in `text`, transparent background and a transparent `borders.hairline` border reserved for the focus ring so focus does not shift layout.
- **Hover and current page.** Hover fills the row with the theme's accent fill. The current page uses the same fill plus medium (500) font weight.
- **Disabled.** Disabled links render at 50% opacity in the shadcn themes, matching the web preview; other themes keep the `disabled` text token so high-contrast text stays legible.
- **Focus.** Keyboard focus (`focus_visible`) colours the reserved item border with `focus` in every theme.

Derived fills (no public mkit-core API is added; the helper is local to this crate and follows the `active_row_colors` convention in DropdownMenu, ContextMenu, MenuBar and TagInput, keyed on the shadcn theme names):

| Role | shadcn light | shadcn dark | Other themes (light, dark, high contrast) |
| --- | --- | --- | --- |
| Container ("muted") | `text` mixed 4% into `background` | `text` mixed 12% into `background` | `surface` |
| Hover / current fill | `text` mixed 4% into the container fill | `text` mixed 12% into the container fill | `accent` fill with `accent_text` |
| Divider | `border` | `text` at 10% alpha | `border` |

The web preview maps both shadcn "muted" and "accent" to the same mix, so there the current row on a muted pane differs only by font weight. GPUI applies the mix a second time over the pane so the current page keeps a visible fill; this is a deliberate deviation. In high contrast the current page keeps the solid `accent` fill with `accent_text`, hover draws the item border in `border` (white), and focus draws it in `focus` (cyan), so every state stays distinguishable without relying on subtle fills.

## WAI-ARIA pattern reference

[Navigation landmark](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/). Links use normal document Tab order.

## Platform notes

The application owns destinations and route changes. GPUI's current element API does not expose AccessKit's `aria-current` or link URL properties; current-page state is represented with selected state and a visible current marker until that API is available. Verify platform accessibility output when active-platform capture is available.

## Gallery screenshot fixtures

`sidebar_selection` renders the enabled vertical navigation list with Projects current and Home/Settings enabled. The fixture places the sidebar in a 192px-wide owner pane, which is the web preview's 190px column rounded to the 8px grid, because the component does not choose its own width. The matrix starts with Home current, requests navigation by clicking Projects, verifies that the uncontrolled value changes to Projects, then captures that selected state in light, dark, and high-contrast themes at 1× and 2×. `sidebar_disabled` renders both the navigation landmark and its links disabled; the matrix checks that its current value remains Home after a Projects click and captures the disabled appearance. Headless pixels do not substitute for active-platform accessibility snapshots.

## Open questions

Visual proposals needing an API decision (not implemented, no API added): the web preview shows an optional small muted group label ("Workspace", `typography.caption`, `text_muted`) above the items and an optional 16px leading icon per item in `text_muted` with an 8px (`spacing.small`) gap. `Item` has no leading-element slot and the landmark label is accessibility-only, so neither is rendered. The icon also depends on the E9.2 icon set.

Maintainer review is needed for whether to add a GPUI-level `aria-current`/link-destination API before claiming full navigation conformance.
