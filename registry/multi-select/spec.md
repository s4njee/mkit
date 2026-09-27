---
spec_version: 1
component: multi-select
states:
  - id: closed
    description: Trigger shows selected labels or placeholder.
    fixture: closed_fixture
  - id: open
    description: Multi-selection listbox is anchored to the measured trigger, constrained to the window with a margin, and uses active row; long lists use a clipped virtual scroll viewport.
    fixture: open_fixture
  - id: disabled
    description: Control cannot receive focus or change.
    fixture: disabled_fixture
  - id: all_options_disabled
    description: Control is enabled but every option is unavailable, so navigation leaves it closed.
    fixture: all_options_disabled_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: Multi-select is focused
    action: Open and move active row.
    initial_state: closed
    expect: { state: open, event: open, focus_target: multi_select }
  - key: Space
    modifiers: []
    when: An option is active
    action: Toggle its selected membership and keep popup open.
    initial_state: open
    expect: { state: open, event: change, focus_target: multi_select }
  - key: Escape
    modifiers: []
    when: Popup is open
    action: Close and preserve current selected values.
    initial_state: open
    expect: { state: closed, focus_target: multi_select }
  - key: Home
    modifiers: []
    when: Popup is open
    action: Move active row to first enabled item and scroll it into view.
    initial_state: open
    expect: { state: open, focus_target: multi_select }
  - key: End
    modifiers: []
    when: Popup is open
    action: Move active row to last enabled item and scroll it into view.
    initial_state: open
    expect: { state: open, focus_target: multi_select }
  - key: Tab
    modifiers: []
    when: Multi-select is focused and popup is open
    action: Follow normal forward focus traversal and close the popup when focus leaves the trigger.
    initial_state: open
    expect: { state: closed, focus_target: next_tab_stop }
  - key: Tab
    modifiers: [Shift]
    when: Multi-select is focused and popup is open
    action: Follow normal backward focus traversal and close the popup when focus leaves the trigger.
    initial_state: open
    expect: { state: closed, focus_target: previous_tab_stop }
  - key: ArrowDown
    modifiers: []
    when: Multi-select is disabled
    action: Keep the popup closed, preserve values, and emit no event.
    initial_state: disabled
    expect: { state: disabled, event: none, focus_target: none }
  - key: ArrowDown
    modifiers: []
    when: Every option is disabled
    action: Keep the popup closed and emit no event.
    initial_state: all_options_disabled
    expect: { state: all_options_disabled, event: none, focus_target: multi_select }
accessibility:
  role: combobox
  properties:
    - { name: aria-haspopup, value: listbox }
    - { name: aria-value, value: selected option labels joined with commas or empty string }
    - { name: aria-multiselectable, value: true }
    - { name: aria-expanded, value: false, when: closed }
    - { name: aria-expanded, value: true, when: open }
    - { name: aria-disabled, value: true, when: disabled }
    - { name: active descendant, value: active option while open }
---

# Multi-select

## Purpose

Multi-select lets users choose option values from a bounded collection. Choose zero or more values.

## Anatomy

A labeled trigger displays the selected option labels. Opening it reveals an in-window multi-select listbox; active navigation is distinct from committed selection.

## States

Closed, open, disabled, and all-options-disabled states are defined in the manifest. The open state includes an active enabled option; disabled rows remain visible and cannot be selected. If every option is disabled, navigation and trigger activation leave the popup closed. A disabled control is omitted from Tab order and ignores keyboard actions, including navigation and dismissal.

## Props and events

Provide a persistent `label`, stable option IDs, labels, and optional disabled flags. Uncontrolled `new` accepts default values. Controlled construction emits a value request and applies owner updates through `set_values`. The `values`, `is_open`, and `active_option_id` accessors expose the current state for host coordination. `OpenChanged` fires once on user-driven visibility changes.

## Keyboard map

Register the component key context so host applications can rebind actions. Arrow keys move among enabled options; Home/End go to boundaries; Enter/Space opens or toggles membership while open; Escape closes without changing selection. Tab and Shift+Tab use GPUI's normal forward and backward focus traversal and close the popup when leaving the trigger. Selected values remain unchanged. Disabled ignores all of these actions.

## Pointer behaviour

Clicking the trigger toggles the popup. Clicking an enabled row toggles its membership and keeps the popup open; the row consumes its click so the trigger does not toggle. Disabled rows cannot change values and also keep the popup open.

## Accessibility role and properties

Expose a named combobox trigger with expanded state, selected option labels joined in option order as its accessible value (empty when none are selected), and a listbox popup. Options expose labels, selection, and disabled state. Multi-select listboxes expose multiselectable semantics. Disabled option rows set the AccessKit disabled property.

## Theme tokens used

Use Theme surface, elevated surface, text, muted text, border, accent, focus, disabled, spacing, radii, borders, and controls tokens. The viewport displays up to eight rows and uses GPUI `uniform_list` virtualization for the full option collection. Keyboard navigation scrolls the active row into view. The popup is placed in the window overlay at the measured trigger's left edge with a small spacing gap. Its measured row viewport height is compared with available space below and above; it flips above when the full viewport plus gap does not fit below and above offers more room. Window snapping preserves a medium spacing margin, and the viewport remains clipped when neither side fits.

For virtualized lists, scrolling to a later viewport must preserve source-option identity: clicking a mounted row after wheel scrolling toggles that row's membership and keeps the popup open. This interaction is covered by a large-list GPUI test.

## WAI-ARIA pattern reference

Follow the WAI-ARIA APG combobox and listbox patterns, adapted to GPUI's accessibility tree.

## Platform notes

Popup placement uses measured trigger geometry and available window space, flipping above when the bottom placement cannot fit and the upper side offers more room. GPUI window snapping preserves the medium margin; platform screen-reader verification remains pending. Outside pointer dismissal uses GPUI's popup hit testing.

## Open questions

Maintainer review is required for public naming and keyboard/accessibility contracts, including Tab dismissal and whether controlled selection should show a pending request before owner application.
