---
spec_version: 1
component: command-palette
states:
  - id: closed
    description: Palette is hidden and does not accept input.
    fixture: closed_fixture
    screenshot_status: captured_by_e8_command_palette_matrix
  - id: open_empty
    description: Palette is open with an empty query and all registered actions shown.
    fixture: open_empty_fixture
    screenshot_status: captured_by_e8_command_palette_matrix
  - id: open_filtered
    description: Palette is open with fuzzy matches and one active result.
    fixture: open_filtered_fixture
    screenshot_status: captured_by_e8_command_palette_matrix
  - id: open_no_matches
    description: Palette is open and the query matches no registered actions.
    fixture: open_no_matches_fixture
    screenshot_status: captured_by_e8_command_palette_matrix
  - id: disabled
    description: Palette is disabled and hidden.
    fixture: disabled_fixture
    screenshot_status: captured_by_e8_command_palette_matrix
keys:
  - key: ArrowDown
    modifiers: []
    when: Palette is open and results are available.
    action: Move the active result to the next match, wrapping at the end.
    initial_state: open_filtered
    expect:
      state: open_filtered
      focus_target: search
  - key: ArrowUp
    modifiers: []
    when: Palette is open and results are available.
    action: Move the active result to the previous match, wrapping at the start.
    initial_state: open_filtered
    expect:
      state: open_filtered
      focus_target: search
  - key: Enter
    modifiers: []
    when: Palette is open with an active result.
    action: Emit activation for the active registered action and close the palette.
    initial_state: open_filtered
    expect:
      state: closed
      event: activate
      focus_target: caller
  - key: Escape
    modifiers: []
    when: Palette is open.
    action: Close the palette without activating an action.
    initial_state: open_filtered
    expect:
      state: closed
      event: close
      focus_target: caller
  - key: Tab
    modifiers: []
    when: Palette is open.
    action: Keep keyboard focus on Search (the palette's only focus stop).
    initial_state: open_filtered
    expect:
      state: open_filtered
      focus_target: search
  - key: Tab
    modifiers: [Shift]
    when: Palette is open.
    action: Keep keyboard focus on Search (the palette's only focus stop).
    initial_state: open_filtered
    expect:
      state: open_filtered
      focus_target: search
accessibility:
  role: dialog
  properties:
    - name: modal
      value: true
      when: open_empty
    - name: modal
      value: true
      when: open_filtered
    - name: modal
      value: true
      when: open_no_matches
    - name: name
      value: Command palette
      when: open_empty
    - name: name
      value: Command palette
      when: open_filtered
    - name: name
      value: Command palette
      when: open_no_matches
    - name: active-descendant-focus
      value: true
      when: open_filtered
    - name: focus-trap
      value: search-only
      when: open_empty
    - name: focus-trap
      value: search-only
      when: open_filtered
    - name: focus-trap
      value: search-only
      when: open_no_matches
---

# Command palette

## Purpose

Search and run actions registered by the host application. The visual reference is shadcn Command: a compact search surface, restrained border, clear active row, and muted shortcut text. The palette owns query and active-result navigation while the host owns the action collection, invocation, and the decision to open it.

## Anatomy

- A centered dialog surface containing an editable search field and a result list.
- Each result shows its action label, optional group, and optional caller-provided keybinding text.
- `closed`: palette content is not rendered and no focus target is exposed.
- `open_empty`: all enabled registered actions are shown; the first result is active when available.
- `open_filtered`: fuzzy matches appear in stable source order, with one active row.
- `open_no_matches`: show a muted “No results found.” message.

## States

- `closed`: palette content is not rendered and no focus target is exposed.
- `open_empty`: all enabled registered actions are shown; the first result is active when available.
- `open_filtered`: fuzzy matches appear in stable source order, with one active row.
- `open_no_matches`: show a muted “No results found.” message.
- `disabled`: hidden and unable to open or activate.

Typing updates the query and resets the active result to the first match. Fuzzy matching is case-insensitive subsequence matching against action label, group, and keywords. It preserves source order; ranking and locale-aware folding are open questions. The query is cleared each time the caller opens the uncontrolled palette. Keyboard focus stays in the search field while moving through results.

## Props and events

Required input is an action list. Each action has a stable string ID, label, and optional group, keywords, and display-only keybinding. Duplicate IDs are invalid. Keybinding strings are presentation content; the palette does not install or execute shortcut bindings.

The palette supports controlled and uncontrolled visibility. In controlled mode, `open` is authoritative and the component emits `OpenChanged` requests; the caller applies the next value with `set_open`. In uncontrolled mode, `default_open` initializes visibility and the component owns subsequent visibility. Query and active result are internal in both modes. `set_actions` replaces action props without emitting activation events and preserves the active action by ID when it still matches.

`ActionActivated { id }` is emitted once after Enter or pointer activation of an enabled result. `OpenChanged(false)` is emitted when Escape or activation closes the palette. Escape never activates. Opening or closing from caller props emits no interaction event. Disabled actions remain in the registry but are omitted from results and cannot be activated. If no result is active, Enter does nothing.

## Keyboard map

Register named actions under `CommandPalette` so applications can rebind ArrowUp, ArrowDown, Enter, Escape, Tab, and Shift+Tab. Up/Down wrap through enabled matching actions. Enter activates the active item. Escape closes without activation and returns focus to the caller's previously focused control. Opening focuses the search input. Search is the only focus stop inside this pilot; Tab and Shift+Tab keep focus in Search. The host must keep the palette mounted while open and restore or transfer focus only after it closes.

## Pointer behaviour

Clicking an enabled result activates it once and closes the palette. Disabled actions are omitted, so they cannot be pointer activated. Clicking the backdrop does not close this pilot; the host may supply its dialog dismissal handling.

## Accessibility role and properties

The open surface exposes dialog role, accessible name “Command palette”, and modal state. Search exposes a persistent “Search commands” name and current query value. Results use listbox and option roles; the active result exposes the active-descendant focus marker. Result option names include the action label, group when present, and keybinding text when present. No-results content is a non-interactive status message. Closing returns focus to the host opener where supported by the host window's prior focus handling.

## Theme tokens used

Read all colors, spacing, radii, borders, typography, and control heights from the Global `Theme`. Use surface/elevated-surface, border, text/text-muted, accent/accent-text, and focus tokens. The default search field uses the medium control height. The palette is width-constrained to a justified 560 logical pixels and its result viewport shows at most eight rows before scrolling.

## WAI-ARIA pattern reference

The surface follows the [WAI-ARIA Dialog (Modal) Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/) for dialog role, accessible name, focus entry, focus containment, and Escape dismissal. The result navigation follows the [WAI-ARIA Listbox Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/) active-descendant model while keeping focus in the search field. This GPUI pilot marks the dialog modal and active option and traps Tab/Shift+Tab at its sole focus stop. The host remains responsible for keeping the surface mounted while open and restoring focus after dismissal.

## Platform notes

Use GPUI's native text input handler for editing, selection, clipboard, and IME behavior. The palette binds named actions so applications can replace key bindings. Platform accessibility adapters need verification for modal state and active-descendant announcements. The host dialog/focus manager is responsible for containing keyboard focus and restoring focus after dismissal.

## Open questions

- Human review is required for public API/event naming, keyboard and accessibility contracts, and visual baseline changes.
- Confirm whether result ranking should prefer word-boundary and prefix matches over plain subsequence matches.
- Verify the one-stop Tab/Shift+Tab trap and opener focus restoration with native keyboard and accessibility behavior on supported platforms.
- The initial pilot is synchronous and does not support async providers, sections with headings, or live announcement of result counts.
