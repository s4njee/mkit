# mkit: The GPUI Book and a Component Kit for Pro Apps

> **Name:** mkit · crates `mkit`, `mkit-core`, `cargo-mkit` (published under the mk7s name)
> **Home:** [mk7s.dev/mkit](https://mk7s.dev/mkit) (the book, component docs, gallery)
> **Status:** Planning · **Plan version:** 0.1 · **Last updated:** 2026-09-28
> **Goal in one sentence:** The best way to learn GPUI, plus components you own, built for pro and
> creative apps, each tested to a written spec.

---

## Table of Contents

1. [Vision & Goals](#1-vision--goals)
2. [Scope](#2-scope)
3. [Guiding Principles](#3-guiding-principles)
4. [Landscape](#4-landscape)
5. [Architecture Overview](#5-architecture-overview)
6. [Planning Conventions](#6-planning-conventions)
7. [Epics & Stories](#7-epics--stories)
   - [E0 — Project Foundation](#e0--project-foundation)
   - [E1 — Test Harness & Inspector](#e1--test-harness--inspector)
   - [E2 — The GPUI Book](#e2--the-gpui-book)
   - [E3 — Book Quality Loop](#e3--book-quality-loop)
   - [E4 — mkit-core](#e4--mkit-core)
   - [E5 — Component Specs & Conformance](#e5--component-specs--conformance)
   - [E6 — Ownable Source (`cargo mkit`)](#e6--ownable-source-cargo-mkit)
   - [E7 — Everyday Components](#e7--everyday-components)
   - [E8 — Pro-App Components](#e8--pro-app-components)
   - [E9 — Theming & Design Language](#e9--theming--design-language)
   - [E10 — Reference Apps](#e10--reference-apps)
   - [E11 — GPUI Tracking & Compatibility](#e11--gpui-tracking--compatibility)
   - [E12 — Site, Release & Distribution](#e12--site-release--distribution)
   - [E13 — Parity & Reach (post-1.0)](#e13--parity--reach-post-10)
8. [Milestones & Roadmap](#8-milestones--roadmap)
9. [Token Budget](#9-token-budget)
10. [Definition of Done](#10-definition-of-done)
11. [Risks & Mitigations](#11-risks--mitigations)
12. [Open Questions](#12-open-questions)

---

## 1. Vision & Goals

### 1.1 Vision

GPUI, the UI framework behind the Zed editor, is fast, has a good programming model and runs on
three operating systems. But it is built for Zed first. Its documentation is thin, it has no
stable release rhythm, and most people learn it by reading Zed's source. GPUI Kit (Longbridge)
fills the widget gap with 60+ components. Nobody has filled the **learning** gap, and nobody
serves **pro and creative apps**: viewports, timelines, curve editors, inspectors.

mkit does four things, in priority order:

1. **The GPUI Book.** The standard guide to GPUI: every concept explained, with an example that
   compiles and renders in CI. The book is how developers discover mkit.
2. **Deeply tested components.** Fewer components than GPUI Kit, each built to a written spec,
   with keyboard behaviour that follows the WAI-ARIA Authoring Practices, accessibility-tree
   snapshots, and screenshots across themes and scales.
3. **Ownable source.** `cargo mkit add combobox` copies the component's source into your project,
   like shadcn/ui on the web. You (and your coding agent) can change anything, and `cargo mkit diff`
   and `cargo mkit update` keep you connected to upstream fixes.
4. **Pro-app components:** a pan-and-zoom viewport, scrubbable number fields, curve and gradient
   editors, colour wheels, histograms, property inspectors, timelines, node editors and layer
   panels.

### 1.2 Goals

| # | Goal | Measure |
|---|---|---|
| G1 | The book is where people learn GPUI | Parts I–IV published in M1. Referenced from community channels; top search result for core GPUI concepts |
| G2 | The book is correct | Every code sample compiles against the pinned GPUI in CI. Every rendered example has a screenshot test |
| G3 | The book teaches | Fresh coding agents build the benchmark apps using **only the book** with a ≥ 90% success rate (E3) |
| G4 | Components are trustworthy | Every component passes its spec: keyboard map, accessibility snapshot, screenshot matrix, docs page |
| G5 | Components are ownable | `add`, `diff` and `update` work across a GPUI upgrade without losing local edits (3-way merge) |
| G6 | Pro apps choose mkit | Pro components used by at least 2 real apps (Laika first) |
| G7 | Works alongside GPUI Kit | mkit and GPUI Kit build together in one app on the same GPUI version |

### 1.3 Non-Goals

- A new GPU engine or a fork of GPUI. mkit is built on GPUI as published.
- Beating GPUI Kit on component count.
- Mobile or web targets.
- Semver-level stability of GPUI types. mkit stabilizes **its own** API and absorbs GPUI churn,
  but GPUI types (`App`, `Window`, `Entity`, `Styled`) appear in it.

---

## 2. Scope

| In scope | Out of scope (for now) |
|---|---|
| The GPUI Book (concepts, guides, cookbook, testing, packaging) | Contributing documentation upstream to Zed (welcome later) |
| `mkit-core`: theme tokens, focus, overlays, motion, accessibility helpers | A new layout or text engine |
| ~20 everyday components, ~14 pro components | Charts (Holochart covers the web; a GPUI chart widget is a later idea) |
| `cargo mkit` CLI and component registry | A visual app builder |
| Test harness and inspector MCP server | Hosted services |
| 3 reference apps, including a component gallery | Porting Laika wholesale (only extracting and adopting components) |

---

## 3. Guiding Principles

1. **The book comes first.** When the book and a component compete for time, the book wins until
   M1 ships.
2. **Every claim is backed by a compiling example.** No code block exists only in the markdown.
   Examples live in a cargo workspace and are pulled into chapters with `{{#include}}` anchors.
3. **Verify with the harness, not by reading.** Screenshots, keystroke scripts and accessibility
   snapshots judge agent-written work. A component is not done until the harness says so.
4. **Specs before code.** Every component starts as a spec (states, events, keyboard map,
   accessibility role, theme tokens). Agents implement the spec; humans approve the API.
5. **Borrow keyboard behaviour; don't invent it.** Follow the WAI-ARIA Authoring Practices wherever
   a pattern exists, and platform conventions (macOS Human Interface Guidelines, Windows) where it
   doesn't.
6. **Coexist with GPUI Kit.** Track the same GPUI version so both libraries work in one app.
7. **Humans own interfaces; agents own breadth.** Public API, naming and design language get human
   review. Implementations, tests, docs pages and upgrades are agent work.

---

## 4. Landscape

As of 2026-09-24, from the crates in the local cargo registry:

| Project | What it is | Relevance |
|---|---|---|
| `gpui` (crates.io) | Official GPUI releases (e.g. 0.2.2), infrequent | Too old and too infrequent to build on |
| `gpui-pre` | Published snapshots of Zed's GPUI at a given commit (0.3.2 → 0.3.5) | **mkit's base.** Includes AccessKit accessibility, `HeadlessAppContext::capture_screenshot`, and `simulate_keystrokes` / `simulate_input` in test contexts |
| GPUI Kit (`gpui-kit`, `gpui-base`, `gpui-component`) | Longbridge's kit: an unstyled behaviour layer plus 60+ styled components, on `gpui-pre` 0.3.5 | Main incumbent. Pin the same GPUI version; interoperate rather than compete |
| Zed's internal `ui` crate | Zed's own components, not published separately | Risk: Zed could publish it |
| iced, Slint, egui, Xilem | Other Rust UI toolkits | Out of scope; the book may include "coming from X" guides |

---

## 5. Architecture Overview

```
mkit/                          (cargo workspace)
├── book/                      mdBook source: the GPUI Book
│   └── src/…                  chapters pull code from examples/ with {{#include}} anchors
├── examples/                  one crate per book example; built and screenshot-tested in CI
├── crates/
│   ├── mkit-core/             theme tokens, focus, overlays/positioning, motion, a11y helpers
│   ├── mkit-harness/          headless screenshots, keystroke scripts, accessibility snapshots
│   ├── mkit-inspector/        MCP server around the harness (screenshot, input, a11y tree)
│   ├── mkit/                  optional crate: all registry components as a normal dependency
│   └── cargo-mkit/            CLI: add / diff / update / list / doctor
├── registry/                  source of truth for ownable components
│   └── <component>/
│       ├── spec.md            states, events, keyboard map, a11y role, tokens, open questions
│       ├── src/…              the component source copied into user projects
│       ├── tests/             conformance tests generated from the spec + custom tests
│       └── docs.md            docs page (examples pulled from examples/)
├── apps/
│   ├── gallery/               every component, every state, every theme (also the screenshot source)
│   └── …                      reference apps (E10)
└── evals/                     book and component evaluation tasks for agents (E3)
```

**Component layers:**

1. **`mkit-core`:** shared foundations that every component depends on. A normal crate dependency,
   versioned carefully.
2. **Registry components:** ownable source. Each depends only on `mkit-core` and GPUI.
3. **App patterns:** inspector, command palette, settings screen. Also ownable, built from
   components.

**Component conventions** (enforced by lints and review):

- **Stateless components** implement `RenderOnce` and use a builder API.
- **Stateful components** (text input, combobox, table, viewport) are `Entity<T>` views that emit
  typed events via `EventEmitter`.
- **Controlled and uncontrolled state:** both supported through one documented pattern.
- **Keyboard:** every component registers a `key_context` and actions, so apps can rebind anything.
- **Theme:** components read tokens from a GPUI `Global`; they never use hard-coded colours.

---

## 6. Planning Conventions

### 6.1 Story Format

```
#### E2.4 — Story title   `P0` `M`   deps: E2.1, E1.2
> As a <role>, I want <capability>, so that <benefit>.
- [ ] Acceptance criterion
```

### 6.2 Priority

| Tag | Meaning |
|---|---|
| `P0` | Required for the first public release (book v0.1, M1) |
| `P1` | Required for mkit 1.0 |
| `P2` | Differentiator or polish; targeted for 1.0, can slip |
| `P3` | Post-1.0 |

### 6.3 Size

Effort for one engineer working with agents; token estimates are in §9.

| Tag | Effort |
|---|---|
| `S` | ≤ 1 day |
| `M` | 2–3 days |
| `L` | 1 week |
| `XL` | 2+ weeks. Split before starting. |

### 6.4 Roles

- **Newcomer:** learning GPUI, possibly coming from React, Tauri or iced.
- **App developer:** building an app with GPUI and mkit.
- **Pro-app developer:** building a creative, data or developer tool.
- **Agent:** a coding agent writing GPUI code from the book and docs.
- **Designer:** customizing the look.
- **End user:** uses an app built with mkit, including with assistive technology.
- **Maintainer:** works on mkit itself.

---

## 7. Epics & Stories

### E0 — Project Foundation

#### E0.1 — Workspace and repository   `P0` `S`
> As a maintainer, I want a cargo workspace with the layout in §5, so that the book, examples and
> components share one build.
- [ ] Workspace builds with `cargo build --workspace` on macOS, Windows and Linux
- [x] Licence chosen (Apache-2.0, or MIT/Apache-2.0 dual) and third-party notices set up
- [x] `rustfmt`, `clippy` (warnings are errors), `cargo-deny` for licences and advisories

#### E0.2 — Pin GPUI to GPUI Kit's version   `P0` `S`   deps: E0.1
> As an app developer, I want mkit and GPUI Kit to build in the same app, so that I can adopt mkit
> gradually.
- [x] `gpui-pre` pinned with `=` to the version GPUI Kit uses (0.3.5 at time of writing)
- [x] A CI job builds a sample app that depends on both `gpui-kit` and `mkit`
- [x] Pinning policy documented in the book's "Versions" appendix

#### E0.3 — CI matrix   `P0` `M`   deps: E0.1
> As a maintainer, I want CI on all three operating systems, so that platform regressions surface
> immediately.
- [x] macOS, Windows and Linux jobs: build, test, clippy, book build
- [x] Linux job installs GPUI's system dependencies; recorded in the book's install chapter
- [ ] Caching keeps a no-change CI run under 10 minutes

#### E0.4 — Agent working agreement   `P0` `S`   deps: E0.1
> As a maintainer, I want written rules for agent-written work, so that output stays consistent.
- [x] `CLAUDE.md` (or equivalent) covering conventions (§5), the harness, and what needs human
      review
- [x] PR template with a harness-evidence section (screenshots, snapshot diffs)

---

### E1 — Test Harness & Inspector

#### E1.1 — Headless harness spike   `P0` `M`   deps: E0.2
> As a maintainer, I want to know what GPUI's headless mode gives us, so that the harness is built
> on facts.
- [x] Render a window with `HeadlessAppContext` and capture it with `capture_screenshot`
- [ ] Determine which renderer headless mode uses on each OS, and whether screenshots are
      pixel-stable run to run and across machines
- [x] Confirm `simulate_keystrokes` and `simulate_input` drive actions, focus and text input (verified with a focused `EntityInputHandler` fixture; native IME composition remains untested)
- [ ] Confirm the accessibility tree can be read in tests (see GPUI's `window/a11y` debug module)
      — Answer: not headlessly on pinned `gpui-pre` 0.3.5. `TestAppContext`'s test window never
      activates the AccessKit adapter, so `debug_a11y_tree_json()` has no tree; capture works only
      in a real window after platform activation. See [E5 accessibility capture](docs/E5_A11Y_CAPTURE.md).
- [x] Findings written to `docs/spikes/headless.md`

#### E1.2 — Screenshot tests   `P0` `M`   deps: E1.1
> As a maintainer, I want screenshot tests with committed baselines, so that visual regressions fail
> CI.
- [x] `mkit_harness::screenshot(view, size, scale, theme)` returns an image
- [x] Baselines stored per test; a perceptual tolerance for platforms that are not bit-stable
- [x] `UPDATE_SNAPSHOTS=1` refreshes baselines; failures write a diff image to the CI artefacts
- [x] Used by every current rendered book example (E2.2; `hello`, `counter`, `reactivity_views`, and `state_entities`; `contexts` has no window)

#### E1.3 — Keyboard and pointer scripts   `P0` `M`   deps: E1.1
> As a maintainer, I want to script input and assert on the result, so that interaction behaviour
> is tested.
- [x] A small script format: `press "down down enter"`, `type "hello"`, `click @ref`,
      `drag @a -> @b`
- [x] Assertions on focused element, emitted events, entity state and screenshots
- [x] IME composition simulated where GPUI's test platform allows it; gaps documented

#### E1.4 — Accessibility snapshots   `P1` `M`   deps: E1.1
> As a maintainer, I want the accessibility tree snapshotted per component state, so that roles,
> names and states don't regress.
- [x] Stable text serialization of a supplied AccessKit debug tree (role, name, value, states, actions); headless capture remains blocked by inactive accessibility
- [ ] Snapshot per state in each component's conformance suite (E5.3)

#### E1.5 — Inspector MCP server   `P1` `L`   deps: E1.2, E1.3, E1.4
> As an agent, I want to screenshot, drive and inspect a running GPUI app, so that I can check my
> own work.
- [x] MCP tools: `launch(example)`, `screenshot`, `press`, `type`, `click`, `a11y_tree`,
      `entity_state`
- [x] Works with every current rendered book example (`hello`, `counter`, `reactivity_views`, `state_entities`) and the gallery app's shared preview view; nonvisual `contexts` reports an explicit error
- [ ] Used by the book evaluations (E3)

---

### E2 — The GPUI Book

**Highest priority.** Written in mdBook. Every code sample is included from a compiling crate in
`examples/`. Every example that renders has a screenshot test. Chapters marked `P0` make up book
v0.1.

#### E2.1 — Public API inventory   `P0` `M`   deps: E0.2
> As a maintainer, I want an inventory of GPUI's public API at the pinned version, so that the book
> covers everything and cites real names.
- [ ] Agents crawl the pinned `gpui-pre` source and produce `book/inventory.md`: every public type,
      trait, macro and module, grouped by concept, with source links. The current 634-name union
      includes shimmed rustdoc metadata for wasm `test-support` and Linux all-features, but full
      target/feature coverage remains unproved (other targets, Windows manifest, path reachability).
- [x] Each inventoried item tagged: assigned to a planned chapter, or deliberately not covered
      (and why)
- [x] Regeneration command and CI drift check for the pinned GPUI version and generated inventory
- [ ] Regenerate on every GPUI upgrade (E11.2); use the diff to drive book updates

#### E2.2 — Book infrastructure   `P0` `M`   deps: E0.3, E1.2
> As a maintainer, I want the book built and checked in CI, so that it never drifts from the code.
- [x] mdBook site with search, dark and light themes, and a version banner showing the pinned GPUI
      version
- [x] `{{#include}}` anchors only; CI fails if a Rust code block in a chapter isn't included from
      `examples/`
- [x] Each current example crate builds, runs headless, and matches its screenshot baseline on
      macOS. The pinned GPUI renderer is unavailable on Linux and Windows; those targets build and
      run the test's explicit unsupported-platform check without a pixel comparison.
- [x] Screenshots on book pages are produced by the harness, never by hand
- [x] Local link checker and spell checker (external URLs are checked for syntax, not fetched)

#### E2.3 — Chapter template and style guide   `P0` `S`   deps: E2.2
> As a maintainer, I want every chapter to have the same shape, so that agents can write chapters
> consistently and readers know what to expect.
- [x] Teaching-chapter template: what you'll build → concept → minimal compiling example → how it
      works → common mistakes → exercises → API reference links
- [x] Style guide: plain language, short sentences, terms defined at first use, no unexplained jargon
- [x] "Common mistakes" guidance cites verified GPUI issues and discussions from Zed

#### E2.4 — Part I: Getting started   `P0` `M`   deps: E2.3
> As a newcomer, I want to go from nothing to a running window, so that I can start learning.
- [x] Install: toolchain, OS prerequisites, and pinned `gpui-pre` versus `gpui` versions
- [x] Hello window: application startup, `App`, `Window`, and root view
- [x] Counter tour: GPUI Kit button, retained count, entity update, and notification
- [x] "How GPUI thinks": immediate-style rendering over retained state and the frame lifecycle;
      harness screenshots compare pixels on macOS only with this GPUI pin

#### E2.5 — Part II: State and views   `P0` `L`   deps: E2.4
> As a newcomer, I want to understand entities and contexts, so that I stop fighting the borrow
> checker.
- [x] Entities: `Entity<T>`, `WeakEntity<T>`, reading and updating, ownership rules
- [x] Contexts: `App`, `Context<T>`, `Window`, `AsyncApp`: which one you have, and why
- [x] Reactivity: `notify`, `observe`, `subscribe`, `EventEmitter`
- [x] Views vs components: `Render` vs `RenderOnce` vs `IntoElement`, and when to use each
- [x] Global state with `Global`
- [x] A "borrow-checker survival guide" chapter: the common errors and their fixes

#### E2.6 — Part III: Elements, styling and layout   `P0` `L`   deps: E2.5
> As a newcomer, I want to lay out and style UI, so that I can build real screens.
- [x] `div()` and the `Styled` builder methods, and how they map to flexbox and grid (Taffy)
- [x] Sizing, spacing, overflow and scrolling
- [x] Text: fonts, text styles, wrapping, truncation, rich text runs
- [x] Images and SVG
- [x] Conditional rendering, lists of children, `id`s and why they matter
- [x] A worked example: a settings screen

#### E2.7 — Part IV: Interaction   `P0` `L`   deps: E2.6
> As a newcomer, I want to handle input properly, so that my app feels native.
- [x] Mouse: click, hover, active and drag events; hit-testing (pointer and drag tests)
- [x] Focus: `FocusHandle`, tab order, focus-visible styling (forward traversal and keyboard baseline)
- [x] Actions and keybindings: the `actions!` macro, `KeyBinding`, `key_context`, keymaps, the
      dispatch order
- [ ] Drag and drop within the app and from the OS
- [x] Clipboard (text roundtrip and empty case)
- [ ] Menus: the application menu and context menus
- [x] A worked example: a keyboard-driven list with a startup-configurable alternate shortcut

#### E2.8 — Part V: Text input and IME   `P1` `L`   deps: E2.7
> As an app developer, I want to know how text input works, so that I can build or customize text
> fields.
- [x] The input-handler interface (`EntityInputHandler`, `ElementInputHandler`) explained with a compiling field
- [ ] IME composition: marked text, candidate window positioning (synthetic geometry tested; native candidate UI pending)
- [x] Selection, cursor movement and text-only undo in the minimal field
- [x] A worked single-line text field with harness input tests and macOS state screenshots

#### E2.9 — Part VI: Custom rendering   `P1` `L`   deps: E2.6
> As a pro-app developer, I want to draw my own elements, so that I can build canvases and
> visualizations.
- [x] The `Element` trait: `request_layout`, `prepaint`, `paint` (phase-order test)
- [x] `canvas()`, paths, quads, shadows, images (compiled scene and macOS pixel baselines)
- [x] Overlays: `anchored`, `deferred`, z-order (macOS baseline and bounds/click test; paint priority does not set hit order)
- [x] Virtualization: `uniform_list` and `list` (visible-slice test)
- [x] Animation: `with_animation`, easing, reduced motion (easing and one-shot static-state test)
- [x] GPU surfaces: what `surface()` can show (macOS `CVPixelBuffer` with pixel-asserted baseline), and the limits of GPU
      interop today
- [x] A worked pan-and-zoom canvas (input/math tests and three macOS baselines)

#### E2.10 — Part VII: Async, data and persistence   `P1` `M`   deps: E2.5
> As an app developer, I want to do background work safely, so that the UI never blocks.
- [x] Executors: foreground vs background, `spawn`, tasks and cancellation, timers (tested)
- [x] Updating entities from async code (weak, stale and released result tests)
- [x] Patterns for settings, files and databases (separate tested SQLite adapter)
- [x] A worked example: a file browser that loads directory listings in the background (tested)

#### E2.11 — Part VIII: Windows, platform and shipping   `P1` `M`   deps: E2.7
> As an app developer, I want to ship a real app, so that users can install it.
- [ ] Window options, multiple windows, custom titlebars, appearance (light and dark), DPI
- [ ] Accessibility in GPUI: how AccessKit is wired, roles and names on custom elements
- [ ] Packaging: macOS bundle, signing and notarization; Windows installer and signing; Linux
      options
- [ ] Logging, crash reporting and diagnostics patterns

#### E2.12 — Part IX: Testing GPUI apps   `P1` `M`   deps: E1.3
> As an app developer, I want to test my UI, so that I can refactor with confidence.
- [x] `TestAppContext`, `VisualTestContext`, `HeadlessAppContext` with compiling test examples
- [x] Simulated keystrokes and clicks; retained-state and macOS screenshot assertions
- [x] `mkit-harness` screenshot workflow in a compiling example, with platform limit documented

#### E2.13 — Cookbook   `P1` `L`   deps: E2.7
> As an app developer, I want short recipes for common tasks, so that I can copy a working answer.
- [x] 32 one-page recipes with compiling anchored examples and scoped tests or screenshots; includes
      debounce, modal focus, split pane, saved window size, toast queue, simulated file selection,
      in-app palette, and drag reorder. Pinned GPUI has no verified tray icon API; macOS dock menu
      is recorded as a source-only alternative outside the 32 recipes.
- [ ] Recipe requests collected from the book evaluations (E3) and from issues

#### E2.14 — "Coming from…" guides   `P2` `M`   deps: E2.7
> As a newcomer, I want GPUI explained in terms I already know, so that I learn faster.
- [x] From React (state and effects mapped to retained entities and subscriptions, with limits)
- [x] From Tauri (webview and IPC compared with the Rust counter example, with process limits)
- [x] From iced and egui (their documented models compared with GPUI retained views and render caching)

#### E2.15 — Appendices   `P1` `M`   deps: E2.1
> As a reader, I want reference material in one place, so that I can look things up quickly.
- [x] Versions: `gpui` vs `gpui-pre`, pinning policy, and reviewed upgrade steps
- [ ] Migration notes per GPUI snapshot (maintained by E11)
- [x] Glossary for the current teaching chapters
- [ ] Concept index linking every inventory item (E2.1) to its chapter

#### E2.16 — Architecture internals   `P2` `L`   deps: E2.9
> As a maintainer or advanced reader, I want to know how GPUI works inside, so that I can debug
> deep problems.
- [x] Frame pipeline: layout, prepaint, paint, scene, and pinned Metal/DirectX/Linux wgpu paths (macOS fixture; other backends source-audited)
- [x] The pinned text system per platform (CoreText, DirectWrite, CosmicTextSystem; native Windows/Linux behavior untested here)
- [x] The entity map and reference counting (pinned source plus strong/weak handle test)
- [ ] Written from source with file links; checked against each GPUI upgrade

---

### E3 — Book Quality Loop

#### E3.1 — Benchmark apps   `P0` `M`   deps: E2.4
> As a maintainer, I want fixed tasks that measure whether the book teaches, so that quality is a
> number, not an opinion.
- [ ] 10 benchmark app specs of increasing difficulty: counter; todo list with keyboard shortcuts;
      settings screen; file browser; markdown previewer; split-pane editor; image viewer with
      zoom; searchable table; canvas drawing app; multi-window notes app
- [ ] Each has acceptance tests runnable with the harness (E1)

#### E3.2 — Book evaluations   `P0` `L`   deps: E3.1, E1.5
> As a maintainer, I want fresh agents to build the benchmark apps from the book alone, so that gaps
> in the book show up.
- [ ] Agents get only the book (no GPUI source, no web) plus the inspector MCP
- [ ] Record success or failure, tokens, and the point of failure for each app
- [ ] Target: ≥ 90% success on apps 1–6 for book v0.1, ≥ 90% on all 10 for 1.0
- [ ] Results published on a "book health" page

#### E3.3 — Failure triage   `P0` `M`   deps: E3.2
> As a maintainer, I want every failure turned into a fix, so that the book improves each round.
- [ ] Each failure classified as: missing concept, unclear explanation, wrong example, GPUI bug
      or gap
- [ ] Fixes land as chapter edits, new cookbook recipes, or upstream issue reports
- [ ] Evaluations re-run after each batch of fixes; trend tracked

#### E3.4 — Human reader review   `P1` `M`   deps: E2.7
> As a maintainer, I want real newcomers to read the book, so that it works for people, not just
> agents.
- [ ] 5+ reviewers new to GPUI read Parts I–IV and complete the exercises
- [ ] Friction points recorded and fixed

---

### E4 — mkit-core

#### E4.1 — Theme tokens   `P0` `M`   deps: E0.2
> As a designer, I want every visual value to come from tokens, so that a theme changes the whole
> app.
- [x] Token set: colour roles, typography scale, spacing, radii, borders, shadows, motion durations
- [x] Tokens stored as a GPUI `Global`; runtime switching re-renders affected windows
- [x] Light, dark and high-contrast built-in themes

#### E4.2 — Focus management   `P0` `M`   deps: E0.2
> As an end user, I want predictable keyboard focus, so that I can use apps without a mouse.
- [x] Focus scopes and focus traps for dialogs and popovers
- [x] Roving focus for toolbars, lists and grids
- [x] Focus-visible styling driven by tokens

#### E4.3 — Overlays and positioning   `P0` `M`   deps: E4.2
> As a maintainer, I want one positioning system for popovers, menus and tooltips, so that they all
> behave the same.
- [x] Anchored placement with flip and shift when near window edges
- [x] Layering order, outside-click and Escape dismissal
- [x] Built on GPUI's `anchored` and `deferred`

#### E4.4 — Motion   `P1` `S`   deps: E4.1
> As an end user, I want subtle, consistent motion that respects my settings, so that the app feels
> polished but not distracting.
- [x] Transition helpers for hover, press, open and close states, using token durations
- [ ] Respect the OS reduced-motion setting

#### E4.5 — Accessibility helpers   `P0` `M`   deps: E1.1
> As a maintainer, I want helpers for roles, names and live regions, so that every component is
> accessible the same way.
- [x] Helpers to set role, name, description, value and states on elements
- [ ] Announcements for toasts and status changes

#### E4.6 — State patterns   `P0` `S`   deps: E0.2
> As an app developer, I want every stateful component to work the same way, so that I learn the
> pattern once.
- [x] Controlled and uncontrolled pattern, with typed events via `EventEmitter`
- [x] Documented in the book (Part II) and in `mkit-core` docs

---

### E5 — Component Specs & Conformance

Angle #3: depth over breadth.

#### E5.1 — Spec template   `P0` `S`   deps: E4.6
> As a maintainer, I want a spec template, so that every component is defined before it is built.
- [x] Sections: purpose; anatomy; states; props and events; keyboard map; pointer behaviour;
      accessibility role and properties; theme tokens used; WAI-ARIA pattern reference; platform
      notes; open questions
- [x] Specs are markdown with a machine-readable front block (states, keys, role)

#### E5.2 — WAI-ARIA pattern mapping   `P0` `S`   deps: E5.1
> As a maintainer, I want each component linked to its WAI-ARIA Authoring Practices pattern, so
> that keyboard behaviour is not invented.
- [x] Table mapping each planned component to an APG pattern, a platform convention, or "custom
      (justify)"
- [x] Differences from the APG documented (for example, desktop conventions that differ from the
      web)

#### E5.3 — Conformance suite generator   `P0` `L`   deps: E5.1, E1.2, E1.3, E1.4
> As a maintainer, I want tests generated from the spec, so that every component is checked the
> same way.
- [ ] From the front block, generate: keyboard tests (each key does what the spec says),
      accessibility snapshots per state, screenshot matrix (states × light/dark/high-contrast ×
      1×/2× scale)
- [x] Custom tests can be added alongside

#### E5.4 — Component definition of done   `P0` `S`   deps: E5.3
> As a maintainer, I want a checklist every component must pass, so that "done" means the same
> thing everywhere.
- [ ] See §10.2; enforced in the PR template and CI

#### E5.5 — Agent pipeline pilot   `P0` `M`   deps: E5.3, E6.1
> As a maintainer, I want to run two components through the whole pipeline with agents, so that
> the token budget is based on real data.
- [ ] Combobox (hardest common widget) and scrubbable number field (pro widget) go spec → code →
      conformance → docs → registry
- [ ] Record tokens, wall time, and how much human rework each needed
- [ ] Replace the estimates in §9 with measured numbers

---

### E6 — Ownable Source (`cargo mkit`)

Angle #1.

#### E6.1 — Registry format   `P0` `M`   deps: E0.1
> As a maintainer, I want a registry format for components, so that the CLI knows what to copy.
- [x] Each component: files, dependencies on other components, required `mkit-core` version,
      required GPUI version
- [x] Registry versioned with the mkit release; served from the repository (no server)

#### E6.2 — `cargo mkit add`   `P0` `M`   deps: E6.1
> As an app developer, I want to add a component's source to my project, so that I own and can
> change it.
- [x] Copies source into a configurable folder (default `src/ui/`), resolving component
      dependencies
- [x] Adds `mkit-core` and any crates it needs to `Cargo.toml`
- [x] Records origin and version in an `mkit.toml` lockfile

#### E6.3 — `cargo mkit diff`   `P1` `M`   deps: E6.2
> As an app developer, I want to see how my copy differs from upstream, so that I know what I
> changed.
- [x] Shows local edits against the recorded upstream version, and upstream changes since then

#### E6.4 — `cargo mkit update`   `P1` `L`   deps: E6.3
> As an app developer, I want upstream fixes without losing my edits, so that owning the source
> isn't a trap.
- [x] Three-way merge (recorded base, local, new upstream); conflicts written as standard markers
- [x] A `--check` mode for CI that reports available updates

#### E6.5 — `cargo mkit doctor`   `P2` `S`   deps: E6.2
> As an app developer, I want to check my setup, so that version mismatches are caught early.
- [x] Checks GPUI version match, `mkit-core` compatibility, and GPUI Kit coexistence

#### E6.6 — The `mkit` crate   `P1` `S`   deps: E6.1
> As an app developer, I want to use components as a normal dependency, so that I don't have to own
> the source.
- [x] One crate re-exporting every registry component, with a feature flag per component

---

### E7 — Everyday Components

Enough generic components that an app doesn't need GPUI Kit. Each passes E5.

#### E7.1 — Buttons and toggles   `P0` `M`   deps: E5.4
- [ ] Button (variants, sizes, icon, loading), icon button, toggle button, toggle group

#### E7.2 — Form controls   `P0` `L`   deps: E5.4
- [ ] Checkbox, radio group, switch, slider (with range, steps and modifier keys for fine
      adjustment), progress

#### E7.3 — Text field and text area   `P0` `L`   deps: E5.4, E2.8
- [ ] Single-line and multi-line, with IME, selection, undo, placeholder, validation state
- [x] The hardest everyday component; built early so the book's Part V draws on it

#### E7.4 — Select and combobox   `P0` `L`   deps: E7.3, E4.3
- [ ] Select, combobox with filtering, multi-select; virtualized options

#### E7.5 — Menus   `P0` `M`   deps: E4.3
- [ ] Dropdown menu, context menu, submenus, checkable items, shortcuts shown

#### E7.6 — Overlays   `P0` `M`   deps: E4.3
- [ ] Tooltip, popover, dialog, sheet, toast

#### E7.7 — Navigation   `P1` `M`   deps: E5.4
- [ ] Tabs, sidebar, breadcrumbs, segmented control

#### E7.8 — Collections   `P1` `L`   deps: E5.4
- [ ] Virtual list, tree (with keyboard navigation and lazy loading), basic data table (sort,
      resize, select)

#### E7.9 — Layout helpers   `P1` `S`   deps: E5.4
- [ ] Split pane (resizable), scroll area, form layout, separator

#### E7.10 — Calendar   `P1` `M`   deps: E5.4
> As an app developer, I want a month calendar, so that users can pick dates without typing them.
- [ ] Month grid with keyboard navigation (arrows by day and week, Page Up/Down by month,
      Shift+Page Up/Down by year, Home/End for the week), following the APG date picker grid
- [ ] Single-date and range selection; min and max dates; app-supplied disabled dates
- [ ] First day of week, month and weekday names supplied by the app, not hard-coded to English
- [x] Spec decides the date type: a small civil-date type in `mkit-core`, or `jiff`/`chrono`
      behind a feature. Record the decision for maintainer approval, since registry components
      otherwise depend only on `mkit-core` and GPUI

#### E7.11 — Date picker and date range picker   `P1` `M`   deps: E7.10, E7.3, E4.3
> As an app developer, I want a date field that opens a calendar, so that users can type or pick a
> date in one control.
- [ ] Text entry with parsing and a validation state; calendar in a popover; Escape restores the
      previous value
- [ ] Range variant with start and end fields sharing one calendar
- [ ] Controlled and uncontrolled modes; change events fire on commit, not on every keystroke

#### E7.12 — Disclosure and accordion   `P1` `S`   deps: E5.4
> As an app developer, I want collapsible sections, so that dense panels and settings pages stay
> scannable.
- [ ] Disclosure (single section) and accordion (group, single or multiple open), following the APG
      Disclosure and Accordion patterns
- [ ] Expanded state exposed to accessibility; optional motion from E4.4
- [ ] Property inspector (E8.8) group rows adopt it instead of their own implementation

#### E7.13 — Toolbar   `P1` `M`   deps: E7.1, E7.5
> As an app developer, I want a toolbar that handles overflow and keyboard focus, so that I don't
> hand-build one for every window.
- [ ] Buttons, toggle buttons, toggle groups and separators in one row, following the APG Toolbar
      pattern (one Tab stop, arrow keys between items)
- [ ] Items that don't fit move into an overflow menu; items can be marked never-overflow
- [ ] Horizontal and vertical orientation

#### E7.14 — Time and date-time fields   `P2` `M`   deps: E7.3
> As an app developer, I want segmented time entry, so that users can set hours, minutes and
> seconds precisely.
- [ ] Segmented field (hour, minute, optional second, AM/PM), each segment a spinbutton with arrow
      keys and typed entry
- [ ] 12- and 24-hour modes; step (for example 15 minutes); min and max
- [ ] Date-time variant combining E7.11 and the time field

#### E7.15 — Display primitives   `P2` `S`   deps: E5.4
> As an app developer, I want small display elements with consistent styling, so that status and
> metadata look the same across the app.
- [ ] Badge (count and status variants), avatar (image, initials fallback), key hint (renders a
      shortcut with platform key names), link
- [ ] Key hint shared with menus (E7.5), command palette (E8.12) and shortcut editor (E8.13)

#### E7.16 — Inline alert and empty state   `P2` `S`   deps: E7.1
> As an app developer, I want inline messages and empty-state panels, so that errors and blank
> views explain themselves.
- [ ] Inline alert (info, success, warning, error) with optional actions and dismiss; announced
      as alert or status by urgency, like toast (E7.6)
- [ ] Empty state with icon, title, description and actions

#### E7.17 — Search field   `P2` `S`   deps: E7.3
> As an app developer, I want a search field, so that filtering lists and panels is consistent.
- [ ] Text field with search icon, clear button, Escape to clear, and an optional debounce
- [ ] Optional result count and a keyboard shortcut to focus it

#### E7.18 — Tag input   `P2` `M`   deps: E7.3, E7.4
> As an app developer, I want a field that holds a list of tags, so that users can add and remove
> labels, recipients or filters.
- [ ] Type and press Enter or comma to add; Backspace and arrow keys to select and remove tags
- [ ] Optional suggestions from the combobox (E7.4); validation per tag; max count

#### E7.19 — File drop zone and file field   `P2` `M`   deps: E7.1
> As an app developer, I want a drop target and a file field, so that users can bring files in by
> dragging or browsing.
- [ ] Accepts files dragged from the OS; hover and reject states; filter by extension or type
- [ ] Button that opens the platform file dialog, as a keyboard path for every drop
- [ ] Selected files shown with remove actions

#### E7.20 — Reorderable list   `P2` `M`   deps: E7.8
> As an app developer, I want a list users can reorder, so that ordering priorities, columns or
> playlists doesn't need custom drag code.
- [ ] Drag to reorder with a drop indicator; keyboard move up and down; position change announced
- [ ] Works with the virtual list (E7.8); layer panel (E8.11) shares the reorder logic

#### E7.21 — In-window menu bar   `P2` `M`   deps: E7.5
> As an app developer, I want a menu bar drawn inside the window, so that Windows and Linux builds
> have one where the platform doesn't.
- [ ] Follows the APG Menubar pattern; Alt focuses the bar and mnemonics open menus
- [ ] Built from the same menu model as GPUI's native macOS menus, so apps define menus once

#### E7.22 — Status bar   `P2` `S`   deps: E5.4
> As an app developer, I want a status bar, so that windows can show state and quick controls along
> the bottom edge.
- [ ] Leading and trailing regions of text, buttons and progress; items can collapse at narrow
      widths
- [ ] Status updates announced politely, not moving focus

#### E7.23 — Stepper   `P3` `M`   deps: E7.1
> As an app developer, I want a multi-step flow, so that setup and import assistants guide users
> through ordered steps.
- [ ] Step indicator (current, complete, error), back and next with per-step validation
- [ ] Works inside a dialog or sheet (E7.6)

---

### E8 — Pro-App Components

Angle #2: what creative, data and developer tools need, and what nobody else provides.

#### E8.1 — Viewport   `P0` `L`   deps: E2.9, E5.4
> As a pro-app developer, I want a pan-and-zoom surface, so that I can build canvases, image
> viewers and editors.
- [ ] Pan (drag, trackpad, space-drag), zoom (pinch, wheel, keyboard) around the cursor, fit and
      100% commands
- [ ] Coordinate transforms exposed; content drawn by the app
- [ ] Optional rulers and guides

#### E8.2 — Scrubbable number field   `P0` `M`   deps: E7.3
> As a pro-app developer, I want number fields that drag to change, so that tweaking values is fast.
- [ ] Drag to scrub, with modifier keys for fine and coarse steps; click to type; arrow keys;
      units; min, max and step
- [ ] Pilot component for E5.5

#### E8.3 — Precision slider   `P1` `M`   deps: E7.2
- [ ] Double-click to reset, fine-drag modifier, bipolar (centre-zero) mode, value tooltip

#### E8.4 — Curve editor   `P1` `L`   deps: E8.1
- [ ] Tone-curve style editor: add, move and delete points; per-channel curves; smooth or linear
      interpolation; keyboard nudging

#### E8.5 — Colour tools   `P1` `L`   deps: E8.2
- [ ] Colour wheel (hue/saturation plus lightness), colour picker with numeric entry (sRGB, HSL,
      OKLCH), eyedropper where the platform allows
- [ ] Colour-grading wheels (shadows, midtones, highlights)

#### E8.6 — Gradient editor   `P2` `M`   deps: E8.5
- [ ] Add, move and delete stops; per-stop colour and position; preview

#### E8.7 — Histogram   `P1` `S`   deps: E5.4
- [ ] Per-channel and luminance, clipping indicators; data supplied by the app

#### E8.8 — Property inspector   `P1` `L`   deps: E8.2, E7.2, E7.4
> As a pro-app developer, I want a property panel generated from my data, so that I don't
> hand-build inspectors.
- [ ] Grouped, collapsible rows; editors chosen by type (number, text, bool, enum, colour, vector)
- [ ] Multi-selection with mixed values; reset to default

#### E8.9 — Timeline   `P2` `XL`   deps: E8.1
- [ ] Tracks, clips, playhead, zoomable time ruler, snapping, keyframe markers
- [x] Split into stories before starting ([T1–T4 proposal](docs/E8_TIMELINE_SPLIT.md))

#### E8.10 — Node editor   `P2` `XL`   deps: E8.1
- [ ] Nodes, ports, connections, box selection, minimap
- [x] Split into stories before starting ([N1–N4 proposal](docs/E8_NODE_EDITOR_SPLIT.md))

#### E8.11 — Layer panel   `P2` `M`   deps: E7.8
- [ ] Reorderable tree with visibility and lock toggles, thumbnails, groups

#### E8.12 — Command palette   `P1` `M`   deps: E7.4
- [ ] Fuzzy search over registered actions, showing their keybindings

#### E8.13 — Shortcut editor   `P2` `M`   deps: E8.12
- [ ] View and rebind keybindings; conflict detection; saves a keymap file

#### E8.14 — Seed components from Laika   `P0` `M`   deps: E5.4
> As a maintainer, I want to start from controls already proven in a real app, so that the first
> pro components aren't built from scratch.
- [ ] Extract Laika's slider, histogram and segmented control (`crates/laika-app/src/controls/`)
      into registry components, rewritten to the spec and conformance suite
- [ ] Laika adopts the mkit versions as the first real user

---

### E9 — Theming & Design Language

#### E9.1 — mkit design language   `P1` `M`   deps: E4.1
> As a designer, I want a distinctive default look for pro apps, so that mkit apps don't look
> generic.
- [ ] Dense, dark-first default suited to pro tools; a light variant
- [x] Documented principles: density, contrast, typography, iconography
      ([design language](book/src/theming/design-language.md))

#### E9.2 — Icons   `P1` `M`   deps: E9.1
- [ ] An icon set (licensed for redistribution) with a consistent grid; icon component

#### E9.3 — Theme authoring   `P2` `M`   deps: E4.1
- [ ] Load themes from a file; a theme editor in the gallery app; export tokens

---

### E10 — Reference Apps

#### E10.1 — Gallery   `P0` `M`   deps: E7.1
> As a reader, I want to see every component in every state, so that I can judge and choose them.
- [ ] Every component, state, theme and scale; doubles as the screenshot source for the docs
- [ ] Live theme switching

#### E10.2 — Photo adjustment demo   `P1` `L`   deps: E8.1, E8.3, E8.4, E8.7
> As a pro-app developer, I want a realistic example of a pro tool, so that I can see the
> components working together.
- [ ] Open an image; viewport; histogram; curve; sliders; before/after split

#### E10.3 — Book companion apps   `P1` `M`   deps: E3.1
- [ ] Reference solutions for the 10 benchmark apps, linked from the book (hidden from evaluation
      agents)

#### E10.4 — External consumer sample   `P0` `M`   deps: E6.6, E7.1, E7.3, E7.6
> As an app developer, I want a realistic app outside the workspace consuming mkit, so that
> release and integration gaps appear before the crate is recommended.
- [x] Recreate the Aria2 Manager handoff with local fixture data in a separate Cargo workspace
- [x] Exercise public mkit controls, keyboard navigation, themes, and screenshot baselines
- [x] Capture native macOS accessibility trees for the sample list, Add URL dialog, and Settings
- [ ] Complete native screen-reader review and the active per-state accessibility matrix
- [x] Replace GPUI-only table cells and static form affordances with mkit DataTable, Select, Slider, and TextField composition
- [x] Record build, test, lint, book, and coexistence results in the sample README
- [x] Smoke-test source vending in a fresh external Cargo project with a temporary release-candidate registry entry
- [ ] Verify the published or production source-ready distribution path after maintainer review

---

### E11 — GPUI Tracking & Compatibility

#### E11.1 — Snapshot watch   `P1` `S`   deps: E0.2
> As a maintainer, I want to know when a new GPUI snapshot or GPUI Kit release appears, so that
> upgrades start promptly.
- [ ] Scheduled check for new `gpui-pre` and `gpui-kit` versions; opens an issue

#### E11.2 — Upgrade pipeline   `P1` `M`   deps: E11.1, E2.1
> As a maintainer, I want upgrades done mostly by agents, so that tracking GPUI stays cheap.
- [ ] Agent-driven upgrade PR: bump the pin, fix build errors, regenerate the API inventory, update
      affected chapters, refresh baselines with human review of visual diffs
- [ ] Migration notes written to the book appendix (E2.15)
- [ ] Target: upgrade merged within 7 days of a snapshot that GPUI Kit also adopts

#### E11.3 — Coexistence test   `P1` `S`   deps: E0.2
- [ ] CI app using GPUI Kit and mkit components in one window, with shared theme colours

---

### E12 — Site, Release & Distribution

#### E12.1 — Site   `P0` `M`   deps: E2.2
> As a reader, I want the book and component docs at one address, so that there is one place to go.
- [ ] Book, component docs, gallery screenshots and book-health page at mk7s.dev/mkit
- [ ] Built in this repository and copied into the mk7s site's public folder (as Holochart's docs
      are)

#### E12.2 — Book v0.1 launch   `P0` `S`   deps: E2.4, E2.5, E2.6, E2.7, E3.2
- [ ] Parts I–IV live; evaluation results published; announcement written

#### E12.3 — Crate releases   `P1` `M`   deps: E6.2
- [ ] `mkit-core`, `mkit`, `cargo-mkit` and `mkit-harness` published to crates.io
- [ ] Changelog, semver policy for mkit's own API, GPUI version compatibility table

#### E12.4 — `llms.txt` and agent docs   `P1` `S`   deps: E12.1
> As an agent, I want the book and component docs in a form I can load, so that I write correct
> code.
- [ ] `llms.txt` index and a full-text export of the book and component docs
- [ ] Inspector MCP setup instructions

---

### E13 — Parity & Reach (post-1.0)

> **Status:** M5 proposed 2026-09-28, pending maintainer approval. The maintainer approved starting
> the visual-parity stories (E13.1, E13.2) now, in parallel with M1–M4; the rest follows 1.0. It
> finishes the visual pass started on E7.1–E7.7, fills API gaps the reviews surfaced, and extends
> mkit to charts, composed blocks, and every desktop platform.

#### E13.1 — Visual parity for the remaining everyday components   `P3` `L`   deps: E5.4
> As an app developer, I want every everyday component to look as finished as the docs previews,
> so that an app built from mkit looks consistent.
- [ ] Restyle to the shadcn look already applied to E7.1–E7.7, using theme tokens only: calendar
      grid, date/time fields, disclosure and accordion, toolbar, display primitives, inline alert
      and empty state, search field (including its icon placement) and tag input, file inputs,
      reorderable list, menu bar, status bar, stepper, collections, and layout helpers
- [ ] Each component's spec Theme section, screenshot matrix and book image updated together;
      baselines inspected in every theme and scale
- [ ] Focus-ring states added to screenshot matrices where none exists

#### E13.2 — Visual parity for pro-app components   `P3` `XL`   deps: E13.1
> As a pro-app developer, I want the pro components to share the everyday components' visual
> language, so that editors and panels don't look bolted on.
- [ ] Restyle the E8 components (viewport chrome, number field, precision slider, curve and
      gradient editors, colour tools, histogram, property inspector, timeline, node editor, layer
      panel, command palette, shortcut editor) to the same tokens and metrics
- [x] Split into stories before starting ([P1–P4 proposal](docs/E13.2_SPLIT.md))

#### E13.3 — Component API completeness   `P3` `L`   deps: E5.4
> As an app developer, I want the features the docs previews show, so that I don't wrap mkit
> components to get them.
- [ ] Spec first, then implement, for the gaps recorded in `docs/REVIEW_QUEUE.md`: menu labels,
      separators, icons and destructive items; dialog close button and footer; toast action and
      icon; side-docked sheet; removable multi-select chips; button loading spinner; outline and
      borderless toggle variants; segmented-control and sidebar item icons; sidebar group labels;
      breadcrumb elision; visible field descriptions
- [ ] Each public API change approved by a maintainer before implementation

#### E13.4 — Blocks   `P3` `M`   deps: E6.2, E13.1
> As an app developer, I want composed screens I can copy in, so that I start from a working app
> shell instead of single components.
- [ ] Registry "blocks" built only from mkit components: app shell (sidebar, toolbar, status
      bar), settings page, data view (search, table, detail), editor chrome (viewport, inspector,
      layers)
- [ ] Installed with `cargo mkit add`, tested with the harness, shown in the gallery and docs

#### E13.5 — Charts   `P3` `XL`   deps: E2.9, E8.1
> As a data-tool developer, I want native GPUI charts, so that dashboards don't need a web view.
- [ ] Answers open question 6 (charts are out of scope in §2 until M5 is approved): spike a chart
      component that shares ideas with Holochart
- [ ] Line, bar and area charts with axes, legend, tooltip, keyboard data inspection and a text
      alternative
- [ ] Split into stories before starting

#### E13.6 — Cross-platform harness   `P3` `L`   deps: E1.1, E1.4
> As a maintainer, I want screenshots and accessibility snapshots on Linux and Windows, so that
> conformance isn't macOS-only.
- [ ] Screenshot rendering on Linux and Windows CI runners, with per-platform baselines or
      documented tolerances
- [ ] Accessibility tree capture on Linux and Windows (today's capture is macOS-only)
- [ ] Propose the GPUI test-support accessibility hook upstream so the harness-owned platform
      in `crates/mkit-harness` can be retired

---

## 8. Milestones & Roadmap

| Milestone | Weeks | Contents | Exit criteria |
|---|---|---|---|
| **M0 — Foundation** | 0–3 | E0, E1.1–E1.3, E2.1–E2.3, E4.1, E4.6 | Harness produces stable screenshots; book builds in CI with one included example |
| **M1 — Book v0.1** | 3–10 | E2.4–E2.7, E3.1–E3.3, E12.1–E12.2 | Parts I–IV published; ≥ 90% agent success on benchmark apps 1–6 |
| **M2 — Pipeline** | 8–14 | E4.2–E4.5, E5, E6.1–E6.2, E8.2, E8.14, E7.1, E10.1 | Pilot measured (E5.5); `cargo mkit add` works; first 6 components pass conformance |
| **M3 — Breadth** | 14–26 | E2.8–E2.13, E7.2–E7.13, E8.1, E8.3–E8.8, E8.12, E6.3–E6.4, E1.4–E1.5, E11 | Book Parts V–IX; all `P0`/`P1` components; `diff`/`update` work across one GPUI upgrade |
| **M4 — 1.0** | 26–36 | E2.14–E2.16, E3.4, E9, E10.2–E10.3, E12.3–E12.4, `P2` components as time allows | ≥ 90% agent success on all 10 benchmark apps; Laika uses mkit components; crates published |
| **M5 — Parity & Reach** *(proposed)* | 36–52; E13.1–E13.2 start now | E13 | Every component matches the docs previews and passes its screenshot matrix with focus states; review-queue API gaps closed; four blocks installable with `cargo mkit add`; line, bar and area charts pass conformance; screenshot and accessibility conformance run on macOS, Linux and Windows CI |

The book is on the critical path through M1. Component work in M2 overlaps the end of M1 only once
Parts I–III are drafted.

---

## 9. Token Budget

Estimates to be replaced by measured numbers after the pilot (E5.5). Most usage is cached context
reads, so actual cost is well below the headline token count.

| Work | Estimate |
|---|---|
| Harness and inspector MCP (E1) | 30–60M |
| API inventory and book infrastructure (E2.1–E2.3) | 20–40M |
| Book chapters, Parts I–IX, cookbook, guides, appendices (E2.4–E2.16) | 120–250M |
| Book evaluations and triage rounds (E3) | 80–150M |
| `mkit-core` and conformance generator (E4, E5) | 40–80M |
| CLI and registry (E6) | 30–60M |
| Everyday components, ~20 at 3–5M (E7) | 60–100M |
| Pro components, ~14 at 4–8M (E8) | 60–110M |
| Reference apps (E10) | 40–80M |
| GPUI upgrades | 5–20M each |
| Review and rework (~30%) | 150–280M |
| **Total to 1.0** | **~0.65–1.2B** |

---

## 10. Definition of Done

### 10.1 Every Book Chapter

- [ ] Follows the chapter template (E2.3)
- [ ] Every code sample is included from a compiling example crate
- [ ] Every rendered example has a harness screenshot and baseline
- [ ] Every GPUI item mentioned links to the API inventory (E2.1)
- [ ] Exercises have reference solutions (hidden from evaluation agents)
- [ ] Reviewed by a human for accuracy and plain language
- [ ] Book evaluations re-run if the chapter is on a benchmark app's path

### 10.2 Every Component

- [ ] Spec approved by a human (API, keyboard map, accessibility role)
- [ ] Conformance suite passes: keyboard, accessibility snapshots, screenshot matrix
- [ ] Only theme tokens; no hard-coded colours or sizes
- [ ] Controlled and uncontrolled modes where applicable
- [ ] Docs page with at least 3 examples, generated screenshots, keyboard table
- [ ] Available via `cargo mkit add` and the `mkit` crate
- [ ] Shown in the gallery in every state
- [ ] Builds alongside GPUI Kit

---

## 11. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| GPUI snapshots break the book and components often | High | Medium | Agent-driven upgrade pipeline (E11.2); API inventory diff shows exactly what changed |
| Headless screenshots are not stable across machines | Medium | High | Spike first (E1.1); perceptual tolerance; one blocking CI platform |
| Book examples look right but teach wrong patterns | Medium | High | Human review of every chapter; evaluations measure real outcomes |
| GPUI Kit adds ownable source or pro components | Medium | Medium | Coexist by design; the book stays the main differentiator |
| Zed publishes its own component library or official docs | Low–Medium | High | Offer the book upstream; stay focused on pro components |
| IME and text input can't be fully tested headless | High | Medium | Manual test checklist on real machines for Japanese, Chinese and Korean input |
| Ownable-source updates cause merge pain | Medium | Medium | Three-way merge with recorded bases; keep components small and layered |
| Agent-written components are inconsistent | Medium | Medium | Specs, generated conformance tests, lints, human API review |
| The book goes stale after launch | Medium | High | Inventory regeneration and evaluation re-runs on every upgrade |

---

## 12. Open Questions

1. **Relationship with GPUI Kit:** reach out to Longbridge about coexistence, shared tokens, or
   depending on `gpui-base` for text input instead of building our own?
2. **Upstream:** would Zed accept the book (or parts of it) into GPUI's own docs? Should we ask
   before launch?
3. **Licence:** Apache-2.0 alone (matching GPUI and GPUI Kit), or MIT/Apache-2.0 dual?
4. **Book name and URL:** "The GPUI Book" at mk7s.dev/mkit/book, or a separate domain?
5. **GPUI version policy:** always follow GPUI Kit's pin, or lead it when a snapshot has fixes we
   need?
6. **Charts:** is a native GPUI chart component (sharing ideas with Holochart) a later epic?
7. **Where the repository lives:** a new repository under the mk7s GitHub organization, with the
   built site copied into `mk7s/public/mkit/`?
