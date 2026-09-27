# Handoff: Aria2 Download Manager UI

## Overview
A cross-platform desktop download manager built on the aria2 engine, in the tradition of Internet Download Manager and DownThemAll. The design covers the main download list, the add-download dialog, three alternative treatments of multi-connection segment progress, a browser-capture popup, a queues/scheduler screen, and engine settings for the aria2 RPC connection.

Product decisions already made:
- Cross-platform / OS-neutral shell (not Windows- or macOS-specific chrome, though the accent is macOS blue).
- HTTP/FTP downloads only. No BitTorrent or Metalink peer UI.
- aria2 internals are **surfaced in advanced areas only** — the main list and add dialog read as an ordinary download manager; raw `--flags` appear behind an "Advanced" disclosure and in Settings → Engine.
- Balanced density: a real data grid, but with breathing room and two-line file rows.

## About the Design Files
`Aria2 Download Manager.dc.html` (plus its `support.js` runtime) is a **design reference created in HTML** — a prototype showing intended look, layout, and data shape. It is not production code to copy.

The task is to **recreate these designs in the target codebase's existing environment** (Electron + React, Tauri, Qt, SwiftUI, GTK, etc.) using its established patterns, component library, and state management. If no codebase exists yet, choose the framework appropriate for a desktop aria2 frontend (Electron/Tauri + React is the common choice, since aria2 exposes a JSON-RPC/WebSocket API) and implement there.

The HTML file is a "design component": one file containing a template and a small logic class that generates mock data. Ignore its templating mechanics; read it for markup structure, exact style values, and the mock data shape.

## Fidelity
**High-fidelity.** Colors, typography, spacing, and border radii are final and exact. Recreate pixel-faithfully using the codebase's own primitives. What is *not* specified: real interaction wiring, animation timings beyond the notes below, and empty/loading states — those are called out as gaps at the end.

## Design Tokens

### Color — light theme (primary)
| Role | Hex |
|---|---|
| Accent (macOS blue) | `#007AFF` |
| Accent tint / selected row background | `#E5F0FF` |
| Accent deep (text on tint) | `#0A4C9E` |
| Accent light (in-flight segment) | `#7FB4FF` |
| Accent hover/pressed link | `#0060DF` |
| Schedule "throttled" fill | `#B3D4FF` |
| App background (canvas behind window) | `#ECEEEF` |
| Surface / card | `#FFFFFF` |
| Surface subtle (titlebar, sidebar, footer, table header) | `#FAFBFB` / `#F2F4F5` |
| Field background | `#F4F6F7` |
| Border strong | `#E3E6E8` |
| Border field | `#DCE0E3` |
| Border row divider | `#F0F2F3` |
| Track / pending segment | `#E7EBED` / `#EBEEF0` |
| Text primary | `#14181B` |
| Text secondary | `#3D464E` |
| Text tertiary | `#6B7580` |
| Text muted / mono meta | `#9AA5AE` |
| Text faint (lane numbers) | `#B4BCC2` |
| Header label | `#8B959D` |
| Success | `#0E9F6E`; bg `#E8F7F0` / `#F0FAF6`, text `#0B6B4B`, border `#CBEBDD` |
| Warning / stalled | `#D97706`; bar `#E8B04B`, bg `#FDF3E4`, text `#8A5A11` |
| Danger | `#DC2626`; bg `#FDECEC`, text `#B21D1D` |
| Queued (violet) | `#7C3AED`; bg `#F4F1FB`, text `#5B32B8` |

### Color — dark skin (option 1i)
Window bg `#12171A`; titlebar/footer `#171D21`; border strong `#262F35`; row divider `#1D2429`; checkbox border `#333E45`; track `#212A2F`; text primary `#DEE5E9`; secondary `#9FADB6`; tertiary `#7C8A93`; muted `#67757E`. Accent `#0A84FF`, accent tint `rgba(10,132,255,.13)`, in-flight segment `rgba(10,132,255,.35)`, badge text `#6FB4FF`. Status badges: success `rgba(14,159,110,.15)` / `#3FCF96`; queued `rgba(124,58,237,.16)` / `#A98BF5`; error `rgba(220,38,38,.15)` / `#F07575`; paused `#212A2F` / `#8A97A1`.

