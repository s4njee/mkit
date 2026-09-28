---
spec_version: 1
component: tabs
states:
  - id: selection
    description: First tab selected with manual activation.
    fixture: tabs_selection
  - id: disabled
    description: Entire tablist disabled.
    fixture: tabs_disabled
  - id: controlled
    description: Controlled selection with first tab selected.
    fixture: tabs_controlled
  - id: vertical
    description: Vertical tablist with first tab selected.
    fixture: tabs_vertical
  - id: focus_pending
    description: First tab selected while the third tab has focus.
    fixture: tabs_focus_pending
  - id: selected_three
    description: Third tab selected.
    fixture: tabs_selected_three
keys:
  - key: ArrowRight
    modifiers: []
    when: a tab is focused in horizontal orientation
    action: Move focus to the next enabled tab with wrapping without changing selection.
    initial_state: selection
    expect: { state: selection, focus_target: three, event: none }
  - key: ArrowLeft
    modifiers: []
    when: a tab is focused in horizontal orientation
    action: Move focus to the previous enabled tab with wrapping without changing selection.
    initial_state: selection
    expect: { state: selection, focus_target: three, event: none }
  - key: Home
    modifiers: []
    when: a tab is focused
    action: Move focus to the first enabled tab without changing selection.
    initial_state: selection
    expect: { state: selection, focus_target: one, event: none }
  - key: End
    modifiers: []
    when: a tab is focused
    action: Move focus to the last enabled tab without changing selection.
    initial_state: selection
    expect: { state: selection, focus_target: three, event: none }
  - key: Enter
    modifiers: []
    when: the third tab is focused and the first tab is selected
    action: Select the focused tab.
    initial_state: focus_pending
    expect: { state: selected_three, focus_target: three, event: selection_changed }
  - key: Space
    modifiers: []
    when: the third tab is focused and the first tab is selected
    action: Select the focused tab.
    initial_state: focus_pending
    expect: { state: selected_three, focus_target: three, event: selection_changed }
  - key: ArrowRight
    modifiers: []
    when: the tablist is disabled
    action: Keep focus and selection unchanged without emitting an event.
    initial_state: disabled
    expect: { state: disabled, focus_target: none, event: none }
  - key: ArrowRight
    modifiers: []
    when: a controlled tablist has the first tab selected
    action: Move focus without changing controlled selection.
    initial_state: controlled
    expect: { state: controlled, focus_target: three, event: none }
  - key: ArrowDown
    modifiers: []
    when: a tab is focused in vertical orientation
    action: Move focus to the next enabled tab with wrapping without changing selection.
    initial_state: vertical
    expect: { state: vertical, focus_target: three, event: none }
accessibility:
  role: tablist
  properties:
    - name: label
      value: Views
    - name: orientation
      value: horizontal by default; vertical when configured
---

# Tabs

## Purpose

Present related panels with one selected tab.

## Anatomy

Labeled semantic navigation control built from caller-provided items. The tablist is a single
filled, rounded track with no outer border and no dividers between items. Each tab is a rounded
trigger inside the track; the selected tab is drawn as a raised pill on the track, following the
shadcn/ui Tabs look used by the documentation site's web preview. Horizontal lists size to their
content; vertical lists stack the same triggers and stretch each one to the list width.

## States
Selected tab, unselected tabs, disabled tabs, and empty panel content. Controlled mode uses the supplied selected value; uncontrolled mode initializes from default selection and updates internally. Focus is tracked separately from selection to implement manual activation.

## Screenshot fixtures
Every matrix case uses one persistent `Entity<Tabs>` with the same three items: `One` (`one`), disabled `Skip` (`skip`), and `Three` (`three`), under the accessible group label `Views`. The screenshot frame shows the `Tabs` heading, state caption, tabs, then a muted note in that order. Fixtures use an enabled-item focus handle only where stated. `focus_pending` starts with `One` selected, focuses `One`, and dispatches the registered `tabs::Next` GPUI action; the disabled middle item is skipped, leaving `Three` focused while `One` stays selected. This action is setup only and must not activate the new tab.

| Fixture | Caption / note | Construction | Orientation | Selected | Focus after setup |
|---|---|---|---|---|---|
| `tabs_selection` | `Selection` / `Manual activation keeps focus separate from selection` | `Tabs::new("Views", items, Some("one"))` | Horizontal | One | None |
| `tabs_disabled` | `Disabled` / `All tabs are unavailable` | `Tabs::new("Views", items, Some("one")).disabled(true)` | Horizontal | One | None; no item can focus |
| `tabs_controlled` | `Controlled` / `Owner retains the selected tab` | `Tabs::controlled("Views", items, Some("one"))` | Horizontal | One | None |
| `tabs_vertical` | `Vertical` / `Arrow keys follow the vertical axis` | `Tabs::new("Views", items, Some("one")).orientation(Vertical)` | Vertical | One | None |
| `tabs_focus_pending` | `Focus pending` / `Three is focused; One remains selected` | `Tabs::new("Views", items, Some("one"))`, then focus One and dispatch `tabs::Next` | Horizontal | One | Three |
| `tabs_selected_three` | `Selected three` / `Three is selected` | `Tabs::new("Views", items, Some("three"))` | Horizontal | Three | None |

