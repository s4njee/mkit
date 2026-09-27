# E7.18 — Tag input

The draft `TagInput` entity collects labels, recipients, and filters. It exposes uncontrolled and controlled modes; controlled additions and removals emit the full proposed tag list, and the owner accepts or corrects proposals with `set_tags`.

## Editing and keyboard behavior

Enter adds the active suggestion or the trimmed query. Comma adds the trimmed query without retaining the separator. A validator may attach a per-tag message while still allowing the value to be added. Duplicate and maximum-count candidates are rejected with a typed `TagRejected` event and polite status text.

With an empty editor, Backspace selects the last tag and a second press removes it. ArrowLeft/ArrowRight move the virtual tag selection while focus remains in the TextField editor; Delete removes the selected tag. Escape clears tag and suggestion selection. Text editing delegates to `mkit-registry-text_field`, a documented draft dependency exception pending maintainer approval.

Optional suggestions filter with the same active-option and disabled-option rules as Combobox. Direct Combobox composition is not used: Combobox owns a single editor and commits by replacing one value, while TagInput appends to a list and needs one shared editor. Suggestion rows remain visible when disabled, are skipped by ArrowUp/ArrowDown, and cannot be activated by pointer.

## Accessibility and theme

The root is a named group. Tags are list items with optional invalid descriptions and named remove buttons. The editor inherits TextField textbox semantics. Suggestions use a listbox and active option; validation and maximum-count messages are status content. TagInput reads all colors, spacing, control sizes, borders, radii, and typography from the mkit-core Global theme.

The component spec is in `registry/tag-input/spec.md`; generated cases are in `registry/tag-input/tests/conformance.json`. API and accessibility semantics are drafts pending maintainer review.

## Verification

Focused GPUI tests cover Enter and comma commits, controlled proposals, two-step Backspace removal, per-tag validation, max-count rejection, active suggestion selection with disabled-option skipping, and disabled suggestion pointer rejection. The gallery matrix covers all eight declared states across light, dark, and high-contrast themes at 1× and 2× (48 captures); update and compare runs pass. Inspected empty, query, selected-tag, validation-error, max-count, and suggestion captures. The validation message is visible inside the invalid chip. Platform accessibility snapshots and native IME composition validation remain pending.
