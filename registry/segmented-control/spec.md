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

Labeled single-selection radio group rendered as a compact segmented choice. The group is a single
filled, rounded track with no outer border and no dividers between options. Each option is a
rounded trigger inside the track; the checked option is drawn as a raised pill on the track,
matching the documentation site's web preview (which shares the shadcn/ui Tabs styling).
Horizontal groups size to their content; vertical groups stack the same triggers and stretch each
one to the group width.
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
The look is resolved from the installed `Theme` in three variants and is identical to Tabs.
`high-contrast` is selected by theme name (the convention other registry components use); every
other theme is treated as dark when `mkit_core::contrast::relative_luminance(colors.background) <
0.5`, otherwise light. Derived colours use `mkit_core::contrast::composite`, the same
text-over-background mix the menu components and the site preview use; no new core tokens are added.

| Part | Light | Dark | High contrast |
|---|---|---|---|
| Group fill ("muted") | `text` at 4% over `background` | `text` at 12% over `background` | `background` |
| Group border | none | none | `borders.hairline` in `colors.border` |
| Unchecked text | `text_muted` | `text_muted` | `text_muted` |
| Checked fill | `background` | `text` at 15% ("input") over the muted fill, composited opaque | `accent` |
| Checked border | transparent | `text` at 15% ("input") | `accent` |
| Checked text | `text` | `text` | `accent_text` |
| Checked shadow | `shadows.small` | `shadows.small` | `shadows.small` (transparent in this theme) |
| Focus | border `focus` plus a 3px ring of `focus` at 50% | same | border `focus` plus a 3px ring of opaque `focus` |
| Focused fill | the resting fill, made opaque | same | same |
| Disabled option | 50% opacity | 50% opacity | 50% opacity |

GPUI paints drop shadows (the selected shadow and the focus ring) as filled shapes that are not
clipped to the element's outside, so the selected and focused fills are always opaque (the list
fill for an unselected focused item). The result matches the preview's composited colours; the ring
corners use the trigger radius rather than CSS's radius-plus-spread, so they are slightly squarer.

Geometry: the group has radius `radii.large` and padding `spacing.xsmall`; each option has radius
`radii.medium`, a `borders.hairline` border (transparent unless checked or focused), horizontal
padding `spacing.small`, and height `controls.medium - 2 × spacing.xsmall`, so a horizontal group is
exactly `controls.medium` tall. The web preview uses a 36px group with 3px padding and 10px option
padding; there is no 3px or 10px token, so the nearest tokens (4px and 8px, the latter matching
shadcn/ui's own `px-2`) are used and the options are 2px shorter than the preview. The 3px focus
ring is the shadcn/ui ring width; it fits inside the group padding and is a fixed component value.
Horizontal options share the group width equally (`flex: 1`). Labels use `typography.body` at
medium weight (500), matching shadcn/ui's `font-medium`; there is no font-weight token yet.
Tooltips keep their existing elevated-surface styling.

## WAI-ARIA pattern reference
[Radio Group](https://www.w3.org/WAI/ARIA/apg/patterns/radio/). Arrow-key selection follows the APG radio pattern; Space selects the focused radio.

## Platform notes

The current GPUI wrapper exposes an unavailable description for disabled options but no AccessKit disabled-property setter. Platform screen-reader output should be verified with the active accessibility tree.

## First application evaluation: Laika (E8.14)

Laika has compact segmented choices in its library and gallery views. `SegmentedControl` covers the useful interaction model (single selection, keyboard movement, focus, disabled items, tooltips, and accessible radio semantics), while the owning view handles view changes and actions such as opening Compare and Survey. The main library switch is composed inline with per-option application actions. Replacing it requires persistent host ownership, typed selection handling, and a synchronization path for keyboard and external view changes.

The E8.14 adoption spike aligns both applications on `gpui-pre` 0.3.5 and embeds a persistent controlled entity. Laika subscribes to `ValueChanged` and routes each key to its existing view action; `.tooltip(text)` keeps per-option help on the radio item and exposes it as its accessible description. API naming and active-platform accessibility output still need maintainer review.

## Open questions
Whether applications need optional deselection can be handled in a future API revision.
In the shadcn light theme, unchecked labels (`text_muted` on the muted group fill) measure about
4.36:1, just under the WCAG AA 4.5:1 text threshold; this matches shadcn/ui and the site preview but
needs maintainer review. The 3px focus ring width and medium label weight are fixed values pending
a ring-width and font-weight token.
Proposal only (no API added): the web preview allows a 16px leading icon with a 6px gap before the
label. `Item` derives `Clone`, `PartialEq`, and `Eq`, so an element slot does not fit it cheaply; an
optional `Item::icon(path)` storing a `SharedString` asset path, rendered with GPUI's `svg()` at
16px in the label colour, would need no new dependency (the app supplies the asset source). The gap
would use `spacing.xsmall`, the nearest token to 6px, and the accessible name would stay the text
label. This needs API review before implementation.
