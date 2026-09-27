# Property inspector disclosure adoption

Property inspector groups are collapsible sections. Their headers expose button role and expanded
state, and collapsed property rows are omitted from keyboard navigation. They follow the same
Enter/Space interaction contract as E7 disclosure while keeping the registry crate dependency
limited to mkit-core and GPUI. The inspector currently owns the group expansion state because its
group schema is distinct from the generic accordion API.

Group expansion remains inspector-owned, so E7.12's shared typed event is not used here. GPUI tests
cover pointer and Space activation and verify collapsed rows disappear. The harness accessibility
snapshot for collapsed panels remains pending. See
[`registry/property-inspector/spec.md`](../registry/property-inspector/spec.md).
