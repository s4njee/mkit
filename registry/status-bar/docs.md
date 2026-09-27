# StatusBar

Place concise status information and quick controls along the bottom edge of an app window.
`StatusBar` has ordered leading and trailing regions with status text, supplied buttons, and progress
items. Text updates announce politely; controls use their ordinary button focus and activation
behavior.

Give each item a collapse priority. At narrow widths the bar removes `Low`, `Normal`, then `High`
items while preserving `Never` items. Supply `available_width` in logical pixels so the builder can
apply the priorities deterministically. Use `estimated_width` for custom buttons or other elements;
the default widths are approximations. There is no overflow menu in this draft. If even `Never`
items do not fit, they are clipped.

The builder does not own status state or actions. Rebuild it with updated text/progress and let the
supplied buttons handle their own events. See the checked spec in `spec.md` and review notes in
`docs/E7_STATUS_BAR.md` for keyboard, accessibility, theme, and width contracts.