### Color — paper/amber skin (option 1j)
Page `#FBF9F5`; titlebar `#F4F0E8`; rule strong `#2B2621` (1.5px); divider `#EBE4D7`; border `#DFD8CB` / `#E2DACB`; ink `#2B2621`; secondary `#5C5347`; muted `#9C9280`. Accent amber `#C98A21`, text amber `#8A5A11`, amber tint `#F6E7CB`. Status badges are outlined (1px border in the text color), not filled. Squared corners throughout (radius 0–3px).

### Typography
Two families, both Google Fonts:
- **IBM Plex Sans** — 400/500/600/700. All UI labels, file names, buttons, headings.
- **IBM Plex Mono** — 400/500/600. Every number, size, speed, ETA, percentage, hostname, path, aria2 flag, column header, and status badge. This split is the core typographic rule of the design: **language is Sans, machine values are Mono.**

Scale as used:
| Use | Font / size / weight | Extra |
|---|---|---|
| Dialog title | Sans 15 / 600 | |
| Panel title | Sans 14 / 600 | |
| Detail file name | Sans 13.5 / 600 | `word-break: break-all` |
| Sidebar item, list file name | Sans 12.5 / 500–600 | |
| Toolbar button | Sans 12–12.5 / 500–600 | |
| Body / helper text | Sans 11.5 / 400, color `#6B7580` | |
| Small label | Sans 10.5–11 / 400 | |
| Column header | Mono 10 / 600, `letter-spacing: .07em`, uppercase, `#8B959D` | |
| Section label | Mono 9.5–10 / 600, `letter-spacing: .07–.09em`, uppercase | |
| Table numeric cell | Mono 11.5 / 400–500 | right-aligned |
| Mono meta (host, path, flag) | Mono 10–11 / 400, `#9AA5AE` | |
| Status badge | Mono 10 / 600, `letter-spacing: .04em` | |
| Micro (lane no., legend) | Mono 8–9.5 / 400 | |

### Spacing, radius, elevation
- Spacing steps in use: 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22 px. Row padding `11px 14px`; panel padding `18px 20px`; header padding `16–18px 20px`.
- Grid gaps: 12px between table columns, 10–14px between form fields, 2–3px between segment bars, 2px between block-map cells.
- Radius: window/card 10; button & field 6–7; badge & pill 5 (20 for the status pill in the titlebar); checkbox 4; progress track 3–4; block cell 1–2.
- Shadow: card `0 2px 10px rgba(0,0,0,.07)`. Focused input ring: `0 0 0 3px rgba(0,122,255,.12)` with a `1.5px solid #007AFF` border.
- Borders are always 1px except the paper skin's 1.5px rules.

## Screens / Views

### 1. Main window (option `1a`) — 1080px wide reference
**Purpose:** the app's home. Monitor and control all downloads.

**Layout:** vertical stack — titlebar (38px) → toolbar (auto, ~10px padding) → body (flex row, min-height 430px) → the body's right column ends in a sticky status footer.
- Body left: sidebar, `width: 196px`, `flex: none`, bg `#FAFBFB`, right border `#E3E6E8`, padding `14px 10px`, `display:flex; flex-direction:column; gap:2px`.
- Body right: `flex:1; min-width:0`, column flex holding the table header, the rows, and the footer (`margin-top:auto`).

**Titlebar** — height 38, bg `#F2F4F5`, bottom border `#E3E6E8`, padding `0 12px`, gap 10.
- Left: three 11px circles, `#D8DCDF`, gap 7 (OS-neutral placeholder — replace with real window controls per platform).
- Center: "Aria2 Manager", Sans 12/500, `#6B7580`, `letter-spacing:.02em`, `flex:1; text-align:center`.
- Right: connection pill — "RPC connected", Mono 10.5/500, `#007AFF` on `#E5F0FF`, padding `3px 8px`, radius 20. When disconnected this should turn danger-colored and read the failure reason.

