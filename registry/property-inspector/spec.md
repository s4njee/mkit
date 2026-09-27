---
spec_version: 1
component: property-inspector
states:
  - id: expanded
    description: All enabled groups and typed property rows are visible with the first enabled row active.
    fixture: expanded_fixture
  - id: collapsed
    description: Appearance group is collapsed and its typed editor rows are absent.
    fixture: collapsed_fixture
  - id: mixed
    description: Number, text, and vector properties show mixed values while retaining typed editors.
    fixture: mixed_fixture
  - id: edited
    description: An opacity edit has been applied and shown as a non-default value.
    fixture: edited_fixture
  - id: disabled
    description: The panel is disabled; editor and group controls are removed from keyboard traversal.
    fixture: disabled_fixture
keys:
  - key: ArrowDown
    modifiers: []
    when: Inspector is focused and a property row is active.
    action: Move active property to the next visible enabled row, skipping collapsed groups and wrapping.
    initial_state: expanded
    expect:
      state: expanded
      focus_target: next_property
  - key: ArrowUp
    modifiers: []
    when: Inspector is focused and a property row is active.
    action: Move active property to the previous visible enabled row, skipping collapsed groups and wrapping.
    initial_state: expanded
    expect:
      state: expanded
      focus_target: previous_property
  - key: Space
    modifiers: []
    when: A boolean property row is active.
    action: Request the opposite boolean value.
    initial_state: expanded
    expect:
      event: value_changed
  - key: r
    modifiers: []
    when: A property row is active and has a default value.
    action: Request its default value.
    initial_state: expanded
    expect:
      event: value_changed
accessibility:
  role: group
  properties:
    - name: name
      value: Property inspector
    - name: row-role
      value: named group
    - name: mixed-value
      value: represented by a mixed label and empty editor value
    - name: disabled
      value: true
      when: disabled
    - name: collapsed
      value: Group header exposes expanded=false and descendants are omitted.
    - name: focus
      value: Disabled controls are not keyboard tabbable; ArrowUp/Down updates the active enabled visible property and wraps.
---

# Property inspector

## Purpose

Render an application supplied property schema as a compact panel for inspecting and editing one
or more selected objects. The visual reference is shadcn's restrained control styling: quiet
surfaces, fine borders, compact labels, and clear focus and selection states. All visual values
come from the mkit Global `Theme`.

## Anatomy

The inspector is a bordered panel containing one or more group headers and compact property rows.
Each row has a label, a type-specific editor, and a reset control.

## States

Expanded and collapsed groups, mixed values across a multi-selection, edited values, and a disabled
inspector/property are supported. A collapsed group hides its property rows. Mixed values remain
typed through the property's declared editor kind. Keyboard navigation skips disabled properties
and rows hidden in collapsed groups; it wraps among enabled visible properties. Disabling the whole
inspector removes its interactive descendants from keyboard traversal.

## Props and events

Groups have stable IDs, labels, and an initial expanded state. Properties have stable IDs, labels,
an editor kind, a current value, a default value, and optional disabled state. Supported editor
kinds are number, text, boolean, enum, colour, and vector (two or three numeric components).
Values are typed as `PropertyValue`; a current value may be `Mixed` when multiple selected objects
disagree. Defaults must match the editor kind. Enum properties include their allowed string options.

## Pointer behaviour

Groups can be expanded or collapsed. A group header toggles its own visibility and exposes its
expanded state to accessibility. Visible property rows show the editor for their declared type.
Number and vector values use decrement/increment controls, text enters a compact keyboard editing
mode when clicked, boolean uses a checkbox, enum cycles among its options, and colour exposes
red/green/blue channel increment and decrement controls for six-digit hex values. Mixed values
show “Mixed” and become a concrete value on the first edit. Each property has a reset-to-default
action. Disabled properties remain readable and do not emit events. The compact text editing mode
supports basic key insertion, Space, and Backspace; native IME, selection, clipboard, and cursor
editing remain open follow-up work.

This initial implementation supports controlled and uncontrolled values. `new(groups)` is
uncontrolled; user interactions update the local property value before emitting
`ValueChanged { property_id, value }`. `controlled(groups)` treats supplied values as
authoritative; interactions emit the proposal but do not update the display. The owner applies
accepted values using `set_value`. Group expansion is local state. Property schema/value replacement
uses `set_groups` and emits no edit events. The inspector emits no event for a no-op.

## Keyboard map

The root registers `PropertyInspector` actions so applications can rebind ArrowUp, ArrowDown,
Space, and `r`. Up/Down moves the active property among visible rows with wrapping. Space toggles a
boolean; `r` requests the active property's default. Arrow navigation updates active-row treatment;
Tab remains normal platform traversal among typed controls, while disabled controls are removed
from that traversal.
Group headers are buttons and toggle with Enter or Space. The group state remains inspector-owned.

## Accessibility role and properties

The root is a named group. Each group header is a button exposing its expanded state. Property rows
are named groups; editors expose their native checkbox/button roles and accessible names derived
from the property label. Mixed values are announced as mixed text, not as a stale concrete value.
Disabled editors expose disabled state. Reset controls are named “Reset {property label}”.

## Theme tokens used

Use Global `Theme` tokens for surface, elevated surface, text, muted text, border, accent, focus,
disabled state, spacing, radius, border width, control height, and typography. No fixed colours are
permitted. Compact row height and editor minimum widths derive from the small control token; fixed
editor width ratios are implementation layout choices for maintainer review.

## WAI-ARIA pattern reference

The panel uses the [Group pattern](https://www.w3.org/WAI/ARIA/apg/patterns/group/), with each
collapsible group header behaving as a button and exposing expanded state. Boolean editing follows
the [Checkbox pattern](https://www.w3.org/WAI/ARIA/apg/patterns/checkbox/).

## Platform notes

Group headers and controls use GPUI's cross-platform click, focus, action, and AccessKit APIs. The
basic compact text interaction does not yet provide native IME, selection, or clipboard handling;
hosts needing those features should offer an application-owned full text editor. Native colour
picker integration remains host-owned.

## Open questions

- Public property/schema builder naming and event naming need maintainer review.
- Should the text editor be composed with a reusable native text field once registry dependency
  layering supports shared registry components?
- Confirm whether keyboard navigation should include group headers or only property editors.
- Verify announced mixed-value semantics and collapsed group state in platform accessibility trees.
