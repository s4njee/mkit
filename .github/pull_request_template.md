## Change

What changed, and which plan story does it address?

## Verification

- Commands run and results:
- Platforms checked:
- Checks not run and why:

## Harness evidence

- Keyboard or pointer scripts and observed results:
- Accessibility snapshots and any diffs:
- Screenshots or screenshot diff artifacts (states, themes, scales):
- Manual checks for behavior the harness cannot cover:

Use “not applicable” where the change has no UI behavior, and “not available yet” for planned harness checks that do not exist. Link artifacts when available.

## Human review

- Public API and naming decisions:
- Component spec, keyboard map, and accessibility contract:
- Book accuracy and example coverage:
- Visual baseline changes:

## Component definition of done (when a registry component changes)

Link the component spec and its conformance report. Use “not applicable” only
when the item genuinely does not apply; leave unmet requirements open.

- [ ] Maintainer approved the spec, API, keyboard map, and accessibility contract.
- [ ] Spec records states, events, controlled/uncontrolled mode, theme tokens,
      platform behavior, APG or platform pattern, differences, and open questions.
- [ ] Generated keyboard cases, per-state accessibility snapshots, and the
      state × light/dark/high-contrast × 1×/2× screenshot matrix pass on
      supported platforms; limitations and manual checks are linked above.
- [ ] Custom tests cover behavior outside the generated suite.
- [ ] Source uses theme tokens and documented sizes; permitted dependencies
      match the approved spec.
- [ ] Docs include three compiling examples, a keyboard table, and generated
      screenshots; the gallery shows every state and theme.
- [ ] `cargo mkit add`, the `mkit` crate, and GPUI Kit coexistence build.

See `docs/E5.4_DEFINITION_OF_DONE.md`.
