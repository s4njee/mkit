# E5 progress audit — 2026-09-24

E5.1 now has a Markdown spec template with a YAML front block for states,
fixture IDs, key dispatch and expected state/event/focus, and accessibility
role/properties. E5.2 maps every planned E7/E8 component to an APG pattern,
platform convention, or justified custom interaction. Both are ready for
maintainer review.

E5.3 generates a case manifest with one keyboard case per declared key, one
accessibility case per state, and a state × three themes × two scales screenshot
matrix. Its runner invokes a component adapter and compares returned state,
event, focus, accessibility, and screenshot evidence against the generated
expectations. An unsupported result fails the run as pending. The runner also
invokes component-specific tests when configured. Both pilots now pass their
complete generated keyboard maps (19 cases total). One number-field screenshot
case also passes on macOS; the accessibility and full screenshot matrices have
not passed. The pinned headless window still has
inactive accessibility. The E5.3 full generation checkbox therefore remains
open.

E5.4 has a [definition of done](E5.4_DEFINITION_OF_DONE.md), pull request
checklist, and a CI guard that checks the checklist and available machine gates
stay wired. CI checks spec validity, generated-case freshness, the overlay
smoke fixture, and both complete pilot keyboard maps. Full component
conformance, gallery coverage, and installation checks remain unavailable;
E5.4 stays open.

E5.5 has [combobox](../registry/combobox/spec.md) and [scrubbable number
field](../registry/scrubbable-number-field/spec.md) specs, generated manifests,
draft Entity implementations, and real GPUI keyboard adapters. The number
field passes all 10 generated keyboard cases, including native draft entry,
Enter commit, and Escape rollback. It also has a pointer scrubbing draft with
focused GPUI tests. The combobox passes all 9 generated keyboard cases,
including Tab and Shift+Tab focus traversal; its editable query uses native
GPUI input handling. Both drafts are mirrored into `mkit` and cataloged as
`implementation_in_progress`, not installable. The book now has two draft
component chapters with compiling constructor examples, and the gallery opens
both pilots with theme controls. A macOS gallery capture shows readable values
for both components in the default light state. Rendered book examples, complete pointer
behavior, active accessibility capture, the remaining screenshot matrices,
full gallery state coverage, and installation remain open. Partial
timing and review notes are in the [pilot log](E5_PILOT_LOG.md); token usage
and maintainer rework have not been measured, so §9 estimates remain unchanged.

## Checks

- `python3 scripts/check_component_specs.py` — passed for two pilot specs and
  their committed manifests (9 + 10 keyboard cases, 8 + 6 accessibility cases,
  48 + 36 screenshot cases). The generator now checks unknown state references
  and duplicate accessibility properties within a state.
- `python3 -m unittest discover -s scripts -p 'test_*.py'` — passed, 51 tests.
- The generator's `--check` path is enforced by CI for each registry spec.
- `python3 scripts/check_registry.py`, `python3 scripts/check_e5_dod.py`, and
  `python3 scripts/sync_mkit_components.py --check` — passed for both draft
  sources and the available CI gates.
- The overlay smoke manifest's `--check` passed (two keyboard, two
  accessibility, 12 screenshot cases). The runner passed two real keyboard
  cases and one real macOS screenshot comparison. Its accessibility-only run
  exited 1 with two explicit pending cases, as designed.
- The complete number-field generated keyboard run passed 10/10 cases; the
  complete combobox run passed 9/9. CI runs both unfiltered keyboard maps.
  Pilot accessibility cases remain pending because headless AccessKit is
  inactive. The number-field idle/light/1× screenshot case passed against a
  Metal baseline with readable value glyphs; the other screenshot cases remain
  pending. CI runs this focused case on macOS. The gallery's default light/1×
  screenshot test also passed with visible combobox and number-field text.
- `cargo fmt --all -- --check`, `cargo build --workspace --locked`,
  `cargo test --workspace --locked`, and `cargo clippy --workspace
  --all-targets --locked -- -D warnings` — passed after the latest pilot
  input, pointer, and screenshot edits.
- `python3 scripts/check_book.py`, `mdbook build book`, and
  `python3 scripts/check_book.py --skip-cargo --built-html` — passed for 94 pages
  and 14 example crates.
- `cargo build --manifest-path apps/coexistence/Cargo.toml --locked` — passed.
- CI YAML parsed successfully. `uvx --from codespell==2.4.3 codespell
  --quiet-level 2` passed on the E5 docs, both pilot specs, and registry README.

The component names, public props/events, keyboard and accessibility contracts,
theme tokens, pattern mapping, and eventual visual baselines require maintainer
review under `AGENTS.md`. Neither pilot spec has received maintainer approval.
Implementation can proceed as a draft, but the public contracts cannot be
considered final until that review.

## Accessibility snapshot blocker

The pinned `gpui-pre 0.3.5` `HeadlessAppContext` constructs its test platform
internally. That platform's test window does not implement the accessibility
initialization or tree-update hooks, so it never activates the AccessKit tree.
The existing harness regression test confirms the inactive result. A real
per-state snapshot requires a GPUI test-platform hook or local GPUI patch that
invokes the activation callback and records actual `TreeUpdate`s. That would
test GPUI tree generation; native screen-reader behavior would still need a
desktop check. Generated accessibility cases must remain pending until such a
hook exists.

Update: `mkit_harness::AccessibilitySession` now captures the real
`TreeUpdate` headlessly on macOS through a harness-owned platform window and
GPUI's public `VisualTestAppContext::new`, without a GPUI patch. `button`,
`checkbox`, and `slider` accessibility cases now run against reviewed-pending
text baselines. See `docs/E5_A11Y_CAPTURE.md` for the approach and remaining
gaps.

## Live generator smoke run

The [overlay fixture](../examples/interaction/conformance/overlay-dismissal.spec.md)
is an integration exercise for E5.3, separate from the product registry. Its
manifest is generated and checked from the spec. Two keyboard cases dispatch
real GPUI keystrokes and pass. One nested-open light 1× screenshot case compares
a macOS Metal capture with a committed baseline and passes. Both accessibility
cases and the other 11 screenshot cases explicitly report pending; an
unfiltered run fails. CI now runs the generated keyboard cases on every runner
and the focused screenshot case on macOS. This proves the generator-to-adapter
route for supported checks without treating the full conformance matrix as
complete.

The E5.5 dependency E6.1 now has a repository-served, release-versioned
[registry catalog](../registry/registry.json), schema, component-dependency
graph, and validator. Both pilots are `implementation_in_progress`; neither
is installable. Their registry sources are mirrored into the publishable
`mkit` crate by a checked sync script; `cargo package -p mkit --list
--allow-dirty --locked` includes both generated copies. E6.2 is still
required for the full pilot pipeline.
