# Benchmark 09: Canvas drawing app

## Goal

Build a drawing app with a freehand pen and a rectangle tool, four colours, undo and redo, and
clear.

## Required features

- A toolbar above a white canvas. In an 800 x 600 window the canvas must cover at least the region
  from (200, 200) to (600, 500).
- **Pen:** pressing in the canvas and dragging records a stroke of every pointer position from the
  press to the release, in canvas coordinates (logical pixels from the canvas's top-left). Strokes
  are painted as connected 3-pixel lines. A stroke with fewer than two distinct points is dropped.
- **Rectangle:** dragging creates a rectangle between the press and release points, painted as a
  2-pixel outline. A rectangle with zero width or height is dropped.
- Four colours, in order: black, red, green, blue. New shapes use the current colour.
- Undo and redo cover adding a shape and clearing the canvas. Adding a shape empties the redo
  history.
- Shapes are drawn with GPUI's painting API (for example `canvas` and paths), not one element per
  point.
- Keyboard focus starts on the canvas.

## Keyboard

| Key | Command |
|---|---|
| `p` / `r` | Pen / rectangle tool |
| `1` to `4` | Black, red, green, blue |
| `secondary-z` | Undo |
| `secondary-shift-z` | Redo |
| `secondary-backspace` | Clear the canvas (one undo step) |

## Targets

| Name | Element |
|---|---|
| `canvas` | The drawing surface; its bounds define canvas coordinates |
| `tool-pen`, `tool-rect` | Tool buttons |
| `color-<n>` | Colour swatch `n` (0 black, 1 red, 2 green, 3 blue) |

## Snapshot

```json
{
  "tool": "pen",
  "color": "black",
  "shapes": [
    {"kind": "stroke", "color": "black", "points": 12,
     "bounds": {"x": 50.0, "y": 50.0, "width": 150.0, "height": 70.0}},
    {"kind": "rect", "color": "red", "points": 2,
     "bounds": {"x": 30.0, "y": 40.0, "width": 100.0, "height": 50.0}}
  ],
  "can_undo": true,
  "can_redo": false
}
```

`tool` is `"pen"` or `"rect"`. `points` is the number of recorded positions (2 for a rectangle).
`bounds` is the shape's bounding box in canvas coordinates.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot has the pen tool, black, no shapes, and nothing to undo or redo.
- **AC2** Dragging in the canvas through (50, 50), (100, 60), (150, 120), (200, 80) adds one black
  stroke with at least 4 points and bounds (50, 50, 150, 70), within 2 pixels.
- **AC3** `r 2` selects the rectangle tool and red; dragging from (30, 40) to (130, 90) adds a red
  rectangle with bounds (30, 40, 100, 50). Clicking `color-3` selects blue and `tool-pen` the pen.
- **AC4** After two shapes, `secondary-z` leaves one and enables redo; `secondary-shift-z` restores
  two; undo then a new shape disables redo.
- **AC5** `secondary-backspace` removes every shape and can be undone in one step.
- **AC6** (screenshot) The first frame is not blank, and a pen drag from (300, 300) to (500, 350)
  changes the frame.

<!-- maintainer-only -->

## Book coverage

Needs Part VI: `custom-rendering/canvas-primitives.md` (paths, quads, and `canvas()`) and
`custom-rendering/element-lifecycle.md` (bounds for converting window to canvas coordinates).
Part IV covers pointer down, move, and up, and actions for the shortcuts.

## Harness coverage and gaps

Stroke rendering is judged only by AC6's frame change and human review. The screenshot check
drags at window coordinates because the headless session cannot resolve debug selectors.
