---
spec_version: 1
component: select
states:
  - id: closed
    description: Select button shows the selected label or placeholder.
    fixture: closed_fixture
  - id: open
    description: Listbox is anchored to the measured trigger, constrained to the window with a margin, and uses an active enabled option; long lists use a clipped virtual scroll viewport.
    fixture: open_fixture
  - id: disabled
    description: Select is omitted from sequential Tab focus and cannot change.
    fixture: disabled_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: Select is focused
    action: Open and move active option to next enabled item.
    initial_state: closed
    expect: { state: open, event: open }
  - key: Enter
    modifiers: []
    when: Popup is open
    action: Commit active option and close.
    initial_state: open
    expect: { state: closed, event: change }
  - key: Escape
    modifiers: []
    when: Popup is open
    action: Close without changing committed value.
    initial_state: open
    expect: { state: closed }
  - key: Home
    modifiers: []
    when: Popup is open
    action: Move active option to first enabled item and scroll it into view.
    initial_state: open
    expect: { state: open }
  - key: End
    modifiers: []
    when: Popup is open
    action: Move active option to last enabled item and scroll it into view.
    initial_state: open
    expect: { state: open }
accessibility:
  role: combobox
  properties:
    - { name: aria-haspopup, value: listbox }
    - { name: aria-value, value: selected option label or empty string }
    - { name: aria-expanded, value: false, when: closed }
    - { name: aria-expanded, value: true, when: open }
    - { name: active descendant, value: active option while open, when: open }
    - { name: aria-disabled, value: true, when: disabled }
---

# Select

## Purpose

Select lets users choose option values from a bounded collection. Choose one value.

## Anatomy

A labeled trigger displays the selected option labels. Opening it reveals an in-window combobox and listbox; active navigation is distinct from committed selection.

## States

Closed, open, and disabled states are defined in the manifest. The open state includes an active enabled option; disabled rows remain visible and cannot be selected.

## Props and events

Provide a persistent `label`, stable option IDs, labels, and optional disabled flags. Uncontrolled `new` accepts a default value. `value` reads the committed value and `is_open` reads popup visibility. Controlled construction emits `ValueChanged` as a value request and applies owner updates through `set_value`; controlled display state does not change until the owner applies the request. `OpenChanged` fires once when user input changes visibility. Disabled Selects ignore keyboard and pointer input.

## Keyboard map

Register the component key context so host applications can rebind actions. Arrow keys move among enabled options; Home/End go to boundaries; Enter/Space opens or commits (Space toggles membership for multi-select); Escape closes without a single-value change; Tab closes and traverses normally.

An enabled Select trigger participates in normal sequential Tab order. A disabled Select trigger is skipped by Tab traversal. Tab closes an open popup and advances to the next focus stop; Shift+Tab closes it and moves to the previous focus stop. Traversal preserves the committed value.

## Pointer behaviour

Clicking the trigger toggles the popup. Clicking an enabled row commits a single selection, emits `ValueChanged`, and closes the popup. Disabled rows cannot change values. Clicking outside the inline popup closes it through GPUI outside-hit testing.

## Accessibility role and properties

Expose a named combobox trigger with expanded state, the committed option label as its accessible value (empty when no option is selected), and a listbox popup. Options expose labels, selected state, and disabled state. GPUI supplies an active-descendant focus mapping while the trigger is focused; platform-specific screen reader announcements still require native accessibility verification. Multi-select listboxes expose multiselectable semantics.

## Theme tokens used

Use Theme surface, elevated surface, text, muted text, border, accent, focus, disabled, spacing, radii, borders, and controls tokens. The viewport displays up to eight rows and uses GPUI `uniform_list` virtualization for the full option collection. Keyboard navigation scrolls the active row into view. The popup is placed in the window overlay at the measured trigger's left edge with a small spacing gap. Its measured row viewport height is compared with available space below and above; it flips above when the full viewport plus gap does not fit below and above offers more room. Window snapping preserves a medium spacing margin, and the viewport remains clipped when neither side fits.

For virtualized lists, scrolling to a later viewport must preserve source-option identity: clicking a mounted row after wheel scrolling commits the row at that visible index, closes the popup, and emits only its value request. This interaction is covered by a large-list GPUI test.

## WAI-ARIA pattern reference

Follow the WAI-ARIA APG combobox and listbox patterns, adapted to GPUI's accessibility tree.

## Platform notes

Popup placement uses measured trigger geometry and available window space, flipping above when the bottom placement cannot fit and the upper side offers more room. GPUI window snapping preserves the medium margin; platform screen-reader verification remains pending. The current component exposes GPUI's active-descendant focus mapping, but platform screen-reader behavior and the synthesized combobox/listbox relationship have not yet been verified on a supported native platform. Outside pointer dismissal uses GPUI's outside-hit testing.

## Open questions

Maintainer review is required for public naming and keyboard/accessibility contracts.
