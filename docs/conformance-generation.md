# Component conformance manifests

Generate a JSON manifest from the YAML front block in a component spec:

```sh
python3 scripts/generate_conformance.py registry/button/spec.md \
  --output registry/button/tests/conformance.json
```

Regenerate after changing the spec. CI can check committed output without modifying it:

```sh
python3 scripts/generate_conformance.py registry/button/spec.md \
  --output registry/button/tests/conformance.json --check
```

The generator needs Python 3 and PyYAML 6.x (`python3 -m pip install 'PyYAML>=6,<7'`). The
manifest contains one keyboard case for every declared key, one accessibility case per state, and
the full states × light/dark/high-contrast × 1×/2× screenshot matrix. Key cases include fixture
loading and machine-readable state, event, or focus assertions. The component adapter maps fixture
identifiers to real component fixtures and checks each generated assertion.
Accessibility property `when` values must name a declared state; a property
cannot be declared twice for the same state. Generation fails on either error
so a misspelled condition cannot silently omit a snapshot assertion.

Run the cases with an adapter command:

```sh
python3 scripts/run_conformance.py registry/button/tests/conformance.json \
  --adapter 'python3 registry/button/tests/adapter.py' --kind keyboard
```

Use `--kind keyboard`, `--kind accessibility`, or `--kind screenshot` for a
focused run; repeat `--kind` to combine them. `--case keyboard-01` selects one
generated case by ID. A filter matching no cases fails. Focused runs skip the
optional custom test command; an unfiltered run includes it. Passing a focused
run never substitutes for the full conformance run.

The button adapter runs the generated Enter and Space keyboard cases against a
real GPUI button fixture. It also checks that disabled and loading buttons do
not activate during those runs. The button accessibility cases currently return
pending because the headless test platform does not expose an active
accessibility tree; screenshot cases are not yet implemented by this adapter.

The adapter receives one JSON case on stdin and must write one JSON object to stdout. A passing
result is `{"passed":true,"actual":...}`, where `actual` is structured evidence the runner checks
against generated expectations. Keyboard evidence includes fields such as `state`, `event`, and
`focus_target` matching the case's `assert` mapping. Accessibility evidence uses `{"role":"switch",
"properties":{"checked":true}}`; every declared property must match. Screenshot evidence uses
`{"baseline":"component/state/theme-1x.png","matched":true}` and must name the expected baseline.
A bare `passed: true`, mismatched value, missing property, baseline mismatch, or `matched: false`
fails the case. Unsupported features use
`{"status":"unsupported","reason":"..."}`. Unsupported results fail the run as pending
gates; they do not count as passes. Accessibility cases require a real active platform accessibility
tree. Screenshot cases run only on macOS, where the pinned GPUI renderer supports image capture.
Custom tests are invoked after generated cases and must be specified as a shell-like command string.

This runner is an executable adapter contract, not a component implementation. The E1 harness can
dispatch keys and capture supported screenshots; component fixtures must still connect those
operations to their assertions, and inactive headless accessibility remains unsupported.

## Live smoke fixture

`examples/interaction/conformance/overlay-dismissal.spec.md` drives the E4
nested-overlay GPUI fixture. Its committed manifest is generated from the spec:

```sh
python3 scripts/generate_conformance.py examples/interaction/conformance/overlay-dismissal.spec.md \
  --output examples/interaction/conformance/conformance.json --check
python3 scripts/run_conformance.py examples/interaction/conformance/conformance.json \
  --adapter 'python3 examples/interaction/conformance/adapter.py' --kind keyboard
```

The two keyboard cases dispatch real GPUI keystrokes. On macOS,
`--kind screenshot --case screen-nested_open-light-1x` compares a Metal capture
with the committed baseline. The other screenshot cases and both accessibility
cases return unsupported pending; an unfiltered run therefore fails until
those capabilities and baselines are provided. The smoke fixture is an
integration check for the E5 machinery, not a registry component.
