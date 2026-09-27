---
spec_version: 1
component: segmented-control
states:
  - id: selection
    description: First radio option selected.
    fixture: segmented_control_selection
  - id: disabled
    description: Entire radio group disabled.
    fixture: segmented_control_disabled
  - id: controlled
    description: Controlled selection with first option selected.
    fixture: segmented_control_controlled
  - id: vertical
    description: Vertical radio group with first option selected.
    fixture: segmented_control_vertical
  - id: empty
    description: No option selected.
    fixture: segmented_control_empty
  - id: selected_three
    description: Third radio option selected.
    fixture: segmented_control_selected_three
keys:
  - key: ArrowRight
    modifiers: []
    when: an option is focused in horizontal orientation
    action: Focus and select the next enabled option with wrapping.
    initial_state: selection
    expect: { state: selected_three, focus_target: three, event: selection_changed }
  - key: ArrowLeft
    modifiers: []
    when: an option is focused in horizontal orientation
    action: Focus and select the previous enabled option with wrapping.
    initial_state: selection
    expect: { state: selected_three, focus_target: three, event: selection_changed }
  - key: Home
    modifiers: []
    when: an option is focused
    action: Focus and select the first enabled option.
    initial_state: selection
    expect: { state: selection, focus_target: one, event: none }
  - key: End
    modifiers: []
    when: an option is focused
    action: Focus and select the last enabled option.
    initial_state: selection
    expect: { state: selected_three, focus_target: three, event: selection_changed }
  - key: Space
    modifiers: []
    when: an option is focused and no option is selected
    action: Select the focused option.
    initial_state: empty
    expect: { state: selection, focus_target: one, event: selection_changed }
  - key: ArrowRight
    modifiers: []
    when: the radio group is disabled
    action: Keep focus and selection unchanged without emitting an event.
    initial_state: disabled
    expect: { state: disabled, focus_target: none, event: none }
  - key: ArrowRight
    modifiers: []
    when: a controlled radio group has the first option selected
    action: Request the next option and focus it without changing controlled selection.
    initial_state: controlled
    expect: { state: controlled, focus_target: three, event: selection_changed }
  - key: ArrowDown
    modifiers: []
    when: an option is focused in vertical orientation
    action: Focus and select the next enabled option with wrapping.
    initial_state: vertical
    expect: { state: selected_three, focus_target: three, event: selection_changed }
accessibility:
  role: radiogroup
  properties:
    - name: label
      value: Views
    - name: orientation
      value: horizontal by default; vertical when configured
---

# Segmented Control

## Purpose

Choose one option from a compact set.

## Anatomy

Labeled single-selection radio group rendered as a compact segmented choice.
Each `Item` may include an optional tooltip. When supplied, it is shown after GPUI's delayed hover
interval and exposed as the radio item's accessible description. Tooltip interaction is attached to
the radio item itself; it does not add another focus stop or change pointer selection.

## States
Selected, unselected, and disabled options. Controlled mode uses the supplied selection; uncontrolled mode initializes from the default. Focus is roved among enabled options.

## Screenshot fixtures
Every matrix case uses one persistent `Entity<SegmentedControl>` with the same three options: `One` (`one`), disabled `Skip` (`skip`), and `Three` (`three`), under the accessible group label `Views`. The screenshot frame shows the `Segmented control` heading, state caption, control, then a muted note in that order. Fixtures use an enabled-option focus handle only where stated. The disabled middle option is skipped by movement actions.

| Fixture | Caption / note | Construction | Orientation | Selected | Focus after setup |
|---|---|---|---|---|---|
| `segmented_control_selection` | `Selection` / `One is selected` | `SegmentedControl::new("Views", items, Some("one"))` | Horizontal | One | None |
| `segmented_control_disabled` | `Disabled` / `All options are unavailable` | `SegmentedControl::new("Views", items, Some("one")).disabled(true)` | Horizontal | One | None; no option can focus |
| `segmented_control_controlled` | `Controlled` / `Owner retains the selected option` | `SegmentedControl::controlled("Views", items, Some("one"))` | Horizontal | One | None |
| `segmented_control_vertical` | `Vertical` / `Arrow keys follow the vertical axis` | `SegmentedControl::new("Views", items, Some("one")).orientation(Vertical)` | Vertical | One | None |
| `segmented_control_empty` | `Empty` / `No option is selected` | `SegmentedControl::new("Views", items, None)` | Horizontal | None | None |
| `segmented_control_selected_three` | `Selected three` / `Three is selected` | `SegmentedControl::new("Views", items, Some("three"))` | Horizontal | Three | None |

The screenshot matrix renders each fixture at 1x and 2x in shadcn light, shadcn dark, and high-contrast themes. Baselines are stored at `tests/baselines/<state>/<theme>-<scale>x.png`.

## Props and events
Label, options, optional item tooltip, selected/default value, disabled state, and orientation.
`Item::new(value, label).tooltip(text)` configures help text. `ValueChanged(Option<String>)` fires
on accepted pointer or keyboard selection requests; in controlled mode this is a proposal and the
displayed selection changes only after `set_value`.

## Keyboard map
Arrow keys for the configured orientation, plus Home/End, move focus and select an enabled option; selection follows focus as required by the radio-group pattern. Space selects the focused option. Actions are registered in `MkitSegmentedControl`; orthogonal arrow keys do nothing.

## Pointer behaviour
Click an enabled option to focus and request selection. Clicking a disabled option has no effect.

## Accessibility role and properties
Labeled radiogroup with horizontal or vertical orientation. Each item is a radio with checked state;
one enabled option is the roving tab stop. Disabled options are skipped and described as unavailable.
If a tooltip is present, its text is included in the item's accessible description as well as delayed
hover content. Optional deselection is unsupported.

## Theme tokens used
Theme surface, text, border, accent, accent text, focus, disabled; spacing, borders, radii, controls, typography tokens.

## WAI-ARIA pattern reference
[Radio Group](https://www.w3.org/WAI/ARIA/apg/patterns/radio/). Arrow-key selection follows the APG radio pattern; Space selects the focused radio.

## Platform notes

The current GPUI wrapper exposes an unavailable description for disabled options but no AccessKit disabled-property setter. Platform screen-reader output should be verified with the active accessibility tree.

## First application evaluation: Laika (E8.14)

Laika has compact segmented choices in its library and gallery views. `SegmentedControl` covers the useful interaction model (single selection, keyboard movement, focus, disabled items, tooltips, and accessible radio semantics), while the owning view handles view changes and actions such as opening Compare and Survey. The main library switch is composed inline with per-option application actions. Replacing it requires persistent host ownership, typed selection handling, and a synchronization path for keyboard and external view changes.

The E8.14 adoption spike aligns both applications on `gpui-pre` 0.3.5 and embeds a persistent controlled entity. Laika subscribes to `ValueChanged` and routes each key to its existing view action; `.tooltip(text)` keeps per-option help on the radio item and exposes it as its accessible description. API naming and active-platform accessibility output still need maintainer review.

## Open questions
Whether applications need optional deselection can be handled in a future API revision.
