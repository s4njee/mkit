# E4 core foundation audit — 2026-09-24

E4.1, E4.2, E4.3, and E4.6 meet their plan checkboxes. E4.3 now has live
headless outside-pointer and rebindable Escape tests, plus a nested overlay
screenshot. Deferred priority still does not provide modal input blocking.
E4.4 has a macOS system preference listener and a Windows system animation
listener; Linux has no bridge, and native OS preference changes remain
unverified. E4.5 sets AccessKit live-region semantics through GPUI and has a
stateful status example, but no live screen-reader announcement has been
verified. Keep the E4.4 and E4.5 plan boxes open.

Story contracts and limits are in [E4.1](E4.1_NOTES.md),
[E4.2](E4.2_NOTES.md), [E4.3](E4.3_NOTES.md), [E4.4](E4.4_NOTES.md),
[E4.5](E4.5_NOTES.md), and [E4.6](E4.6_NOTES.md). The book's overlay,
animation, and accessibility chapters now describe the E4 helpers and link the
new rendered examples.

## Checks run

- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed, including 20 `mkit-core` tests,
  11 interaction tests, six state example tests, and macOS screenshot
  comparisons.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `python3 scripts/check_book.py` — passed: 92 pages, 13 example crates.
- `mdbook build book` and `python3 scripts/check_book.py --skip-cargo
  --built-html` — passed.
- `uvx --from codespell==2.4.3 codespell --quiet-level 2 book/src` — passed.
- `python3 scripts/generate_gpui_inventory.py --check` and
  `python3 -m unittest discover -s scripts -p 'test_*.py'` — passed.
- `cargo build --manifest-path apps/coexistence/Cargo.toml --locked` — passed.

The controlled-state example's light, dark, and high-contrast 640 × 400
scale-1 baselines and light scale-2 baseline were inspected. The nested overlay
light baseline was compared at 640 × 480 scale 1; dark, high-contrast, and
scale-2 previews were inspected but are not checked-in baselines. The status
example's initial and updated light baselines were inspected at 640 × 400
scale 1. These captures use macOS Metal headless rendering. Focus movement and
roving actions have rendered headless keyboard tests; theme switching redraws
two open headless windows. The stock GPUI test window does not activate an
accessibility tree, so no live-region snapshot or spoken announcement was
captured. Native pointer/Escape behavior and OS preference changes still need
desktop checks.

Windows GNU/MSVC target checks stopped inside GPUI Pre's resource build before
reaching mkit-core. GNU lacks `windres` on this host. Homebrew LLVM supplies
`llvm-rc` for MSVC, but GPUI Pre's resource script then fails to resolve its
relative manifest include. Windows binding signatures were inspected but the
full adapter has not been cross-compiled here. A standalone MSVC-target probe
with the pinned Windows and futures crates compiled the listener's reader,
event callback, and unregistration API. The Linux mkit-core target check
passed; a later state-example target check stopped in Wayland's build
dependency because this host lacks `x86_64-linux-gnu-gcc`.

The public theme/state/focus/overlay/accessibility API, keyboard contract,
component-facing specs, visual baselines, and Part II book chapter need maintainer review under
`AGENTS.md` before release claims are made.
