# E7.12 — Disclosure and Accordion

The component contracts live in [`registry/disclosure/spec.md`](../registry/disclosure/spec.md) and
[`registry/accordion/spec.md`](../registry/accordion/spec.md). Both components expose a trigger
button with expanded state and a labelled, stable-ID panel. Their APIs are draft pending maintainer
review.

Disclosure provides one independently controlled region. Accordion provides shared single-open or
multiple-open behavior. Both support controlled and uncontrolled state and emit a typed
`ExpandedChanged` proposal. Enter and Space activate triggers. Panels are omitted while closed and
rebuilt from their content factories when opened. Motion is deferred until the shared E4.4 motion API
is available.

The property inspector's group rows are also collapsible. They share Enter/Space activation and the
button/expanded accessibility semantics while keeping inspector-owned state and depending only on
mkit-core and GPUI. Full component adoption remains open.

## Verification

GPUI entity tests cover keyboard toggling, controlled-state proposals, and disabled behavior. The
component harness screenshot and accessibility matrix remains pending unless the current harness
provides fixtures for these new crates. Maintainer review is required for API naming, keyboard and
accessibility contracts, and any visual baseline updates.
