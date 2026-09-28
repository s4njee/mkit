# Property inspector disclosure adoption

Property inspector groups are collapsible sections built from the E7.12 Disclosure parts. Each
group header is a `DisclosureTrigger`, and each expanded group's rows are children of a
zero-padding `DisclosurePanel` (`group-panel-{group id}`). Headers therefore share Disclosure's
button role, expanded state, `Disclosure` key context, and rebindable `disclosure::Toggle` action.
Collapsed property rows are omitted from rendering and from arrow-key navigation.

Group expansion remains inspector-owned because the group schema differs from the generic
Disclosure/Accordion entity API. The stateless parts emit no `ExpandedChanged` event; the
inspector toggles its own `PropertyGroup::expanded` flag. `default_key_bindings()` now returns six
bindings (the four inspector bindings plus Disclosure's Enter/Space). While a header has focus,
its deeper `Disclosure` context binding for Space outranks the root's `ToggleBoolean`.

The crate now depends on `mkit-registry-disclosure`. That registry-to-registry dependency is
declared in `registry/registry.json` (`component_dependencies: ["disclosure"]`), in the package
manifest, and in the `mkit` feature `property-inspector = ["mkit-mirror", "disclosure"]`. It is a
draft exception pending maintainer approval, like DatePicker→Calendar and SearchField→TextField.

## Evidence

`cargo test -p mkit-registry-property-inspector` passes 13 tests, including header Enter/Space
through the Disclosure action, a disabled inspector's inert headers, and the combined bindings.
`E8_SNAPSHOT_ONLY=e8-property-inspector cargo test -p mkit-example-e8-components --test
screenshots` matches all 30 existing inspector baselines unchanged. The header keeps the
inspector's elevated surface and normal text color through `Styled` overrides, including in the
disabled state. The accessibility tree changes in two ways that remain unverified natively:
disabled headers now expose disabled state, and expanded rows are nested in a labelled group. See
[`registry/property-inspector/spec.md`](../registry/property-inspector/spec.md).
