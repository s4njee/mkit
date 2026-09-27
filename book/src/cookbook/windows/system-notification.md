# Request a tagged system notification

## What you'll build

A title and body sent to GPUI's system-notification route.

## Concept

A `SystemNotification` carries a tag, text, and optional actions. OS delivery depends on permissions and platform integration.

## Minimal compiling example

```rust
{{#include ../../../../examples/cookbook/src/lib.rs:recipe_system_notification}}
```

Run `cargo test -p mkit-example-cookbook --lib --locked`. `notification_recipe_records_the_tagged_local_notification` checks the request in the test platform. It does not prove the OS displayed a banner.

## How it works

`notify` builds a tagged notification and asks `App` to show it. This recipe does not create an in-app toast.

## Exercises

Send a second notification with the same tag and inspect replacement behavior on each target OS.

## API reference links

- [`gpui::SystemNotification`](../../appendices/api-inventory.md)
- [`gpui::App`](../../appendices/api-inventory.md)

