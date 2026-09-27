# E8 maintainer review handoff

The [E8 audit](E8_AUDIT.md) records implemented drafts, focused checks, and remaining evidence for every E8.1–E8.14 story. Registry entries remain `implementation_in_progress`; the functional code and visual baselines are ready for review, not release approval.

| Review | Concrete decision and evidence |
| --- | --- |
| Public API and component specs | Review the `spec.md` and generated contract for each component before stabilizing names, builder methods, typed events, controlled ownership, keyboard shortcuts, and accessibility roles. Pay particular attention to [Slider gesture lifecycle and SegmentedControl item tips](E8_LAIKA_ADOPTION.md), [Timeline](E8_TIMELINE_SPLIT.md), [NodeEditor](E8_NODE_EDITOR_SPLIT.md), and [ColourTools gamut behavior](E8_COLOUR_TOOLS.md). |
| Visual baselines | Review the dedicated state/theme/scale screenshots under `registry/<component>/tests/baselines/` and the copied previews in [the E8 book guide](../book/src/components/e8/README.md). Candidates were inspected before acceptance and compared by the harness; baseline approval remains with the maintainer. |
| Book accuracy | Review the compiling `{{#include}}` examples, behavior tables, and preview captions in the E8 book pages. `mdbook build` and `check_book.py` verify rendering and image provenance, but cannot judge whether the prose matches product intent. |
| Platform behavior | Verify active-platform accessibility output and native input: screen-reader names/states, IME/selection/clipboard in typed editors, trackpad gestures, dense Timeline/NodeEditor targets, and Laika's multi-photo undo/reset and keyboard flow. The headless harness does not provide a native accessibility tree. |
| Colour sampling and portability | Decide whether to add an OS-specific eyedropper after its permission and cancellation behavior is defined; this pinned GPUI backend currently has no supported sampler. The isolated Laika adoption branch uses a local absolute mkit path because mkit has no configured remote or published package; choose a distributable dependency before merging it. |

The review should record any requested API or visual changes in the relevant component spec first, then refresh affected examples, baselines, and book pages together. Keep `source_ready` and plan checkboxes open until the public contracts and required platform checks are accepted.