The screenshot matrix renders each fixture at 1x and 2x in shadcn light, shadcn dark, and high-contrast themes. Baselines are stored at `tests/baselines/<state>/<theme>-<scale>x.png`.

## Props and events
Label, items, selected/default value, disabled state, and orientation. `ValueChanged(Option<String>)` fires when the focused tab is activated or an enabled tab is clicked; in controlled mode this is a proposal and the displayed selection changes only after `set_value`.

## Keyboard map
Left/Right (horizontal) or Up/Down (vertical) moves focus among enabled tabs with wrapping. Home/End move focus to enabled endpoints. Enter/Space activates the focused tab. Actions are registered in `MkitTabs`; orthogonal arrow keys do nothing.

## Pointer behaviour
Click an enabled tab to focus and request its selection. Clicking a disabled tab has no effect.

## Accessibility role and properties
Tablist with accessible label and orientation; each item is Tab with selected state. Only the focused enabled tab is in the tab sequence; disabled tabs cannot receive focus or activate and are described as unavailable. Panels and `aria-controls` relationships are owned by the application.

## Theme tokens used
The look is resolved from the installed `Theme` in three variants. `high-contrast` is selected by
theme name (the convention other registry components use); every other theme is treated as dark
when `mkit_core::contrast::relative_luminance(colors.background) < 0.5`, otherwise light. Derived
colours use `mkit_core::contrast::composite`, the same text-over-background mix the menu components
and the site preview use; no new core tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| List fill ("muted") | `text` at 4% over `background` | `text` at 12% over `background` | `background` |
| List border | none | none | `borders.hairline` in `colors.border` |
| Unselected text | `text_muted` | `text_muted` | `text_muted` |
| Selected fill | `background` | `text` at 15% ("input") over the muted fill, composited opaque | `accent` |
| Selected border | transparent | `text` at 15% ("input") | `accent` |
| Selected text | `text` | `text` | `accent_text` |
| Selected shadow | `shadows.small` | `shadows.small` | `shadows.small` (transparent in this theme) |
| Focus | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Focused fill | the resting fill, made opaque | same | same |
| Disabled tab | 50% opacity | 50% opacity | 50% opacity |

GPUI paints drop shadows (the selected shadow and the focus ring) as filled shapes that are not
clipped to the element's outside, so the selected and focused fills are always opaque (the list
fill for an unselected focused item). The result matches the preview's composited colours; the ring
corners use the trigger radius rather than CSS's radius-plus-spread, so they are slightly squarer.

Geometry: the list has radius `radii.large` and padding `spacing.xsmall`; each trigger has radius
`radii.medium`, a `borders.hairline` border (transparent unless selected or focused), horizontal
padding `spacing.small`, and height `controls.medium - 2 × spacing.xsmall`, so a horizontal list is
exactly `controls.medium` tall. The web preview uses a 36px list with 3px padding and 10px trigger
padding; there is no 3px or 10px token, so the nearest tokens (4px and 8px, the latter matching
shadcn/ui's own `px-2`) are used and the triggers are 2px shorter than the preview. The 3px focus
ring is the shadcn/ui ring width; it fits inside the list padding and is a fixed component value.
Horizontal triggers share the list width equally (`flex: 1`). Labels use `typography.body` at
medium weight (500), matching shadcn/ui's `font-medium`; there is no font-weight token yet.

## WAI-ARIA pattern reference
[Tabs](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) manual activation model. Focus and selection are separate: arrows/Home/End only move focus, while Enter/Space or pointer activation request selection.

## Platform notes

The application owns route targets and panel composition. The current GPUI wrapper exposes an unavailable description for disabled items but no AccessKit disabled-property setter. Platform screen-reader output should be verified with the active accessibility tree.

## Open questions
Panel association can be extended when panel composition is standardized. Active platform accessibility tree verification remains pending.
In the shadcn light theme, unselected labels (`text_muted` on the muted list fill) measure about
4.36:1, just under the WCAG AA 4.5:1 text threshold; this matches shadcn/ui and the site preview but
needs maintainer review. The 3px focus ring width and medium label weight are fixed values pending
a ring-width and font-weight token.
