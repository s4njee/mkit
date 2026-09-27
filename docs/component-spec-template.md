---
spec_version: 1
component: component-name
states:
  - id: idle
    description: Ready for interaction.
    fixture: idle_fixture
keys:
  - key: Enter
    modifiers: []
    when: component is focused
    action: Activate the component.
    initial_state: idle
    expect:
      event: activate
accessibility:
  role: button
  properties:
    - name: aria-pressed
      value: false
      when: idle
---

# [Component name]

## Purpose

[What the component is for and when to use it.]

## Anatomy

[Name the visible and semantic parts of the component.]

## States

[Describe each state declared in `states`, including entry/exit conditions and visible,
interactive, and accessibility changes. Add state-transition details here; the front block is the
stable state inventory used by tooling. Each state’s `fixture` names the conformance adapter fixture
that constructs it.]

## Props and events

[List properties, defaults, controlled/uncontrolled behavior when stateful, emitted events, and the
conditions under which events fire.]

## Keyboard map

[Document each key declared in `keys`, including focus conditions, modifiers, and resulting
behavior. Each key declares an `initial_state` and an `expect` mapping with one or more executable
assertions: resulting `state` (an ID in `states`), emitted `event` (event name), or `focus_target`
(fixture target ID). The conformance adapter maps state fixtures and focus target IDs to the
component under test. `when` and `action` remain explanatory prose and are never used to infer
assertions. Document focus movement and any additional keyboard behavior here.]

## Pointer behaviour

[Document pointer interactions, hit targets, and how pointer input changes state or emits events.]

## Accessibility role and properties

[Explain the declared role and properties, accessible name, focus behavior, and any state-dependent
accessibility changes.]

## Theme tokens used

[List the GPUI Global theme tokens used for each visual purpose. Do not list literal colors or
unapproved fixed sizes.]

## WAI-ARIA pattern reference

[Link the applicable WAI-ARIA Authoring Practices pattern. If none applies, state `custom` and
justify the interaction model.]

## Platform notes

[Record relevant platform conventions, differences, and limitations.]

## Open questions

[List unresolved decisions, or write `None`.]
