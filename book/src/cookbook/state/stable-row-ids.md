# Update a row by stable ID

## What you'll build

A row title update that does not depend on its current list position.

## Concept

An ID identifies the logical row even when the list is reordered. An index identifies only a position.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_stable_ids}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `stable_id_updates_do_not_depend_on_list_position` checks a successful update and a missing ID.

## How it works

`update_row_title` searches for the requested ID, updates its title, and returns false when no row matches. It is a pure Rust helper used in a GPUI app; it does not render a list.

## Exercises

Reorder the row vector, update the same ID, and assert that the same logical row changes.

## API reference links

This pure Rust helper calls no GPUI public API directly. See [the retained-state chapter](../../state/entities.md) when storing rows in an entity.
