---
spec_version: 1
component: key-hint
states:
  - id: single_key
    description: A single key is shown with its platform display name.
    fixture: single_fixture
  - id: modifier_chord
    description: Modifier keys and a main key are shown in platform order.
    fixture: chord_fixture
  - id: named_key
    description: A special key uses a localized caller-provided display label.
    fixture: named_fixture
keys: []
accessibility:
  role: generic
  properties:
    - name: label
      value: spoken shortcut chord
      when: single_key
---

# KeyHint

## Purpose

Render a keyboard shortcut consistently in menus, command surfaces, and shortcut settings.

## Anatomy

A sequence of compact keycaps shows zero or more modifiers followed by a main key. Modifier names and order are selected for the current operating system; the key label is app-supplied so special key names can be localized.

## States

- `Single key`: a main key without modifiers.
- `Modifier chord`: platform-aware modifier keycaps and a main key.
- `Named key`: caller-provided visible label for a special key or localized key name.

## Props and events

Stateless `RenderOnce` builder; no events. A `KeyChord` carries modifier flags and a main-key display string. An optional accessible label can provide localized speech; otherwise the component generates a platform-aware spoken chord name.

## Keyboard map

No key context; this is a display primitive and does not handle key presses.

## Pointer behaviour

None.

## Accessibility role and properties

Generic, nonfocusable content with an accessible name describing the full chord in reading order. Decorative keycap borders do not create nested accessibility nodes.

## Theme tokens used

Uses `Theme.colors.surface`, `elevated_surface`, `text`, `text_muted`, and `border`; `Theme.typography.caption`, `spacing.xsmall/small`, `radii.small`, and `borders.hairline`.

## WAI-ARIA pattern reference

No dedicated APG pattern applies. Shortcut text follows the ARIA keyboard shortcut guidance; this visual hint does not bind an action or set `aria-keyshortcuts` on another control.

## Platform notes

macOS displays Command, Option, Control, and Shift glyph names. Other platforms display Ctrl, Alt, Super, and Shift text labels. Main-key text is caller-supplied to allow locale-aware labels. Platform rendering is compile-target dependent.

## Open questions

- Maintainer review is needed for platform modifier ordering, glyphs, and spoken labels.
- Menus, CommandPalette, and ShortcutEditor adoption is a follow-up integration; this primitive does not alter their behavior in this draft.
