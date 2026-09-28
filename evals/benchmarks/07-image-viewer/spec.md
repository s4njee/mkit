# Benchmark 07: Image viewer with zoom

## Goal

Build an image viewer for `fixture/image.png` that fits the image to the window, zooms in fixed
steps, and pans with the pointer and keyboard.

## Required features

- A viewport fills the window below a status line that shows the zoom as a percentage and the
  image size in pixels.
- On open the image is in **fit** mode: the largest zoom at which the whole image fits the
  viewport, centred. Fit mode follows viewport size changes.
- Zoom steps multiply or divide by 1.25 and stay between 0.1 and 8.0. Keyboard zoom keeps the
  point at the viewport centre fixed. Any zoom or pan leaves fit mode (**manual** mode).
- `pan` is the image's offset in logical pixels from its centred position. Dragging in the viewport
  moves the image by the drag distance. Arrow keys move the image 50 pixels in the arrow's
  direction (`right` increases `pan.x`, `down` increases `pan.y`). Returning to fit resets `pan`.
- Scrolling with Ctrl held zooms around the pointer: a positive vertical delta zooms in one step,
  a negative delta zooms out one step.
- If `image.png` is missing or cannot be decoded, show an error message instead of the image.
- Keyboard focus starts on the viewport.

## Keyboard

| Key | Command |
|---|---|
| `secondary-=` / `secondary--` | Zoom in / out one step |
| `secondary-0` | Fit |
| `secondary-1` | Actual size (zoom 1.0, centred) |
| arrow keys | Pan 50 pixels |

## Targets

| Name | Element |
|---|---|
| `viewport` | The area that shows the image |

## Snapshot

```json
{
  "image": {"width": 400, "height": 300},
  "error": null,
  "zoom": 1.5,
  "fit_zoom": 1.5,
  "mode": "fit",
  "pan": {"x": 0.0, "y": 0.0}
}
```

`image` is `null` and `error` is a message when the image could not be loaded. `fit_zoom` is the
zoom fit mode would use for the current viewport.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The snapshot reports a 400 x 300 image, no error, fit mode, `zoom` equal to `fit_zoom`
  (greater than 0), and zero pan.
- **AC2** `secondary-1` gives zoom 1.0; `secondary-=` gives 1.25; 20 more presses clamp at 8.0; 40
  presses of `secondary--` clamp at 0.1. Each leaves fit mode.
- **AC3** `secondary-1` then `secondary-0` returns to fit mode with `zoom` equal to `fit_zoom` and
  zero pan.
- **AC4** Dragging from the viewport centre by (40, -30) gives pan (40, -30); `right down` then
  gives (90, 20).
- **AC5** At actual size, a Ctrl-held scroll with delta +120 at the viewport centre zooms in to
  1.25, and delta -120 zooms back to 1.0.
- **AC6** With `image.png` removed from the fixture, the snapshot has `image` `null` and a non-empty
  `error`.
- **AC7** (screenshot) The first frame is not blank, and one zoom-in step changes the frame.

<!-- maintainer-only -->

## Book coverage

Needs Part III images (`elements/images-and-svg.md`) plus Part VI custom rendering: the worked
pan-and-zoom canvas (`custom-rendering/pan-and-zoom.md`) is the core pattern, and
`custom-rendering/canvas-primitives.md` covers painting an image into computed bounds. Part IV
covers pointer drag and actions. The mkit Viewport component is optional.

## Harness coverage and gaps

Whether the pointer stays anchored during Ctrl-scroll zoom is specified but only the zoom value is
asserted. Visual placement is judged by AC7 and human review of the saved frames.
