# Benchmark 03: Settings screen

## Goal

Build a preferences window with a sidebar of sections, switches, a theme choice that recolours the
whole window, and a font-size control. It must work with the pointer and the keyboard.

## Required features

- A sidebar with three sections, in order: `general`, `appearance`, `notifications` (display them
  as General, Appearance, Notifications). The selected section is highlighted and its settings fill
  the main area, which scrolls when it is taller than the window.
- **General:** switches *Launch at login* (default off) and *Restore windows* (default on).
- **Appearance:** a theme choice between Light (default) and Dark, and a font size shown as a
  number with smaller and larger buttons (default 14, range 10 to 24, step 1).
- **Notifications:** a switch *Desktop alerts* (default on). The *Alert sound* switch (default off)
  is shown only while desktop alerts are on.
- The theme is stored in a GPUI `Global` and drives the background and text colours of the whole
  window. Light has a light background; Dark has a dark background.
- Keyboard focus starts on the sidebar.

## Keyboard

| Key | Where | Command |
|---|---|---|
| `up` / `down` | Sidebar | Select the previous or next section (stops at the ends) |
| `tab` / `shift-tab` | Anywhere | Move focus through the sidebar, then the visible controls of the section, in display order |
| `space` | A focused control | Toggle a switch, choose a theme, or press a font-size button |

## Targets

| Name | Element |
|---|---|
| `section-<name>` | Sidebar entry, e.g. `section-appearance` |
| `launch-at-login`, `restore-windows`, `desktop-alerts`, `alert-sound` | Switches |
| `theme-light`, `theme-dark` | Theme choices |
| `font-smaller`, `font-larger` | Font size buttons |

`alert-sound` must not be rendered while desktop alerts are off.

## Snapshot

```json
{
  "section": "general",
  "launch_at_login": false,
  "restore_windows": true,
  "theme": "light",
  "font_size": 14,
  "desktop_alerts": true,
  "alert_sound": false,
  "focus": "sidebar"
}
```

`focus` is `"sidebar"`, the target name of the focused control (for example `"launch-at-login"` or
`"theme-dark"`), or `null` when nothing in the window has focus.

## Acceptance criteria

- **AC0** The crate builds, including `src/main.rs`.
- **AC1** The first snapshot matches the defaults above.
- **AC2** Clicking `section-appearance` then `section-notifications` updates `section` each time.
- **AC3** In General, clicking `launch-at-login` turns it on and clicking `restore-windows` turns it
  off.
- **AC4** In Notifications, `alert-sound` is rendered; clicking `desktop-alerts` turns alerts off
  and removes `alert-sound`; clicking again restores it, still off.
- **AC5** In Appearance, `font-larger` 12 times gives 24, and `font-smaller` 20 times gives 10.
  Clicking `theme-dark` sets `theme` to `"dark"`.
- **AC6** `down down` selects notifications and `up` selects appearance, with `focus` still
  `"sidebar"`. From General, `tab` focuses `launch-at-login`, `space` turns it on, `tab` focuses
  `restore-windows`, and `shift-tab shift-tab` returns to `"sidebar"`.
- **AC7** (screenshot) The first frame is not blank. After `down tab tab space` (Appearance, then
  focus Light, then Dark, then choose it), the frame's mean luminance is at least 0.15 lower than
  with Light.
- **AC8** (accessibility) When GPUI exposes an accessibility tree, it names the General switches.

<!-- maintainer-only -->

## Book coverage

Answerable from Parts I–IV. `elements/settings-screen.md` is the closest worked example (sidebar,
scrollable content, switches, conditional note); the Part III chapters cover layout, scrolling,
text, and conditional rows. `state/globals.md` covers the theme global, `interaction/focus.md` the
tab order, and `interaction/actions.md` the bound sidebar commands.

## Harness coverage and gaps

AC7 uses mean luminance rather than a baseline because each candidate chooses its own palette.
Focus-visible styling and scrolling are not asserted. AC8 is skipped while headless accessibility
is inactive.
