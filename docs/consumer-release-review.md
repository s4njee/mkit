# Consumer sample release review · 2026-09-26

The [Aria2 Manager sample](../apps/aria2-sample/README.md) is a separate Cargo workspace using the public `mkit` crate through a path dependency. It renders mock data only.

| Gate | Evidence | Status |
| --- | --- | --- |
| Composed download grid | The sample uses mkit DataTable with rich cell and header renderers, text equivalents for every cell, selection, keyboard navigation, and row activation. Its 36 scene/theme/scale screenshots pass after inspection of the changed main and Add URL baselines. The default DataTable gallery retains all 30 existing baselines. | Implemented; public renderer, activation, and density API need maintainer review. |
| Editable forms | Add URL uses mkit TextField, Select, and Slider for save location, queue, connections, and split options. A focused test confirms those values reach the local download model. The RPC token uses `TextField::secure(true)`. | Implemented; secure-mode contract and six newly captured text-field baselines need maintainer review. The complete 36-case text-field matrix passes. |
| Modal keyboard and AX | The sample's Tab script enters the URL and Save to fields and stays within the dialog. Native macOS AX snapshots under [`apps/aria2-sample/tests/ax-snapshots/`](../apps/aria2-sample/tests/ax-snapshots/) show an AX table on the main screen, an AX dialog with form controls, and an `AXSecureTextField` subrole in Settings. The Add URL snapshot omits the background table. | Native snapshots captured; human screen-reader review remains open. |
| Headless accessibility | `mkit-harness::AccessibilitySnapshot::capture` returns `Inactive` in GPUI 0.3.5 headless windows; an interaction test asserts this result. The headless platform never activates the AccessKit adapter. | Blocked by upstream headless platform behavior. Generated per-state AX cases remain pending. |
| Source distribution | In a fresh external Cargo project, `cargo-mkit add button` vended source from a temporary release-candidate registry entry, `cargo check --offline` passed, and `cargo-mkit doctor` reported no warnings. | Candidate path verified; the production catalog remains `implementation_in_progress` and no crate is published. |
| GPUI Kit coexistence | `apps/coexistence` now renders an mkit button and a GPUI Kit button in the same window; its focused headless test passes. | Implemented. |

## Review requests

A maintainer should review the DataTable renderer and activation API, TextField secure mode, Dialog focus behavior, the component specs and book text, and the 12 changed sample baselines plus six new secure-field baselines. A person using VoiceOver should walk through row navigation, Add URL focus order and dismissal, and token entry before a release claim. Promotion of registry entries to `source_ready` and publication remain separate decisions after that review.

## Reproducing the native AX checks

Launch `apps/aria2-sample`, capture the live process with `swift scripts/capture_macos_ax.swift <pid>`, and inspect the main list, Add URL, and Settings. The checked-in snapshots were captured this way from the running app. Run `python3 apps/aria2-sample/tests/check_ax_snapshots.py` to assert the recorded roles, names, modal isolation, and secure subrole. The JSON files are evidence of the captured native tree; this script does not control a screen reader.
