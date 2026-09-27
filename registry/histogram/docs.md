# Histogram

Histogram is a read-only plot for image-value distributions. The app computes and owns the
underlying bins, updates them when the image changes, and provides the accessible text summary.

## Luminance and RGB data

The builder accepts either one luminance series or three RGB channel series. Values are normalized
per series to its own peak so low-contrast distributions remain visible. Inputs are clamped to the
unit interval; nonfinite values are treated as zero. Empty series produce an empty row.

Channel colors come from the active mkit theme: accent for luminance, danger for red, success for
green, and warning for blue. They are semantic theme colors and can vary with the host theme.

## Clipping indicators

The host supplies independent booleans for shadow and highlight clipping. The component shows a
short textual status row for active indicators. The host should include clipping context in its
accessible summary as well, since that summary is the complete textual equivalent of the plot.

## Accessibility

The chart exposes one image role with the label and description supplied by the application.
Individual bars are not focusable or announced. Keep the description concise and update it with the
same data revision used to update the plot.

## Keyboard and pointer

The chart does not accept keyboard or pointer input. Hover inspection and bin selection are not
part of the current component contract.

## Examples

Compiling usage examples and rendered screenshots are pending the E8 gallery/example wiring. See
the component source and spec for the current builder surface and state contract.
