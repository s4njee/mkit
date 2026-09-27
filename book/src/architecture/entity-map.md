# Follow an entity's lifetime

## What you'll build

Run the entity and weak-handle example, then trace a strong handle's final drop through GPUI's entity map. The macOS harness shows the view's initial state.

![Entity example window with retained state and weak-handle controls](../images/state-entities.png)

## Concept

An [`Entity<T>`](../appendices/api-inventory.md) is a strong typed handle to state owned by GPUI. [`EntityMap`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L56-L89) stores values by `EntityId` and keeps separate atomic reference counts. A [`WeakEntity<T>`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L738-L777) does not retain the value; upgrading it can fail after release.

On the last strong handle drop, [`AnyEntity::drop`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L340-L353) queues the ID for cleanup. [`App::release_dropped_entities`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L1785-L1806) drains that queue during effect flushing, removes callbacks and window tracking, and invokes release listeners. Dropping a handle is not a promise that all cleanup callbacks run synchronously before `drop` returns.

## Minimal compiling example

```rust
{{#include ../../../examples/state_entities/src/lib.rs:state_entities_view}}
```

Run `cargo test -p mkit-example-state-entities --locked`. Its tests check retained view updates and that a weak handle stops upgrading after the last strong handle drops. The macOS screenshot compares the initial view. These tests do not directly assert the internal `EntityMap` queue or a strong-reference cycle; those are source-audited here.

## How it works

The example holds an entity for retained state and uses a weak handle where long-term ownership is not needed. The map allocates an ID and stores the value separately from its handle count. Cloning a strong handle raises that count; a weak handle does not. The last strong drop places the ID in `dropped_entity_ids`; a later app flush removes the value. The pinned source warns that [strong entity cycles can keep every member alive](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L891-L911).

## Exercises

Extend the weak-handle test with a release observer and assert when it runs after effect flushing. Use weak back-references when two retained entities need to refer to each other, and test that both can be released.

## API reference links

- [`Entity`, `WeakEntity`, `EntityId`, and `App`](../appendices/api-inventory.md) — public handles and app context. `EntityMap` is an internal type linked to pinned source above.
