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

`Theme.spacing.xsmall/small/medium/large/xlarge/xxlarge`, `Theme.controls.medium`,
`Theme.typography.body/caption`, and `Theme.colors.text/text_muted/danger`. No component colours or
fixed sizes are hard-coded.

### Visual design

The look follows the docs-site web preview (`site/src/demos/e7.ts`, styled by `.ui-label`,
`.ui-field`, `.ui-description`, and `.ui-error-text` in `site/src/ui/ui.css`) and shadcn/ui's field
metrics. The same tokens apply in every theme, including high contrast, because the layout draws no
fills or borders of its own.

| Part | Default | Compact |
|---|---|---|
| Gap between rows (fields) | `spacing.xlarge` (24px, shadcn's field spacing) | `spacing.large` (16px) |
| Label-to-control column gap | `spacing.large` (16px, the web grid's column gap) | `spacing.medium` (12px) |
| Control-to-helper gap | `spacing.small` (8px, `.ui-field` `gap: 8px`) | `spacing.xsmall` (4px) |
| Label | `typography.body` (14px), medium (500) weight, `text` | same |
| Description | `typography.caption`, `text_muted` | same |
| Error | `typography.caption`, `danger` | same |

- **Label alignment.** The label box is at least `controls.medium` (36px, the default control
  height) tall and centres its text vertically, so a label lines up with a default-height control;
  the web preview does the same with `padding-top: 11px` on a 14px, line-height-1 label. Taller
  controls keep the label at their top.
- **Helper text size.** The web uses 13px descriptions and errors. There is no 13px token; the
  nearest tokens are 12px (`caption`) and 14px (`body`), and `caption` keeps helper text visibly
  secondary to the 14px label.
- **Row gap.** The web preview's grid uses 16px between rows; shadcn's own form and field examples
  space fields 24px apart, which reads better once descriptions and errors sit under controls. The
  default follows shadcn; `compact(true)` gives the web grid's 16px.
- Labels use medium weight, shadcn's `font-medium`; there is no font-weight token yet.

## WAI-ARIA pattern reference

Follow native form grouping conventions and each child control's relevant WAI-ARIA pattern.

## Platform notes

Rows remain horizontal at all widths; callers should choose a narrow label width when the available space is constrained.

## Open questions

Whether automatic label-to-control association should require a shared control ID.