**Toolbar** — bg white, bottom border `#E3E6E8`, padding `10px 12px`, `display:flex; align-items:center; gap:8px`.
- Primary: "＋ Add URL" — bg `#007AFF`, white text Sans 12.5/600, padding `7px 13px`, radius 7, inner gap 7.
- 1px × 22px dividers `#E3E6E8` with `margin: 0 3px` between groups.
- Secondary buttons — 1px `#DCE0E3` border, radius 7, padding `7px 11px`, Sans 12/500, `#3D464E`: "▶ Resume", "❙❙ Pause", "✕ Remove" | "⌗ Queues", "◴ Scheduler".
- Right: filter field, width 210, bg `#F4F6F7`, border `#E3E6E8`, radius 7, padding `7px 10px`, "⌕" glyph + placeholder "Filter downloads" in `#9AA5AE`.
- Resume/Pause/Remove are disabled (50% opacity, no pointer) when no row is selected.

**Sidebar** — items are `display:flex; align-items:center; gap:9px; padding:7px 9px; radius:6`, Sans 12.5/400, `#3D464E`, with a right-aligned Mono 11 count in `#9AA5AE`. Selected item: bg `#E5F0FF`, text `#0A4C9E` weight 600, count `#007AFF`.
- Section label "CATEGORIES": All downloads (8, selected) · Downloading (3, `↓` in accent) · Paused (1, `❙❙` in `#9AA5AE`) · Completed (2, `✓` in `#0E9F6E`) · Failed (1, `!` in `#DC2626`).
- Section label "QUEUES" (20px top padding): Main queue (`#007AFF` 7px swatch, radius 2) · Overnight (`#D97706`) · Datasets (`#7C3AED`).
- Bottom (`margin-top:auto`): speed-limiter card — white, 1px `#E3E6E8`, radius 8, padding 12. Row: "Speed limiter" Sans 11/600 `#3D464E` + "ON" Mono 10.5/500 accent. Then a 5px track `#E7EBED` radius 3 with a 62% accent fill and a 13px round white knob (2px accent border) centered on the fill edge. Then "25 MB/s max · 16 conn/file" Mono 10.5 `#6B7580`.

**Download table** — a CSS grid repeated per row:
`grid-template-columns: 26px minmax(0,1fr) 82px 168px 92px 74px 96px; gap: 12px; align-items: center;`
- Header: padding `9px 14px`, bg `#FAFBFB`, bottom border `#E3E6E8`. Labels: (blank) · FILE · SIZE (right) · PROGRESS · SPEED (right) · ETA (right) · STATUS.
- Row: padding `11px 14px`, bottom border `#F0F2F3`.
  - Col 1: 16px checkbox, 1px `#D3D8DC`, radius 4, white.
  - Col 2: file name Sans 12.5/500 `#14181B`, single-line ellipsis; below it hostname Mono 10.5 `#9AA5AE`, ellipsis, `margin-top:2px`. Both need `min-width:0` on the cell for ellipsis to work inside grid.
  - Col 3: size, Mono 11.5 `#3D464E`, right.
  - Col 4: **inline segment bar** (see below) + a caption row `display:flex; justify-content:space-between; margin-top:4px`, Mono 10 `#9AA5AE`, showing "42%" left and "16 conn" right (right side empty unless downloading).
  - Col 5: speed, Mono 11.5/500, accent `#007AFF` when active, `#B4BCC2` when the value is "—".
  - Col 6: ETA, Mono 11.5 `#6B7580`, right.
  - Col 7: status badge — inline-block, padding `3px 8px`, radius 5, Mono 10/600 `letter-spacing:.04em`. Downloading `#E5F0FF`/`#0A4C9E`; Paused `#F0F2F3`/`#6B7580`; Completed `#E8F7F0`/`#0B6B4B`; Queued `#F4F1FB`/`#5B32B8`; Error `#FDECEC`/`#B21D1D`.
- Row hover: `#FAFBFB`. Row selected: `#E5F0FF`. Double-click opens the detail view. Right-click opens a context menu (Pause, Resume, Move to queue ▸, Open folder, Copy URL, Remove, Remove with file).

