---
spec_version: 1
component: badge
states:
  - id: count
    description: Compact numeric count, clamped to the configured maximum display.
    fixture: count_fixture
  - id: count_overflow
    description: Count above the configured maximum uses the caller-provided overflow label.
    fixture: overflow_fixture
  - id: status_success
    description: Success status with a visible dot and caller label.
    fixture: success_fixture
  - id: status_warning
    description: Warning status with a visible dot and caller label.
    fixture: warning_fixture
  - id: status_danger
    description: Danger status with a visible dot and caller label.
    fixture: danger_fixture
  - id: disabled
    description: Muted informational badge.
    fixture: disabled_fixture
keys: []
accessibility:
  role: generic
  properties:
    - name: label
      value: supplied label or visible text
      when: count
---

# Badge

## Purpose

Show small count or status metadata beside another piece of content.

## Anatomy

A rounded capsule contains either a count string or a status dot and label. Count badges can cap a displayed number and use an app-supplied overflow label. Status variants are neutral, success, warning, and danger.

## States

- `Count`: numeric value; `max_count` defaults to 99 and values above it display the supplied overflow label or `99+`.
- `Status`: neutral, success, warning, or danger dot with visible caller label.
- `Disabled`: muted visual treatment for unavailable metadata.

## Props and events

Stateless `RenderOnce` builder; no events. Props include count/status constructors, maximum count, overflow text, visible status label, and optional accessible name. Count formatting is numeric and locale formatting remains app-owned.

## Keyboard map

No key context. Badge is not interactive or focusable.

## Pointer behaviour

None.

## Accessibility role and properties

Noninteractive generic content. The visible count/status text supplies its accessible text; an optional accessible name overrides it. Status color is never the only status cue: status badges include a visible dot and text label. No live region is implied.

## Theme tokens used

Uses `Theme.colors.surface`, `text`, `text_muted`, `border`, `accent`, `success`, `warning`, `danger`, and `disabled`; `Theme.typography.caption`, `spacing.xsmall/small`, `radii.pill`, and `borders.hairline`.

## WAI-ARIA pattern reference

No dedicated APG pattern applies to static badge content. Semantics follow generic text content and avoid live-region announcements for static values.

## Platform notes

The status dot is accompanied by text for color-independent meaning. Layout is checked at 1× and 2× in light, dark, and high-contrast themes.

## Open questions

- Maintainer review is needed for the public count overflow and status variant API.
