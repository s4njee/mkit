---
spec_version: 1
component: form-layout
states:
  - id: default
    description: Labeled rows with helper text and controls using standard spacing.
    fixture: default_fixture
  - id: compact
    description: Labeled rows with reduced row, column, and helper-text spacing.
    fixture: compact_fixture
keys: []
accessibility:
  role: group
  properties: []
---

# Form layout

## Purpose

Align visible labels, controls, descriptions, and validation errors in a consistent form.

## Anatomy

A vertical stack of rows. Each row has a fixed-width label column followed by a flexible control column. Descriptions and errors appear below their control.

## States

Default and compact spacing. Child controls own focus, validation, and input state. An error is displayed after the description when both are supplied.

## Props and events

`label_width` sets the label column width in logical pixels (clamped to zero or greater); the default is four `spacing.xxlarge` units. `compact` selects denser row, label/control, and helper-text spacing. `label` names the group. A row contains a visible label, a control, and optional description and error. The component is stateless and emits no events.

## Keyboard map

No layout key bindings. Child controls keep their own key contexts and tab order.

## Pointer behaviour

The layout adds no pointer behavior; controls keep theirs.

## Accessibility role and properties

The root has the `group` role and an optional accessible name from `label`. Callers should associate each control with its visible row label; layout text alone does not set the child's accessible name. Child controls retain their own accessibility semantics.

## Theme tokens used

`Theme.spacing.xsmall/small/medium/large/xxlarge`, `Theme.typography.body/caption`, and `Theme.colors.text/text_muted/danger`. No component colors or fixed sizes are hard-coded.

## WAI-ARIA pattern reference

Follow native form grouping conventions and each child control's relevant WAI-ARIA pattern.

## Platform notes

Rows remain horizontal at all widths; callers should choose a narrow label width when the available space is constrained.

## Open questions

Whether automatic label-to-control association should require a shared control ID.