**Inline segment bar:** `display:flex; height:6px; gap:1px; border-radius:3px; overflow:hidden; background:#EBEEF0`, one `flex:1` child per connection. A child is `#007AFF` when its slice is fully downloaded, `#7FB4FF` when it is the in-flight slice, `#EBEEF0` when pending. With N connections, slice *i* covers `(i+0.5)/N * 100`% of the file — the mock maps overall percent onto that so the bar fills left-to-right; a real implementation should instead map each aria2 connection's own byte range.

**Status footer** — `margin-top:auto`, padding `9px 14px`, top border `#E3E6E8`, bg `#FAFBFB`, `display:flex; align-items:center; gap:16px`.
- "24.5" Mono 13/600 accent + "MB/s down" Mono 10.5 `#6B7580`.
- A 150×24 sparkline: 1.5px accent polyline over a `fill-opacity: .1` accent area, 16 samples.
- Right: "3 active · 48 connections · 12.1 GB remaining", Mono 11 `#6B7580`.

**Mock rows (exact copy used):**
| File | Host | Size | % | Speed | ETA | Status | Conns |
|---|---|---|---|---|---|---|---|
| ubuntu-24.04.2-desktop-amd64.iso | releases.ubuntu.com | 6.10 GB | 42 | 11.4 MB/s | 07:12 | Downloading | 16 |
| Blender-4.5.0-linux-x64.tar.xz | mirror.clarkson.edu | 341 MB | 78 | 8.2 MB/s | 00:09 | Downloading | 16 |
| project-backup-2026-09-01.zip | s3.eu-central-1.amazonaws.com | 1.80 GB | 63 | 4.9 MB/s | 02:14 | Downloading | 8 |
| imagenet-shard-03.tar | data.vision.ee.ethz.ch | 12.4 GB | 9 | — | — | Paused | 8 |
| macos-recovery-image.dmg | swcdn.apple.com | 4.20 GB | 0 | — | — | Queued | 16 |
| nvidia-driver-580.11.run | us.download.nvidia.com | 412 MB | 100 | — | — | Completed | 8 |
| postgresql-17.2.tar.bz2 | ftp.postgresql.org | 28.4 MB | 100 | — | — | Completed | 4 |
| archive-photos-2019.tar.gz | cdn.fileshare.io | 9.02 GB | 0 | — | — | Error | 8 |

### 2. Add download dialog (option `1b`) — 520px wide
**Purpose:** paste a URL, confirm destination and connection count, optionally reach aria2 flags.

**Header** — padding `18px 20px 14px`, bottom border `#E3E6E8`. Title "Add download" Sans 15/600. Sub "Paste one or more URLs. Mirrors on separate lines download in parallel." Sans 11.5 `#6B7580`, `margin-top:3px`.

**Body** — padding `18px 20px`, `display:flex; flex-direction:column; gap:16px`.
1. **URL** — section label Mono 10/600 `.07em` `#8B959D`, 6px below it a focused textarea: padding `10px 11px`, 1.5px `#007AFF` border, radius 7, focus ring `0 0 0 3px rgba(0,122,255,.12)`, content Mono 11.5/1.6 `#14181B`, `word-break: break-all`. Sample value `https://releases.ubuntu.com/24.04/ubuntu-24.04.2-desktop-amd64.iso`. Below, 7px gap: resolve result "✓ Resolved · 6.1 GB · server supports ranges" Sans 11 `#0E9F6E`. If the server does not support ranges, this line turns `#D97706` and the connections control locks to 1 with the note "server does not support ranges — single connection".
2. **Save to / Queue** — `grid-template-columns: minmax(0,1fr) 96px; gap: 12px`. Save-to field: 1px `#DCE0E3`, radius 7, padding `9px 11px`, path in Mono 11.5 `#3D464E` (ellipsis), trailing "Browse" Sans 11/500 accent. Queue field: same box, Sans 11.5 `#3D464E`, trailing "⌄" `#9AA5AE`.
3. **Connections** — header row: "CONNECTIONS" section label + current value "16" Mono 11.5/600 accent. Control: a 26px-tall row of 32 bars, `display:flex; gap:3px; align-items:flex-end`, bar *i* height `8 + round(i*0.55)`px, radius 2, `#007AFF` up to the selected index and `#E7EBED` after — a click/drag target that doubles as its own scale. Below, `justify-content:space-between` Mono 10 `#9AA5AE`: "1" and "32 max (server-limited)".
4. **Advanced (aria2 options)** — separated by a top border `#EEF0F1` with 14px padding. Header row "⌄ Advanced (aria2 options)" Sans 11.5/500 `#3D464E`, disclosure caret `#9AA5AE`. Content (expanded in the mock): a 2-column grid, `gap: 9px 14px`, each item `justify-content:space-between`, 7px bottom padding, `1px dashed #E9ECEE` bottom border; flag name Mono 11 `#6B7580`, value Mono 11/500 `#14181B`. Items: `--split` 16, `--min-split-size` 10M, `--max-tries` 5, `--file-allocation` falloc. Values are editable in place; the set shown should reflect any option overridden from the global default.

