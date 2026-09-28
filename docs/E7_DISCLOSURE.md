# E7.12 — Disclosure and Accordion

The component contracts live in [`registry/disclosure/spec.md`](../registry/disclosure/spec.md) and
[`registry/accordion/spec.md`](../registry/accordion/spec.md). Both components expose a trigger
button with expanded state and a labelled, stable-ID panel. Their APIs are draft pending maintainer
review.

Disclosure provides one independently controlled region. Accordion provides shared single-open or
multiple-open behavior. Both support controlled and uncontrolled state and emit a typed
`ExpandedChanged` proposal. Enter and Space activate triggers. Panels are omitted while closed and
rebuilt from their content factories when opened.

## Parts, motion, and the panel relationship

Disclosure's trigger and panel are exported as stateless `DisclosureTrigger` and `DisclosurePanel`
parts (draft API). The `Disclosure` entity renders through them. The `Disclosure` key context now
sits on the trigger rather than the whole region, so Enter and Space pressed on controls inside the
open panel no longer collapse it.

Optional motion uses the E4.4 tokens. `.motion(true)` fades the panel in over
`TransitionKind::Open` (`Theme.motion.normal_ms`) through `mkit_core::motion::transition_animation`.
The panel is present in layout, focus order, and the accessibility tree immediately; only opacity
animates. No animation element is created when `App::reduce_motion` is set or the duration token is
zero (high contrast). Collapse is immediate, and the chevron stays static because GPUI cannot rotate
text. Motion is off by default, so existing baselines are unaffected.

The `aria-controls` relation remains a documented limit of pinned GPUI 0.3.5. AccessKit has
`Node::set_controls`, but GPUI has no relation builder, keeps `GlobalElementId::accesskit_node_id`
crate-private, and lets `A11ySubtreeBuilder` mint IDs only for synthetic children. The trigger
cannot name the panel node. The relationship is conveyed structurally instead: the panel is the
trigger's next sibling, keeps a stable `disclosure-panel-{id}` element ID, and is named by the
trigger label.

## PropertyInspector adoption

PropertyInspector group rows now render `DisclosureTrigger` headers and `DisclosurePanel` row
containers while keeping inspector-owned state. The registry dependency on Disclosure is recorded as
a draft exception pending maintainer approval. See [property inspector notes](E8_PROPERTY_INSPECTOR.md).

## Verification

GPUI entity tests cover keyboard toggling, controlled-state proposals, disabled behavior, panel
content keys not toggling, the stateless trigger routing a rebound `Toggle` and clicks to its owner,
and motion rendering with and without reduced motion. A unit test covers the opt-in, reduced-motion,
and zero-token rules (`cargo test -p mkit-registry-disclosure`: 6 passed). The
screenshot matrix exists: `cargo test -p mkit-gallery --test e7_disclosure_matrix --locked` compares
Disclosure's three states (18 captures) and Accordion's four states (24 captures) across shadcn
light/dark/high-contrast at 1×/2×, and passed on 2026-09-27, and again unchanged after the parts refactor (see the
[E7 audit](E7_AUDIT.md#2026-09-27-e710e713-and-e716-drafts)). Native accessibility snapshots remain
pending because headless capture is unavailable in pinned GPUI (see
[E5 accessibility capture](E5_A11Y_CAPTURE.md)). Maintainer review is required for API naming,
keyboard and accessibility contracts, and the visual baselines.
