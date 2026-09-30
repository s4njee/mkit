# Agent working agreement

Follow `plan.md` for product scope and priorities. Work in small changes that a maintainer can review. Keep the book, examples, component specs, and implementation in sync.

## Component conventions

- Write or update the component spec before implementation. Record states, events, keyboard behavior, accessibility role and properties, theme tokens, and open questions.
- Use `RenderOnce` and a builder API for stateless components. Use an `Entity<T>` view with typed `EventEmitter` events for stateful components.
- For stateful components, document the controlled and uncontrolled modes and their event contract.
- Register a `key_context` and actions for keyboard behavior, so apps can rebind shortcuts. Follow a relevant WAI-ARIA Authoring Practices pattern or documented platform convention.
- Read theme values from the GPUI `Global` token set. Do not hard-code component colors. Treat any fixed size as a token or justify it in the spec.
- Keep registry components dependent only on `mkit-core` and GPUI unless the approved spec requires otherwise.

## Evidence and review

- Check behavior with the harness when it is available. Include keyboard script results, accessibility snapshots, and screenshot results for affected states, themes, and scales. Inspect snapshot diffs before updating baselines.
- If the harness cannot cover a behavior yet, describe the gap and the manual check in the PR. Do not claim a harness result that was not run.
- Book code samples must come from compiling example crates via mdBook `{{#include}}` anchors once the example infrastructure exists. Rendered examples need harness screenshots. Do not add illustrative Rust blocks that cannot be checked.
- Request human review for public API and naming, component specs (including keyboard and accessibility contracts), visual baseline changes, and book accuracy. Agents may implement, test, and draft documentation, but these interfaces need maintainer judgment.
- Run the relevant workspace build, tests, lint, book build, and coexistence check for the area changed. Record commands and results in the PR.

## Screenshots and the site

- Capture book and site screenshots with the `SHADCN_LIGHT`, `SHADCN_DARK`, and `HIGH_CONTRAST` themes at 1× and 2×. The first image in a component chapter is the shadcn dark 2× capture, because the book and site are dark by default. After refreshing E7 registry baselines, run `python3 scripts/sync_e7_book_images.py`.
- The site in `site/` builds component pages from the book. Do not edit `site/src/generated/`; change the book or `site/content/catalog.json`, then run `npm --prefix site run content`.
- Every component chapter needs a live web preview in `site/src/demos/`, keyed by the chapter file name. Catalog cards show the card's `featured` preview, or the first chapter's preview when none is set. When a component's states, variants, or keyboard behavior change, update its preview in the same change.
- Build previews from the `.ui-*` primitives and `--ui-*` variables in `site/src/ui/`. Those variables come from mkit-core's shadcn themes, so do not hard-code chrome colors. Colors that are app data, such as histogram channels or gradient stops, are fine.
- A preview illustrates the component; it is not evidence. Keep the harness capture in the page's "GPUI render" tab, and never cite a preview as a test result.
- For site changes, run `npm --prefix site run typecheck` and `npm --prefix site run build`. Check affected pages in both site themes and at a 375 px width.

The E0 foundation may not yet have all harness and book checks. State which checks exist and which are pending instead of treating planned checks as complete.
