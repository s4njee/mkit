# E9 contrast audit

Status: measured, awaiting maintainer review. No colours were changed.

The audit lives in `crates/mkit-core/src/contrast.rs` (`builtin_theme_token_pairs_meet_wcag_targets_or_are_known_failures`). It computes WCAG 2.x contrast ratios with the public `mkit_core::contrast::contrast_ratio` helper for 23 token pairs in each of the five built-in themes.

Targets:

- Text pairs: 4.5:1 (SC 1.4.3), or 7:1 (SC 1.4.6) in `HIGH_CONTRAST`.
- Non-text pairs (control boundaries, focus indicators): 3:1 (SC 1.4.11).

Text pairs: `text` and `text_muted` on `background`, `surface`, and `elevated_surface`; `accent_text` on `accent` and on `danger` (the destructive button); `accent` on `background` and `surface` (the link button); and `danger`, `success`, and `warning` on `background` and `surface` (inline error and status text). Non-text pairs: `focus` on `background`, `surface`, `elevated_surface`, and `accent`; and `border` on `background`, `surface`, and `elevated_surface`.

`disabled` is not audited, because WCAG exempts inactive components.

## Results

| Theme | Pass | Known failures | Closest passing text pair | Closest passing non-text pair |
| --- | --- | --- | --- | --- |
| `LIGHT` | 19 | 4 | `success` on `background`, 4.51 | `focus` on `background`, 6.79 |
| `DARK` | 19 | 4 | `accent` on `surface`, 5.74 | `focus` on `elevated_surface`, 6.64 |
| `HIGH_CONTRAST` | 19 | 4 | `success` on `background`, 15.50 | `focus` on `background`, 16.75 |
| `SHADCN_LIGHT` | 20 | 3 | `text_muted` on `elevated_surface`, 4.54 | `focus` on `accent`, 3.78 |
| `SHADCN_DARK` | 19 | 4 | `text_muted` on `elevated_surface`, 6.00 | `focus` on `elevated_surface`, 6.00 |

Totals: 96 pass and 19 are known failures out of 115 pairs. Every theme passes `text` and `text_muted` on all three surfaces.

## Known failures

The test's `KNOWN_FAILURES` list records each failing pair with the measured ratio, rounded to two decimals. The test fails if an unlisted pair drops below its target, if a listed pair's ratio changes, or if a listed pair starts passing. In each case, update the list and this report.

| Theme | Pair | Ratio | Target |
| --- | --- | --- | --- |
| `LIGHT` | `border` on `background` / `surface` / `elevated_surface` | 1.29 / 1.38 / 1.38 | 3 |
| `DARK` | `border` on `background` / `surface` / `elevated_surface` | 2.01 / 1.80 / 1.57 | 3 |
| `SHADCN_LIGHT` | `border` on `background` / `surface` / `elevated_surface` | 1.26 / 1.26 / 1.21 | 3 |
| `SHADCN_DARK` | `border` on `background` / `surface` / `elevated_surface` | 1.91 / 1.73 / 1.46 | 3 |
| `LIGHT` | `focus` on `accent` | 1.35 | 3 |
| `DARK` | `focus` on `accent` | 1.32 | 3 |
| `SHADCN_DARK` | `focus` on `accent` | 2.42 | 3 |
| `HIGH_CONTRAST` | `focus` on `accent` | 1.17 | 3 |
| `HIGH_CONTRAST` | `danger` on `background` / `surface`; `accent_text` on `danger` | 6.68 (each) | 7 |

The worst pairs are `HIGH_CONTRAST` `focus` on `accent` (1.17) and `SHADCN_LIGHT` `border` on `elevated_surface` (1.21).

## Proposed fixes (need maintainer review)

1. **Border.** `border` serves two jobs: decorative dividers, which WCAG exempts, and the only visible outline of inputs such as the text field and checkbox, which need 3:1. One option is to add a `border_strong` (or `control_border`) token for control outlines and keep `border` for dividers. The alternative is to darken or lighten `border` itself. These are the smallest same-hue values that reach 3:1 on all three surfaces:
   - `LIGHT`: `#d8dce3` to `#8e9095`
   - `DARK`: `#454b58` to `#717680`
   - `SHADCN_LIGHT`: `#e5e5e5` to `#919191`
   - `SHADCN_DARK`: `#404040` to `#6f6f6f`

   Changing `border` in place makes every divider much heavier. A new token changes the public API and every registry component that draws a control outline.
2. **Focus on accent.** The button's focus-visible style replaces its 1 px border with `focus`. On an accent-filled button the ring sits against the accent fill, but its outer edge still meets `background` at 6.6:1 or more, so SC 1.4.11 is arguably met. The stricter focus-appearance reading (SC 2.4.13, AAA) is not met. A colour change cannot fix this without breaking `focus` on the backgrounds, so the fix would change structure instead: draw the ring 2 px wide, or offset it by a 2 px `background` gap, in the shared focus styling.
3. **High-contrast danger.** Change `HIGH_CONTRAST.danger` from `#ff5555` to `#ff6b6b`. That measures 7.57:1 against the black background and against the black `accent_text`, which fixes all three pairs.

## `Default` for `Theme` (not decided)

"Dark-first default" is a maintainer decision, so this change does not add a `Default` impl. If one is wanted, it would be:

```rust
impl Default for Theme {
    fn default() -> Self {
        DARK
    }
}
```

With that impl, apps could call `cx.default_global::<Theme>()`. Today, `cx.global::<Theme>()` panics until a theme is installed. The impl would also make the choice of `DARK` over `SHADCN_DARK` part of the public API.

## Related notes

- `site/scripts/generate-theme.mjs` still parses `theme.rs` with regular expressions. It could switch to `Theme::to_json()`, whose output uses the same group and token names with hex colours. The site's `ShadowToken` type would need `x` and `spread` added. Because the site build is Node-only, the switch would need either a Rust step in the site build or checked-in JSON files kept current by a test. The site was not edited.
