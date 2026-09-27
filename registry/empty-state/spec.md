---
spec_version: 1
component: empty-state
states:
  - id: basic
    description: Empty state shows title and description without an icon or actions.
    fixture: basic_fixture
  - id: illustrated
    description: Empty state includes an application-provided icon.
    fixture: illustrated_fixture
  - id: actionable
    description: Empty state includes one or more caller-provided actions.
    fixture: actionable_fixture
keys:
  - key: Tab
    modifiers: []
    when: Empty state contains actions.
    action: Move through caller-provided action controls in normal platform order.
    initial_state: actionable
    expect:
      focus_target: first_action
accessibility:
  role: group
  properties:
    - name: label
      value: title
    - name: description
      value: optional explanatory text
    - name: actions
      value: caller-provided controls, in insertion order
theme_tokens: [text, text_muted, spacing.small, spacing.medium, spacing.large, spacing.xlarge, typography.heading_small, typography.body]
open_questions: [Review layout API and whether the empty state should impose a minimum height.]
---

# Empty state

## Purpose

Explain why a view has no content and offer a clear path forward when one exists.

## Anatomy

A stateless centered group with optional caller-provided icon, required title, optional description, and zero or more caller-provided action elements. The component does not create data or trigger application operations itself.

## States

Basic title and description, illustrated with an icon, and actionable with one or more controls. Missing description, icon, or actions are valid. The component has no internal state.

## Props and events

`EmptyState::new(title)` creates a `RenderOnce` builder. `.description(text)` adds supporting copy, `.icon(element)` adds an app-owned visual, and `.action(element)` appends an action control. Caller-provided controls own their interaction and events; EmptyState does not re-emit them.

## Keyboard map

The component binds no keys. Tab traverses caller-provided action controls in normal platform order; actions must supply their own accessible names and focus behavior.

## Pointer behaviour

Pointer behavior is delegated to caller-provided action elements. The surrounding empty state is not interactive.

## Accessibility role and properties

The root has group role and is labelled by its title. Description text is exposed as supporting text. The icon is decorative unless the caller's element adds a meaningful accessible name. Action controls remain child elements in insertion order with their own semantics.

## Theme tokens used

Read text, muted text, spacing, and typography values from the mkit-core Global `Theme`. The caller-provided icon and action elements retain their own styling. No fixed dimensions or literal colors are introduced.

## WAI-ARIA pattern reference

Empty state has no dedicated APG pattern. It uses the [WAI-ARIA Group Role] for a related title, description, and optional actions; contained buttons follow their own patterns.

## Platform notes

GPUI maps the named group through AccessKit. Screen readers should encounter the title, supporting description, then action controls in visual order. Verify this order in active platform accessibility snapshots.

## Open questions

- Maintainers should review the public builder methods and default alignment.
- Decide whether gallery layouts should provide minimum height externally or through an optional token-based size.

[WAI-ARIA Group Role]: https://www.w3.org/TR/wai-aria-1.2/#group
