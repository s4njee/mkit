# E7.16 — Inline messages

Specs: [`registry/inline-alert/spec.md`](../registry/inline-alert/spec.md) and
[`registry/empty-state/spec.md`](../registry/empty-state/spec.md).

`InlineAlert` covers contextual info, success, warning, and error messages. It exposes status role
with polite priority for info/success and alert role with assertive priority for warning/error. An
optional action emits `ActionInvoked`; an optional dismiss control emits `DismissRequested`. The
uncontrolled dismissible alert hides itself before emitting; controlled mode waits for the owner to
apply `set_dismissed`.

`EmptyState` is a stateless `RenderOnce` builder for an app-provided icon, title, optional
description, and caller-owned action elements. It does not execute application operations or
re-emit action events. Both components take their colors, spacing, and type sizes from the mkit-core
Global `Theme`.

## Verification

Focused GPUI tests cover alert actions, controlled and uncontrolled dismissal, disabled dismissal,
per-instance IDs, and empty-state icon/description/action rendering. The macOS gallery matrix passed
all 54 declared screenshot cases: six alert states (including a keyboard-focused action) and three
empty-state states, each in light, dark, and high-contrast themes at 1× and 2×. Every capture was
inspected before accepting the E13.1 restyled baselines. The dark 2× alert warning and actionable empty-state images are copied to the
book with exact baseline bytes. Live-region announcement timing and active-platform accessibility
snapshots remain unverified; headless screenshots do not establish spoken output. Maintainers
should review public API, announcement urgency, and accessibility behavior on macOS.
