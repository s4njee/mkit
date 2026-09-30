# Maintainer review queue

Start here for everything that needs maintainer judgment under [`AGENTS.md`](../AGENTS.md). Each
section links to the detailed evidence. Agents keep this file current; the maintainer records
outcomes in the review log.

All 66 registry entries are `implementation_in_progress`. A component can move to `source_ready`
only after its spec, API and keyboard/accessibility contract are approved and its required
platform checks are done ([E5.4 definition of done](E5.4_DEFINITION_OF_DONE.md)).

## Review log

| Date | Scope | Outcome | What it unblocks | Still pending |
|---|---|---|---|---|
| 2026-09-27 | Visual appearance of all 66 components | Maintainer inspected the components visually and cleared them for building out the docs site | Using the current previews and screenshots on the docs site | Spec, API, keyboard and accessibility approval; formal acceptance of the harness baselines; book accuracy; platform checks |

## 1. Quick decisions

Small, specific decisions that unblock agent work. Record the answer in the linked spec or note,
then in the review log.

| Decision | Where it is described | Blocks |
|---|---|---|
| **M5 (Parity & Reach) and E13:** approve the proposed milestone and stories; charts (E13.5) would bring a §2 out-of-scope item into the plan. E13.1–E13.2 visual parity already started at maintainer request | [plan.md E13 and §8](../plan.md#e13--parity--reach-post-10) | M5 |
| **E13.1 everyday visual parity (done, needs confirmation):** 28 more components restyled (date & time, structure, content & input, lists & layout); about 720 component baselines updated or added after inspection, plus book images and knock-ons (E7 collections/layout scenes, stepper compositions, property inspector book images and E8 inspect scene, all 36 Aria2 scenes). New keyboard-focused states: calendar, disclosure, accordion, inline alert, search field, tag input, file drop zone, file field, scroll area, reorderable list. Decide: **DataTable `row_divider` now defaults to `true`** (a default change); calendar chevrons became pointer-clickable (month navigation) with no accessible names; no "today" marker; calendar card border shows inside the date picker's popover border (double outline); date picker's width floor and button overlay work around TextField having no trailing inset; time segments want a monospaced-digit font token; MenuBar shows stacked mnemonic hints ("(f)") above labels; disclosure chevron flips while the preview's doesn't; stepper labels dropped their "1." prefix; TagInput hides TextField's own border; scroll-area thumbs are indicators only (not draggable); split-pane grip has no opt-out; form rows use 24px gaps vs the web's 16px | `registry/<name>/spec.md` Theme sections | Everyday components |
| **E13.2 pro-app split:** approve slices P1–P4; decide shadcn control heights vs today's denser pro heights (E9.1); whether handles share one new size token; whether histogram/curve styling waits for charts; whether Laika baselines move per slice | [E13.2 split](E13.2_SPLIT.md) | E13.2 |
| Make `mkit_core::CivilDate` a public type (timezone-free Gregorian date) instead of depending on `jiff` or `chrono` | [E7_CALENDAR](E7_CALENDAR.md), [`calendar` spec](../registry/calendar/spec.md) | Calendar, DatePicker, DateTimeField |
| Allow registry components to depend on other registry components: DatePicker → Calendar and TextField | [E7_DATE_PICKER](E7_DATE_PICKER.md) | DatePicker |
| Same, for DateTimeField → DatePicker and TimeField | [E7_TIME_FIELDS](E7_TIME_FIELDS.md) | DateTimeField |
| Same, for SearchField → TextField, plus the new TextField leading-inset builder | [E7_SEARCH_FIELD](E7_SEARCH_FIELD.md) | SearchField, TextField API |
| Cmd+F / Ctrl+F as SearchField's default focus shortcut | [E7_SEARCH_FIELD](E7_SEARCH_FIELD.md) | SearchField |
| TagInput suggestions built directly, not on Combobox | [E7_TAG_INPUT](E7_TAG_INPUT.md) | TagInput |
| PropertyInspector → Disclosure dependency; new stateless `DisclosureTrigger`/`DisclosurePanel` parts (callers can override styling); inspector `default_key_bindings()` now returns six bindings (Disclosure's Enter/Space added; Space on a header toggles the group); Disclosure key context moved to the trigger; opacity-only reveal for `.motion(true)`; inspector headers report disabled and expanded rows sit in a labelled group; `aria-controls` not expressible in pinned GPUI | [E7_DISCLOSURE](E7_DISCLOSURE.md), [E8_PROPERTY_INSPECTOR](E8_PROPERTY_INSPECTOR.md) | Disclosure, Accordion, PropertyInspector |
| **Text fields, select/combobox, menus and overlays restyle (E7.3–E7.6), done at maintainer request:** TextField, TextArea, Select, Combobox, MultiSelect, DropdownMenu, ContextMenu, Popover, Dialog, Sheet and Toast follow the web preview's shadcn look (vector chevrons, checks and submenu arrows; popover surfaces with shadows; 14px field text). 78 text, 90 select/combobox, 24 menu and 72 overlay baselines plus 15 book images were refreshed after inspection, along with knock-on baselines for DatePicker, DateTimeField, SearchField, TagInput (all embed TextField), the E7 gallery scenes, the stepper-in-dialog/sheet compositions, the main gallery and all 36 Aria2 sample scenes; please confirm. Decide: MenuItem has no label/separator/icon/destructive options; menu fixtures use a plain box trigger; Dialog has no close X or footer, Toast no action/icon, Sheet is bottom-docked (preview shows a side panel); tooltip popup isn't capturable headless so its restyle is unverified; combobox list now shrinks to fit up to eight rows and the popup flip check counts padding and border; multi-select chips have no remove button; field descriptions aren't shown on screen; SearchField's magnifier icon overlaps the field's left edge (it did before too); the Calendar grid inside DatePicker still uses the old boxed-cell look | `registry/{text-field,text-area,select,combobox,multi-select,dropdown-menu,context-menu,tooltip,popover,dialog,sheet,toast}/spec.md` | Everyday components |
| **Buttons and form controls restyle (E7.1, E7.2), done at maintainer request:** Button, IconButton, ToggleButton, ToggleGroup, Checkbox, RadioGroup, Switch, Slider and Progress follow the web preview's shadcn look (vector check marks, 32×18 switch, 6px slider track, 8px progress track). 60 + 132 component baselines, 9 book images, the E7 gallery scenes (main, navigation, overlays), 30 StatusBar baselines (it contains Buttons) and `gallery-light-1x.png` (which also picks up the earlier ScrubbableNumberField change) were refreshed after inspection; please confirm. Decide: progress track uses accent at 20% (as the web preview) rather than muted; loading Button looks like disabled (no spinner slot); outline toggle and borderless group variants have no API; toggle-button spec's Props section wrongly lists variant/size; focus rings aren't captured in any screenshot state; high-contrast slider/progress tracks are outlines with a thin fill | `registry/{button,icon-button,toggle-button,toggle-group,checkbox,radio-group,switch,slider,progress}/spec.md` | Buttons, form controls |
| **Navigation restyle (E7.7), done at maintainer request:** Tabs, SegmentedControl, Sidebar and Breadcrumbs now follow the web preview's shadcn look. 72 tabs/segmented, 24 sidebar/breadcrumbs, 4 gallery navigation baselines and 4 book images were refreshed after inspection; please confirm. Decide: light-theme unselected tab labels are 4.36:1 (just under AA, same as shadcn); sidebar current row uses a second mix step so it differs from the pane (the site uses one colour for both); proposals for segmented-item icons, a sidebar group label, sidebar item icons and breadcrumb "…" elision (no API added); sidebar spec/book say `SelectionChanged` but code emits `ValueChanged`; focus rings for segmented control, sidebar and breadcrumbs aren't captured in any screenshot state | `registry/{tabs,segmented-control,sidebar,breadcrumbs}/spec.md` | Navigation components |
| **Baselines waiting (tests fail until accepted):** KeyHint reuse changes shortcut text in CommandPalette (12 cases, `⌘⇧S` → `⇧⌘S`), ShortcutEditor (36 cases, `cmd-o` → `⌘O`), 4 shortcut-editor book images, the byte-copied `e8-shortcut-editor-conflict-dark-2x.png` and `e8-command-palette-filtered-{dark,light}-2x.png`, and 4 `apps/gallery/snapshots/e8-command-shadcn-*`. The four `book/src/images/e8-timeline-t3-t4-*` images also predate the spec'd keyframe marker band (the timeline matrix baselines already show it). Accept by rerunning those tests with `UPDATE_SNAPSHOTS=1` after inspecting. Also decide: KeyHint dependency for the four consumers, macOS modifier order, parse grammar, text vs glyph key names (`↩`, `⌫`), `aria_keyshortcuts`, MenuBar adoption | [E7_DISPLAY_PRIMITIVES](E7_DISPLAY_PRIMITIVES.md), [E7_AUDIT](E7_AUDIT.md) | CommandPalette, ShortcutEditor, E8 gallery tests |
| Toolbar overflow from an explicit `visible_capacity` instead of measured width | [E7_TOOLBAR](E7_TOOLBAR.md) | Toolbar |
| StatusBar `available_width` estimation and omission vs overflow | [E7_STATUS_BAR](E7_STATUS_BAR.md) | StatusBar |
| Stepper navigation and synchronous validation | [E7_STEPPER](E7_STEPPER.md) | Stepper |
| Stepper in Dialog and Sheet: give Sheet Dialog's rendered-order Tab (its focus stops are fixed at creation, so the conditional Back button is skipped)? Accept that Shift+Tab from the Dialog surface doesn't wrap? Add a focus ring to Stepper buttons (changes accepted baselines)? Show the scene in the gallery? Review 12 new screenshots in `examples/e7_compositions/tests/baselines/` and `book/src/images/e7-stepper-in-{dialog,sheet}.png` | [E7_STEPPER](E7_STEPPER.md), [E7_AUDIT](E7_AUDIT.md) | Stepper, Dialog, Sheet |
| File dialog fallback behaviour | [E7_FILE_INPUTS](E7_FILE_INPUTS.md) | FileDropZone, FileField |
| Non-virtualized ReorderableList for now; later sharing with VirtualList and LayerPanel | [E7_REORDERABLE_LIST](E7_REORDERABLE_LIST.md) | ReorderableList |
| Removing SegmentedControl `allow_empty`; Separator's splitter role; overlay placement and focus contracts; controlled text buffer contract | [E7_AUDIT](E7_AUDIT.md) (E7.3, E7.6, E7.7, E7.9 rows) | Several E7.1–E7.9 components |
| Headless accessibility capture: accept the harness-owned platform approach (public GPUI API, macOS only, may break on GPUI upgrades) instead of waiting for an upstream hook; approve the snapshot format and ARIA naming (including the `disabled` alias); make snapshot evidence mandatory in `run_conformance.py`; then generate snapshots for the other 63 components. Review the first 12 in `registry/{button,checkbox,slider}/tests/baselines/a11y/` | [E5_A11Y_CAPTURE](E5_A11Y_CAPTURE.md) | Every component's accessibility evidence |
| Findings from the first captures: a disabled or loading Button still exposes the Focus action (`tab_index(-1)`); range-slider thumbs report 0–100 instead of bounds limited by the other thumb | [E5_A11Y_CAPTURE](E5_A11Y_CAPTURE.md), `registry/button/spec.md`, `registry/slider/spec.md` | Button, Slider |
| Eyedropper: add an OS-specific sampler, or keep "unavailable" | [E8_REVIEW_HANDOFF](E8_REVIEW_HANDOFF.md) | ColourTools |
| How Laika depends on mkit (git, crates.io, or vendored source) | [E8_LAIKA_ADOPTION](E8_LAIKA_ADOPTION.md) | E8.14 merge. The spike is committed on Laika branch `codex/e8-mkit-adoption` |
| Book evaluations (E3): approve the 10 benchmark specs (app 6 has no IME/selection so M1 doesn't depend on Part V); choose the model and agent and whether the context is Parts I–IV or the whole book; is the M1 gate pooled ≥90% across apps 1–6 or ≥90% per app; budget (estimated 1–5M tokens per attempt, 20–90M for 6 apps × 3 attempts; suggest 1 attempt per app first); run agents with a clean profile so global `CLAUDE.md` files don't leak in | [evals/README](../evals/README.md), `evals/benchmarks/` | E3.2, E3.3, M1 |
| Book URL: currently deployed at `/GPUI/`, plan says `mk7s.dev/mkit` | [plan.md open question 4](../plan.md#12-open-questions) | E12.1, E12.2 |
| Contrast fixes: 19 of 115 token pairs miss WCAG targets (borders on surfaces, focus ring on accent, high-contrast `danger`). Apply the proposed colours, or add a `border_strong` token and a thicker/offset focus ring? | [E9_CONTRAST](E9_CONTRAST.md) | E9.1, conformance baselines (any colour change refreshes screenshots) |
| Theme files: allow partial files that extend a built-in theme? Keep `Theme::name` as `&'static str` (keeps `Copy`, interns loaded names)? Should the site read `Theme::to_json()` output instead of parsing `theme.rs`? | `crates/mkit-core/src/theme_file.rs`, [E9_CONTRAST](E9_CONTRAST.md) | E9.3 |
| Icon set and licence; dark-first default theme (an `impl Default for Theme` returning `DARK` would enable `cx.default_global::<Theme>()`) | [plan.md E9.1–E9.2](../plan.md#e9--theming--design-language) | E9 |
| Scope: E7 grew from ~20 to 53 everyday components; E7.10–E7.13 are P1 | [plan.md §2 and E7](../plan.md#e7--everyday-components) | M3 and 1.0 scope |
| Spec vs pattern-map disagreements: TagInput editor is a plain textbox (APG expects combobox when suggestions show); Badge and KeyHint name a generic role (not allowed in ARIA); DatePicker uses a non-modal popover (APG uses a modal dialog); Accordion group name hard-coded as "Accordion"; Stepper's Alt+Left/Right bindings missing from its spec; MenuBar has no F10 binding; several specs rely on live regions the pinned accessibility bridge can't express | [component pattern map](component-pattern-map.md) rows for each component | Those components' specs |
| Docs site labels: catalog entries previously marked "ready" now say "Documented"; should the "Disclosure" and "Inline messages" cards name both components they cover (e.g. "Disclosure & accordion")? | `site/src/catalog.ts`, `site/content/catalog.json` | Docs site copy |
| E9.1 principles criterion checked against [design language](../book/src/theming/design-language.md); confirm it meets the plan | [plan.md E9.1](../plan.md#e9--theming--design-language) | — |
| Upgrade GPUI: `gpui-pre` 0.3.6 and `gpui-kit` 0.6.6 are out (pinned 0.3.5 / 0.6.x) | `scripts/check_snapshot_versions.py`, [plan.md E11.2](../plan.md#e11--gpui-tracking--compatibility) | Following GPUI Kit's pin (open question 5) |

## 2. Component specs and public API

This is the largest review. For each component, read `registry/<name>/spec.md` (states, events,
keyboard map, accessibility role, tokens, open questions) and `src/lib.rs` for the public builder
and event API.

Suggested order: review the components others build on first, because approving them settles
names and patterns reused elsewhere.

1. Foundations: [mkit-core](E4_AUDIT.md) (theme tokens, focus, overlays, state patterns), then the
   [E5 pilot specs](E5_AUDIT.md) (combobox, scrubbable number field).
2. Shared building blocks: `button`, `text-field`, `popover`, `dialog`, `dropdown-menu`,
   `virtual-list`.
3. Remaining everyday components: [E7 audit](E7_AUDIT.md), with the per-story notes for E7.10–E7.23:
   [calendar](E7_CALENDAR.md), [date picker](E7_DATE_PICKER.md),
   [disclosure](E7_DISCLOSURE.md), [toolbar](E7_TOOLBAR.md),
   [time fields](E7_TIME_FIELDS.md), [display primitives](E7_DISPLAY_PRIMITIVES.md),
   [inline messages](E7_INLINE_MESSAGES.md), [search field](E7_SEARCH_FIELD.md),
   [tag input](E7_TAG_INPUT.md), [file inputs](E7_FILE_INPUTS.md),
   [reorderable list](E7_REORDERABLE_LIST.md), [menu bar](E7_MENU_BAR.md),
   [status bar](E7_STATUS_BAR.md), [stepper](E7_STEPPER.md).
4. Pro-app components: [E8 review handoff](E8_REVIEW_HANDOFF.md) and [E8 audit](E8_AUDIT.md).
5. Tooling contracts: [`cargo mkit` CLI and `mkit.toml` format](E6_AUDIT.md).
6. External sample: [consumer release review](consumer-release-review.md) (DataTable renderer API,
   TextField secure mode, Dialog focus, 18 changed or new sample baselines).

Cross-check keyboard and accessibility contracts against the
[component pattern map](component-pattern-map.md).

## 3. Visual baselines

The 2026-09-27 visual inspection cleared the components for the docs site. AGENTS.md also asks for
explicit maintainer acceptance of harness baselines before they count as conformance evidence.
Baselines are under `registry/<name>/tests/baselines/`, `apps/gallery/snapshots/`, and (for the
property inspector and layer panel) `book/src/images/`. Record acceptance in the review log, or
note which components need visual changes.

## 4. Book accuracy

`mdbook build` and `scripts/check_book.py` verify rendering, includes and image provenance, not
whether the prose is right. Per-part notes list what each chapter claims and what was verified:
[E2.1](../book/E2.1_NOTES.md), [E2.5](../book/E2.5_NOTES.md), [E2.6](../book/E2.6_NOTES.md),
[E2.7](../book/E2.7_NOTES.md), [E2.8](../book/E2.8_NOTES.md), [E2.9](../book/E2.9_NOTES.md),
[E2.10](../book/E2.10_NOTES.md), [E2.11](../book/E2.11_NOTES.md), [E2.12](../book/E2.12_NOTES.md),
[E2.13](../book/E2.13_NOTES.md), [E2.14](../book/E2.14_NOTES.md), [E2.15](../book/E2.15_NOTES.md),
[E2.16](../book/E2.16_NOTES.md). Parts I–IV are on the Book v0.1 critical path; review those first.
Component pages are under `book/src/components/`.

## 5. Checks on a real machine

The headless harness cannot cover these. Record results in the linked note.

| Check | Record in |
|---|---|
| Native drag and drop and menus (Part IV; last attempt failed because the Mac was locked) | [E2.7 notes](../book/E2.7_NOTES.md) |
| IME candidate window with Japanese, Chinese and Korean input | [E2.8 notes](../book/E2.8_NOTES.md) |
| Native titlebar, accessibility adapter, signed installers, crash collection | [E2.11 notes](../book/E2.11_NOTES.md) |
| Screen-reader announcements: toast, inline alert, status bar, reorderable list moves | [E7 audit](E7_AUDIT.md) |
| Screen-reader walkthrough of the Aria2 sample | [consumer release review](consumer-release-review.md) |
| Trackpad gestures and dense Timeline/NodeEditor targets | [E8 review handoff](E8_REVIEW_HANDOFF.md) |
| Laika multi-photo undo/reset and keyboard flow | [E8_LAIKA_ADOPTION](E8_LAIKA_ADOPTION.md) |
| Reduced-motion OS setting changes (no Linux bridge yet) | [E4.4 notes](E4.4_NOTES.md) |
| Windows and Linux builds (needs CI on a remote) | [plan.md E0.1](../plan.md#e0--project-foundation) |