**Footer** — padding `14px 20px`, bg `#FAFBFB`, top border `#E3E6E8`, flex with gap 10. Left: "Start later ◴" Sans 11 `#6B7580` (opens the scheduler picker). Right: "Cancel" (white, 1px `#DCE0E3`, radius 7, padding `8px 14px`, Sans 12/500 `#3D464E`) and "Start download" (accent fill, white, padding `8px 16px`, Sans 12/600).

### 3. Detail view — three progress treatments
All three share a 560px card and the same header: padding `16px 20px 14px`, bottom border `#E3E6E8`; file name Sans 13.5/600 `#14181B` with `word-break: break-all`; a meta row 8px below, `display:flex; gap:14px`, Mono 11 `#6B7580`, with the speed in accent.

**3a. Connection lanes (option `1c`)** — one row per aria2 connection. Meta adds "16 lanes".
- Section label "CONNECTION LANES", 12px below it a `flex-direction:column; gap:5px` list.
- Row grid: `22px minmax(0,1fr) 62px; gap:10px; align-items:center`.
  - Lane number, zero-padded ("01"), Mono 9.5 `#B4BCC2`, right-aligned.
  - Track: 9px tall, radius 2, bg `#F0F2F3`, `position:relative; overflow:hidden`. Inside it, an absolutely positioned fill starting at that lane's byte-range offset (`left: i * 6.25%` for 16 equal lanes) with the downloaded width, colored `#007AFF` (or `#E8B04B` if the lane is stalled), plus a 2px `#14181B` at 35% opacity playhead at the fill's leading edge. **This is the treatment that shows aria2's actual split geometry — each lane starts where its range starts, not at zero.**
  - Rate, Mono 9.5, right: `#6B7580` normally, "stalled" in `#D97706`.
- Summary strip: 18px above, 14px top padding, top border `#EEF0F1`, three equal columns each with a Mono 9.5/600 `#9AA5AE` `.06em` label and a Mono 12/500 value — FASTEST LANE 1.4 MB/s · STALLED 2 lanes (`#D97706`) · RETRIES 3.

