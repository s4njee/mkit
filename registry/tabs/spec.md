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

Labeled semantic navigation control built from caller-provided items.

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
Theme surface, text, muted text, border, accent, accent text, focus, disabled; spacing, borders, radii, controls, typography tokens.

## WAI-ARIA pattern reference
[Tabs](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) manual activation model. Focus and selection are separate: arrows/Home/End only move focus, while Enter/Space or pointer activation request selection.

## Platform notes

The application owns route targets and panel composition. The current GPUI wrapper exposes an unavailable description for disabled items but no AccessKit disabled-property setter. Platform screen-reader output should be verified with the active accessibility tree.

## Open questions
Panel association can be extended when panel composition is standardized. Active platform accessibility tree verification remains pending.
