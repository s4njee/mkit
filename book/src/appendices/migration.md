# Migration notes

## Baseline: `gpui-pre` 0.3.5

This book currently has one pinned GPUI snapshot: `gpui-pre` 0.3.5 at Zed revision [`d89e9c2124b2786a390c7a451c7488601b4da2e1`](https://github.com/zed-industries/zed/tree/d89e9c2124b2786a390c7a451c7488601b4da2e1). There is no earlier book snapshot to migrate from, so this entry records the starting point rather than a list of API changes.

The workspace calls the package `gpui_pre` in Rust. GPUI Kit 0.6.4 uses the same `gpui-pre` package through its local `gpui` dependency name. The [versions appendix](versions.md) explains why the package identity matters.

For each future pin change, add a dated section with the old and new package versions and commits, changed inventory items, example and component edits, affected platforms, and the exact build, test, coexistence and book commands that passed. Keep an unresolved item explicit when an OS target or feature profile cannot be checked. The [inventory](api-inventory.md) records the current extract limits; a missing target label does not prove an API is unavailable.