**3b. Block map (option `1d`)** — the IDM-style mosaic. Meta adds "1024 blocks · 6 MB each".
- Header row: "BLOCK MAP" label + a legend, `display:flex; gap:12px`, Mono 9.5 `#9AA5AE`, each entry an 8px radius-1 swatch + word: done `#007AFF`, active `#7FB4FF`, pending `#E7EBED`.
- Grid: `repeat(32, 1fr)`, `gap:2px`, padding 10, bg `#FAFBFB`, 1px `#EDF0F1`, radius 6. Cells are `aspect-ratio:1`, radius 1. In the mock 512 cells stand in for 1024 blocks; production should aggregate the real bitfield down to whatever cell count fits (aria2's `tellStatus` returns a hex `bitfield` — OR the bits per cell).
- Below, 16px gap: caption row Mono 10.5 `#6B7580` — "42% complete" / "430 / 1024 blocks" — then an 8px overall bar, radius 4, `#E7EBED` track, accent fill.

**3c. Ribbon + throughput (option `1e`)** — the calmest option.
- "FILE RANGE 0 → 6.10 GB" label, then a 34px-tall ribbon: radius 5, bg `#F0F2F3`, 1px `#E7EBED`, `overflow:hidden`; 16 equal `flex:1` cells each with a `1px solid #fff` right divider and an absolutely positioned left-anchored fill (accent, or `#E8B04B` for a stalled segment) whose width is that segment's completion percent. Caption Mono 10 `#9AA5AE`: "16 segments" / "each segment ≈ 390 MB".
- "THROUGHPUT · LAST 60 s" label 22px below, then a full-width 88px SVG (`viewBox="0 0 500 88"`, `preserveAspectRatio="none"`), bg `#FAFBFB`, 1px `#EDF0F1`, radius 6, with three horizontal gridlines `#E7EBED` at y = 22/44/66, a `fill-opacity:.12` accent area and a 2px accent polyline over 21 samples. Caption: "peak 14.8 MB/s" / "avg 9.6 MB/s".

**Recommendation:** ship `1c` as the default detail view (it is the only one that reflects aria2's real split model), and offer `1d` as a toggle for users who expect the IDM mosaic. `1e`'s throughput chart is worth keeping as a strip under either.

### 4. Browser capture popup (option `1f`) — 360px wide
**Purpose:** the extension's popup after intercepting downloadable links on a page.
- Header: padding `13px 15px`, bottom border `#E3E6E8`, gap 9. A 22px accent square, radius 6, containing "a2" in white Mono 11/700. Title "4 links captured" Sans 12.5/600. A right-aligned "✕" Mono 11 `#9AA5AE`.
- List: `6px 0` vertical padding; each item padding `9px 15px`, gap 10. A 15px checkbox, radius 4 — checked = accent fill, accent border, white "✓" at 9px; unchecked = white on `#D3D8DC`. Then name Sans 11.5/500 `#14181B` (ellipsis) over MIME meta Mono 10 `#9AA5AE`. Right: size Mono 10.5 `#6B7580`.
- Items: ubuntu-24.04.2-desktop-amd64.iso / "application/octet-stream · ranges ok" / 6.10 GB / checked · ubuntu-24.04.2-desktop-amd64.iso.torrent / "application/x-bittorrent" / 312 KB / unchecked · SHA256SUMS / "text/plain" / 1.4 KB / checked · SHA256SUMS.gpg / "application/pgp-signature" / 833 B / checked.
- Footer: padding `12px 15px`, bg `#FAFBFB`, top border `#E3E6E8`. Left "→ Main queue" Mono 10.5 `#9AA5AE` (a queue picker). Right "Options" (outlined, radius 6, padding `7px 12px`, Sans 11.5/500) and "Download 3" (accent fill, padding `7px 13px`, Sans 11.5/600) — the count tracks the checked items and the button disables at zero.

### 5. Queues & scheduler (option `1g`) — 660px wide
- Header: title "Queues & scheduler" Sans 14/600; sub "Queues run one at a time within their window. Bandwidth is shared across active queues." Sans 11.5 `#6B7580`.
- Body padding `18px 20px`, `flex-direction:column; gap:10px`.
- **Queue cards** — grid `12px minmax(0,1fr) 96px 120px 74px; gap:14px; align-items:center`, padding `12px 14px`, 1px `#E7EBED`, radius 8, white.
  - 9px radius-3 color swatch · name Sans 12.5/600 over detail Mono 10.5 `#9AA5AE` · window Mono 11 `#3D464E` · a 5px progress track `#EDF0F1` radius 3 with a queue-colored fill and a Mono 10 `#9AA5AE` caption "47% of queue" 4px below · a right-aligned state badge (Mono 10/600, radius 5, padding `3px 8px`).
  - Rows: **Main queue** — "3 running · 2 waiting · no limit", Always on, 47%, `#007AFF`, RUNNING (`#E5F0FF`/`#0A4C9E`). **Overnight** — "4 waiting · 5 MB/s cap", 01:00 – 07:00, 0%, `#D97706`, WAITING (`#FDF3E4`/`#8A5A11`). **Datasets** — "1 running · 12 waiting · 2 at a time", Weekends, 22%, `#7C3AED`, PAUSED (`#F4F1FB`/`#5B32B8`).
  - Cards are drag-reorderable; order sets priority.
- **Weekly window grid** — label "WEEKLY WINDOW · OVERNIGHT". Grid `34px repeat(24, 1fr); gap:2px; align-items:center`. First row: a blank cell then hour labels (Mono 8 `#B4BCC2`, centered, shown only every 3rd hour). Then 7 rows MON–SUN: day label Mono 9.5 `#8B959D`, then 24 cells 14px tall, radius 2. Fill rules in the mock: hours 1–6 = `#007AFF` (full speed); otherwise weekends and weekday hours ≥19 = `#B3D4FF` (throttled); else `#EDF0F1` (paused). Cells are click-and-drag paintable, cycling paused → throttled → full.
- Legend: `gap:14px`, Mono 10 `#9AA5AE`, 9px radius-2 swatches — full speed / throttled 5 MB/s / paused.

### 6. Settings → Engine (option `1h`) — 660px wide, min-height 400
- Left nav: `width:168px`, bg `#FAFBFB`, right border `#E3E6E8`, padding `16px 10px`. Label "SETTINGS". Items padding `7px 9px`, radius 6, Sans 12 `#3D464E`: General · Downloads & folders · Browser integration · **Engine (aria2)** (selected: `#E5F0FF` bg, `#0A4C9E`, weight 600) · Proxies · Notifications.
- Right pane padding `18px 20px`.
  - **Status banner** — padding `11px 13px`, bg `#F0FAF6`, 1px `#CBEBDD`, radius 8, `margin-bottom:20px`. An 8px `#0E9F6E` dot, "Connected to aria2 1.37.0" Sans 11.5/500 `#0B6B4B`, right-aligned "local · pid 4821" Mono 10.5 `#3F8E72`. Error state: `#FDECEC` / `#F5C9C9` / `#B21D1D` with the RPC error and a Retry action.
  - **RPC ENDPOINT** — grid `1fr 108px; gap:12px`: Host `127.0.0.1`, Port `6800`. Fields: 1px `#DCE0E3`, radius 6, padding `8px 10px`, Mono 11.5 `#14181B`; each with a Sans 10.5 `#6B7580` label 5px above. Then full-width "Secret token": masked `••••••••••••••••` Mono 11.5 with a trailing "Reveal" Sans 10.5/500 accent.
  - **LIMITS** — a list where each row is `display:flex; align-items:center; gap:12px; padding:11px 0`, bottom border `#F0F2F3`. Left: human label Sans 11.5/500 `#14181B` over the raw flag Mono 10 `#9AA5AE`. Right: a value input, 1px `#DCE0E3`, radius 6, padding `5px 10px`, Mono 11/500, `min-width:64px`, right-aligned text.
    | Label | Flag | Value |
    |---|---|---|
    | Max concurrent downloads | `--max-concurrent-downloads` | 3 |
    | Connections per server | `--max-connection-per-server` | 16 |
    | Minimum split size | `--min-split-size` | 10M |
    | Global download limit | `--max-overall-download-limit` | 25M |
    | Disk cache | `--disk-cache` | 64M |
    | File allocation | `--file-allocation` | falloc |
  - **Actions** — 18px above: "Apply" (accent fill, padding `8px 14px`, radius 7, Sans 12/600), "Restart engine" (outlined), and a right-aligned Mono 10.5 `#9AA5AE` hint "aria2.changeGlobalOption" naming the RPC method that will fire. Options that cannot be changed at runtime must trigger a restart-required note instead.

### 7. Style skins (options `1i`, `1j`)
Two alternative visual treatments of the same download table, for the theme decision.
- **`1i` dark engine-room** — same grid, tokens from the dark palette above; column set drops ETA (`24px minmax(0,1fr) 76px 150px 84px 88px`); status text is uppercased; titlebar's right pill shows live speed rather than connection state; footer reads "3 active · 48 connections" with a right-aligned accent "aria2 1.37.0".
- **`1j` paper & amber** — near-monochrome print aesthetic. No traffic lights; the titlebar is a Mono 11.5/600 `.06em` wordmark "ARIA2 MANAGER" with a squared amber speed chip. The header rule and footer rule are 1.5px solid ink `#2B2621`. No checkbox column. Progress is a 12px-tall row of **bottom-aligned bars** (`align-items:flex-end`, gap 2, completed bars 12px tall, pending 4px) rather than a continuous track — completed ink `#2B2621`, in-flight amber `#C98A21`, pending `#DFD8CB`. Status badges are outlined, squared, uppercase.

## Interactions & Behavior
- **Add URL** opens the dialog (screen 2). Paste triggers a HEAD/probe: while pending, show a neutral "Resolving…" line; on success show size + range support; on failure show the error and keep Start enabled (aria2 will retry).
- **Row selection**: single click selects, cmd/ctrl-click and shift-click extend, toolbar actions apply to the selection. Double-click on a completed row opens the file; on an active row opens the detail view.
- **Pause/Resume** map to `aria2.pause` / `aria2.unpause`; **Remove** to `aria2.remove` + `aria2.removeDownloadResult`, with a confirm when "also delete file" is chosen.
- **Live updates**: poll `aria2.tellActive` / `tellWaiting` / `tellStopped` on a ~1s interval (or subscribe over the RPC WebSocket). Segment bars, speeds, ETA and the sparkline animate with a 200–300ms linear width transition so they read as continuous rather than stepping. Never animate row reordering during a poll tick — reorder only on explicit sort or state change.
- **Sorting**: click a column header to sort; the active header shows a small caret and switches to `#14181B`.
- **Drag and drop** a URL or a `.txt` of URLs onto the window opens the add dialog pre-filled.
- **Speed limiter slider** writes `--max-overall-download-limit` live via `aria2.changeGlobalOption`; debounce ~300ms.
- **Scheduler grid** paint-drag sets per-hour policy; changes take effect at the next hour boundary.
- **Hover**: buttons darken their border to `#C9CFD4`; the primary button darkens to `#0060DF`; rows tint `#FAFBFB`. Focus rings follow the input pattern (`0 0 0 3px rgba(0,122,255,.12)`).
- **Empty state** (no downloads): centered, the "＋ Add URL" affordance repeated with the line "Paste a URL, or drop a link here." — not designed in the mocks; build it in the same type scale.

## State Management
- `engine`: `{ status: 'connected'|'connecting'|'error', version, pid, host, port, secret }`.
- `downloads[]`: `{ gid, name, host, url, totalLength, completedLength, downloadSpeed, connections, status, errorCode, dir, bitfield, files[] }` — straight from `aria2.tellStatus`.
- `selection`: Set of gids. `filter`: category + search string. `sort`: `{ column, dir }`.
- `queues[]`: `{ id, name, color, window, maxActive, speedCap, state }` — aria2 has no native queue concept, so queues are an app-level layer that decides which gids are unpaused at any moment.
- `schedule`: 7 × 24 array of `'off'|'throttled'|'full'`; a timer at each hour boundary applies the policy.
- `globalOptions`: mirrors `aria2.getGlobalOption`; dirty fields enable Apply.
- `capturedLinks[]` (extension side): `{ url, name, mime, size, supportsRanges, checked }`.

## Assets
No images or icon files. Every glyph in the mocks is a Unicode character used as a placeholder: `＋ ▶ ❙❙ ✕ ⌗ ◴ ⌕ ≡ ↓ ✓ ! ⌄ →`. **Replace all of these with the target codebase's icon set** (Lucide, SF Symbols, Fluent, etc.) at a 14–16px optical size in the color specified for each context. Fonts: IBM Plex Sans and IBM Plex Mono, both open source (SIL OFL) — bundle them locally rather than loading from a CDN in a desktop app.

## Open gaps for the implementer
Not covered by the mocks and worth deciding early: empty and first-run states, the completed/history view, the media sniffer, the tray widget, error-detail presentation for a failed download, checksum verification UI, and responsive behavior below ~900px window width.

## Files
- `Aria2 Download Manager.dc.html` — all screens and skins, each in a labeled block with a visible id badge (`1a` … `1j`). The mock data lives in the logic class at the bottom of the file.
- `support.js` — the prototype runtime. Needed only to open the HTML locally; not part of the design.

Open the HTML file directly in a browser to inspect the designs, measure, and sample colors.
