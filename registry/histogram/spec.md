---
spec_version: 1
component: histogram
states:
  - id: luminance
    description: Read-only luminance distribution supplied by the application.
    fixture: luminance_fixture
  - id: rgb
    description: Read-only per-channel red, green, and blue distributions.
    fixture: rgb_fixture
  - id: clipping
    description: Optional shadow and highlight clipping indicators are visible.
    fixture: clipping_fixture
  - id: empty
    description: No bins are available; the chart area is empty while its summary remains available.
    fixture: empty_fixture
keys: []
accessibility:
  role: img
  properties:
    - name: accessible name
      value: Caller-supplied label
    - name: description
      value: Caller-supplied summary with channel and clipping detail
---

# Histogram

## Purpose

Show the distribution of image values in an editing or analysis tool. The application computes
the data; Histogram only renders it.

## Anatomy

A restrained bordered panel contains the bin plot, optional channel legend, and optional clipping
status indicators. The panel uses the shared surface, border, text, and semantic colors. Luminance
uses accent; RGB channels use success, warning, and danger tokens. No pixel colors are embedded in
the component.

## States

The component is stateless. Luminance and RGB bins are caller supplied, finite values are clamped
to 0–1, and nonfinite values render as zero. The chart scales each data series to its own maximum
so the shape remains visible regardless of pixel count. Empty input renders an empty chart region.
Clipping indicators are independent booleans for shadows and highlights.

The empty chart reserves the same configurable plot height as a single-series chart (78 logical
pixels by default). This keeps the containing inspector layout stable while data is loading or
when no samples are available. Its summary remains in the image accessibility description.

## Conformance fixtures

The dedicated screenshot matrix renders four deterministic states at light, dark, and high
contrast themes, each at 1× and 2× scale. The luminance fixture has one nonuniform series; the
RGB fixture has three distinct nonuniform series; clipping adds both status indicators to the
luminance fixture; empty has no series. All fixtures use the default plot height and the same
window dimensions so panel geometry can be compared across states. The matrix checks every
declared screenshot case in `tests/conformance.json` against a named baseline. Accessibility
cases require an active platform accessibility tree; headless capture may report unsupported.

## Props and events

Builder props include bin data for luminance or red, green, and blue; a required accessible
name and summary; optional shadow/highlight clipping booleans; and configurable plot height.
There are no events. The host recomputes and replaces data when the image changes.

## Keyboard map

Not focusable. It has no keyboard actions because it is a read-only visualization.

## Pointer behaviour

Decorative and ignores pointer input. Hover inspection and bin selection are outside this component.

## Accessibility role and properties

Expose one image role with the caller-provided accessible name and a caller-provided textual
summary describing channels, distribution context, and clipping. Individual bars and decorative
legends are hidden from the accessibility tree. The application is responsible for keeping the
summary synchronized with the displayed data.

## Theme tokens used

`Theme.colors.surface`, `border`, `text`, `text_muted`, `accent`, `success`, `warning`, and
`danger`; `Theme.spacing` and `Theme.radii`; `Theme.borders.hairline`; and
`Theme.typography.caption`. Fixed chart and bin geometry must be exposed as builder dimensions or
documented defaults. No hard-coded colors are permitted.
The channel legend reserves 6.5 caption-size units so “Luminance” stays on one line in the
standard gallery width; hosts can give the component more width for longer localized labels.

## WAI-ARIA pattern reference

The chart is a static image, following the WAI-ARIA `img` role pattern. The accessible name and
description provide the useful textual equivalent; individual graphical bars are not separately
announced.

## Platform notes

The first preview has macOS screenshot coverage at light and dark 1×/2×. The dedicated matrix
extends this to all four states and the high-contrast theme. Active-platform accessibility
remains open. Callers should provide a concise summary on every platform.

## Open questions

- Should a future version support hover inspection of individual bins?
- Should scale-to-maximum remain the only normalization mode, or should a host-selectable linear
  count scale be added?
