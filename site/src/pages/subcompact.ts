import { subcompact } from '../generated/components'
import { themes } from '../generated/theme'
import { icon } from '../icons'
import { bookBlobUrl } from '../links'
import { mountBehaviors } from '../ui/behaviors'
import { ui } from '../ui/icons'
import { installUiTokens, subcompactPalettes } from '../ui/tokens'
import type { SubcompactPalette } from '../ui/tokens'
import { docsSidebar, docsTopbar, mountDocsNav } from './docs'
import '../ui/subcompact.css'

const base = import.meta.env.BASE_URL || '/'

const defaultPalette: SubcompactPalette = 'subcompact-phosphor'

const paletteLabel = (key: SubcompactPalette) => key.replace('subcompact-', '')

// The same scene as examples/subcompact_console, drawn with the web primitives.
const processes = [
  ['4021', 'ingestd', 'svc', '38.2', '1.9G', 'run'],
  ['4188', 'postgres', 'db', '21.7', '6.4G', 'run'],
  ['3310', 'edge-proxy', 'net', '12.4', '412M', 'run'],
  ['5102', 'indexer', 'svc', '9.8', '2.2G', 'run'],
  ['2207', 'metricsd', 'obs', '4.1', '188M', 'sleep'],
  ['6650', 'backup', 'ops', '2.6', '96M', 'wait'],
  ['1093', 'sshd', 'root', '0.3', '12M', 'sleep'],
  ['7731', 'cron', 'root', '0.1', '4M', 'sleep'],
  ['8012', 'rotate-logs', 'ops', '0.0', '2M', 'stop'],
]

const log: [string, 'info' | 'ok' | 'warn' | 'err', string][] = [
  ['12:04:01.112', 'info', 'scheduler: 14 jobs queued on worker pool `edge-3`'],
  ['12:04:01.287', 'ok', 'deploy: build 7f3c9a2 promoted to canary (3/24 nodes)'],
  ['12:04:02.004', 'info', 'net: 1,284 open sockets, 0.4% retransmit'],
  ['12:04:02.519', 'warn', 'disk: /var/log at 81% on ingest-02'],
  ['12:04:03.770', 'info', 'cache: hit ratio 97.2% over the last 60 s'],
  ['12:04:04.031', 'err', 'auth: 3 failed logins for `svc-backup` from 10.4.2.19'],
  ['12:04:04.588', 'ok', 'health: all probes green on cluster `fra-1`'],
]

const levelTag = { info: 'INFO', ok: ' OK ', warn: 'WARN', err: 'ERR ' }

const escape = (text: string) => text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')

const treeRow = (depth: number, label: string, opts: { open?: boolean; leaf?: boolean; active?: boolean } = {}) =>
  `<div class="ui-menu__item${opts.active ? ' is-active' : ''}" role="treeitem" aria-level="${depth + 1}"${opts.leaf ? '' : ` aria-expanded="${!!opts.open}"`}${opts.active ? ' aria-selected="true"' : ''} style="padding-left:calc(var(--sc-space-small) + ${depth} * var(--sc-space-large))">
    <span style="width:var(--sc-space-large);display:inline-grid">${opts.leaf ? '' : ui(opts.open ? 'chevron-down' : 'chevron-right')}</span>${label}</div>`

const meter = (label: string, pct: number) =>
  `<div class="sc-meter"><span class="ui-muted">${label}</span><div class="ui-progress" role="progressbar" aria-label="${label}" aria-valuenow="${pct}" aria-valuemin="0" aria-valuemax="100"><i style="width:${pct}%"></i></div><span class="is-num">${pct}%</span></div>`

function consoleScene(palette: SubcompactPalette) {
  const table = `<table class="ui-table" aria-label="Processes">
    <thead><tr><th scope="col">PID</th><th scope="col">NAME</th><th scope="col">USER</th><th scope="col" class="is-num" aria-sort="descending">CPU% ▾</th><th scope="col" class="is-num">RSS</th><th scope="col">STATE</th></tr></thead>
    <tbody>${processes
      .map(
        ([pid, name, user, cpu, mem, state]) =>
          `<tr${pid === '4188' ? ' aria-selected="true"' : ''}><td class="is-num">${pid}</td><td>${name}</td><td>${user}</td><td class="is-num">${cpu}</td><td class="is-num">${mem}</td><td>${state}</td></tr>`,
      )
      .join('')}</tbody>
  </table>`
  return `<div class="ui ui--subcompact sc-console" data-ui-theme="${palette}" data-subcompact-console role="region" aria-label="Subcompact console preview">
    <div class="sc-bar">
      <span class="sc-brand">mkit://ops</span>
      <nav class="sc-crumbs" aria-label="Location">fra-1 / ingest-02 / <b aria-current="page">procs</b></nav>
      <span class="sc-spacer"></span>
      <span class="ui-badge ui-badge--outline"><span class="sc-ok">●</span>12 up</span>
      <span class="ui-badge ui-badge--outline"><span class="sc-warn">●</span>1 degraded</span>
      <button class="ui-btn ui-btn--outline" type="button">Restart</button>
      <button class="ui-btn" type="button">Deploy</button>
    </div>
    <div class="sc-body">
      <section class="sc-pane" aria-label="Hosts">
        <div class="sc-pane-title">HOSTS</div>
        <div role="tree" aria-label="Hosts" style="padding:var(--sc-space-xsmall) 0">
          ${treeRow(0, 'fra-1', { open: true })}
          ${treeRow(1, 'edge-3', { leaf: true })}
          ${treeRow(1, 'ingest-02', { leaf: true, active: true })}
          ${treeRow(1, 'db-01', { leaf: true })}
          ${treeRow(0, 'iad-2')}
          ${treeRow(0, 'bastion', { leaf: true })}
        </div>
      </section>
      <section class="sc-pane" aria-label="Processes">
        <div class="sc-pane-body" style="padding-bottom:0">
          <div class="ui-tabs__list" role="tablist" aria-label="Console views">
            <button class="ui-tabs__trigger" role="tab" aria-selected="true" type="button">procs</button>
            <button class="ui-tabs__trigger" role="tab" aria-selected="false" tabindex="-1" type="button">sockets</button>
            <button class="ui-tabs__trigger" role="tab" aria-selected="false" tabindex="-1" type="button">env</button>
          </div>
        </div>
        <div class="sc-scroll">${table}</div>
      </section>
      <section class="sc-pane" aria-label="Tuning">
        <div class="sc-pane-title">TUNING</div>
        <div class="sc-pane-body">
          <div class="sc-form">
            <label class="ui-label" for="sc-host-${palette}">Host</label>
            <input class="ui-input" id="sc-host-${palette}" value="ingest-02.fra-1" spellcheck="false">
            <span class="ui-label">Logs</span>
            <label class="ui-choice"><button class="ui-switch" role="switch" aria-checked="true" aria-label="Verbose" type="button"></button>verbose</label>
            <span class="ui-label">Tail</span>
            <label class="ui-choice"><button class="ui-check" role="checkbox" aria-checked="true" aria-label="Follow tail" type="button">${ui('check')}${ui('minus')}</button>follow</label>
            <label class="ui-label" for="sc-rate-${palette}">Rate</label>
            <div class="ui-row" style="flex-wrap:nowrap"><input class="ui-slider" type="range" id="sc-rate-${palette}" min="50" max="1000" step="50" value="250" aria-label="Sample rate"><span class="is-num" data-ui-output="sc-rate-${palette}" data-unit="ms" style="min-width:4em;text-align:right">250ms</span></div>
          </div>
        </div>
        <div class="sc-pane-title">LOAD</div>
        <div class="sc-pane-body">${meter('CPU', 62)}${meter('Memory', 81)}${meter('Disk', 44)}</div>
      </section>
    </div>
    <section class="sc-pane sc-log" aria-label="Log">
      <div class="sc-pane-title">LOG  tail -f /var/log/ops.log</div>
      <div class="sc-log-lines">${log
        .map(
          ([time, level, message]) =>
            `<div class="sc-log-line"><span class="sc-time">${time}</span><span class="sc-${level}">[${levelTag[level]}]</span><span>${escape(message)}</span></div>`,
        )
        .join('')}</div>
    </section>
    <div class="sc-keys"><span><kbd class="ui-kbd">⌘</kbd><kbd class="ui-kbd">K</kbd> palette</span><span><kbd class="ui-kbd">/</kbd> filter</span><span><kbd class="ui-kbd">⌃</kbd><kbd class="ui-kbd">R</kbd> restart</span></div>
    <div class="sc-status" role="status" aria-label="Console status">
      <span class="sc-mode">NORMAL</span><span>ingest-02.fra-1</span>
      <span class="ui-row" style="gap:var(--sc-space-small)">sync <span class="ui-progress"><i style="width:72%"></i></span></span>
      <span class="sc-spacer"></span><span>rtt 3.1 ms</span><span>12:04:04 UTC</span>
    </div>
  </div>`
}

// One small settings form drawn at the default density and at subcompact density.
function settingsForm(prefix: string) {
  return `<div class="ui-col">
    <div class="ui-field"><label class="ui-label" for="${prefix}-name">Name</label><input class="ui-input" id="${prefix}-name" value="ingest-02"></div>
    <div class="ui-field"><label class="ui-label" for="${prefix}-region">Region</label><input class="ui-input" id="${prefix}-region" value="fra-1"></div>
    <label class="ui-choice"><button class="ui-switch" role="switch" aria-checked="true" aria-label="Autoscale" type="button"></button>Autoscale</label>
    <label class="ui-choice"><button class="ui-check" role="checkbox" aria-checked="false" aria-label="Pin version" type="button">${ui('check')}${ui('minus')}</button>Pin version</label>
    <div class="ui-row"><button class="ui-btn ui-btn--outline" type="button">Cancel</button><button class="ui-btn" type="button">Apply</button></div>
  </div>`
}

function comparison() {
  const dense = themes[defaultPalette]
  const roomy = themes['shadcn-dark']
  return `<div class="sc-compare">
    <figure class="sc-compare-cell">
      <div class="ui sc-compare-canvas" data-ui-theme="dark">${settingsForm('cmp-default')}</div>
      <figcaption>Default density · <code>controls.medium</code> ${roomy.controls.medium} px · body ${roomy.typography.body} px</figcaption>
    </figure>
    <figure class="sc-compare-cell">
      <div class="ui ui--subcompact sc-compare-canvas" data-ui-theme="${defaultPalette}">${settingsForm('cmp-dense')}</div>
      <figcaption>Subcompact · <code>controls.medium</code> ${dense.controls.medium} px · body ${dense.typography.body} px</figcaption>
    </figure>
  </div>`
}

function chapterSections() {
  return subcompact.sections
    .map(
      (section) => `<section class="doc-section" id="${section.id}">
        <h3>${section.title}</h3>
        ${section.html}
      </section>`,
    )
    .join('')
}

function toc() {
  const rows = [
    `<a class="docs-toc-link" href="#subcompact--preview">Live preview</a>`,
    `<a class="docs-toc-link" href="#subcompact--compare">Density side by side</a>`,
    ...subcompact.sections.map((section) => `<a class="docs-toc-link" href="#${section.id}">${section.title}</a>`),
  ].join('')
  return `<aside class="docs-toc" aria-label="On this page">
    <span class="docs-toc-heading">On this page</span>
    ${rows}
  </aside>`
}

export function renderSubcompact() {
  return `<div class="docs-shell">
    ${docsTopbar('Subcompact')}
    <div class="docs-layout">
      ${docsSidebar('subcompact')}
      <main class="docs-main" id="main" tabindex="-1">
        <nav class="docs-breadcrumb" aria-label="Breadcrumb">
          <a href="${base}">Home</a><span aria-hidden="true">/</span><a href="${base}theming">Theming</a><span aria-hidden="true">/</span><b aria-current="page">Subcompact</b>
        </nav>
        <header class="docs-header">
          <span class="docs-kicker">Theming &amp; design language</span>
          <h1>${subcompact.title}</h1>
          <p class="docs-lede">${subcompact.lede}</p>
          <div class="docs-callout">${icon('book', 15)}<span>The console below is a <b>web preview</b> drawn from the subcompact tokens in <code>crates/mkit-core/src/subcompact.rs</code>. It is not a GPUI capture: harness screenshots of <code>examples/subcompact_console</code> are pending, because the harness captures only on macOS.</span></div>
          <a class="doc-source" href="${bookBlobUrl}/${subcompact.sourcePath}" target="_blank" rel="noreferrer">Read the book chapter<span class="sr-only"> (opens in a new tab)</span> ${icon('external', 13)}</a>
        </header>
        <article class="docs-article">
          ${subcompact.intro ? `<div class="doc-intro">${subcompact.intro}</div>` : ''}
          <section class="theme-board-section" id="subcompact--preview">
            <div class="docs-section-head">
              <span class="docs-kicker">Live preview</span>
              <h2>An ops console at subcompact density</h2>
              <p>The same scene as <code>examples/subcompact_console</code>. Switch palettes to see the three subcompact themes; the tabs, switch, checkbox, and slider respond to input.</p>
            </div>
            <div class="sc-switcher" role="group" aria-label="Subcompact palette">
              ${subcompactPalettes
                .map(
                  (key) =>
                    `<button type="button" class="sc-palette${key === defaultPalette ? ' active' : ''}" aria-pressed="${key === defaultPalette}" data-palette="${key}">${paletteLabel(key)}</button>`,
                )
                .join('')}
            </div>
            ${consoleScene(defaultPalette)}
          </section>
          <section class="theme-board-section" id="subcompact--compare">
            <div class="docs-section-head">
              <span class="docs-kicker">Density</span>
              <h2>Density side by side</h2>
              <p>One settings form, drawn with the same primitives at the default density and at subcompact density.</p>
            </div>
            ${comparison()}
          </section>
          ${chapterSections()}
        </article>
      </main>
      ${toc()}
    </div>
  </div>`
}

export function mountSubcompact() {
  installUiTokens()
  const cleanup = mountDocsNav()
  document.querySelectorAll<HTMLElement>('.ui--subcompact, .sc-compare-canvas').forEach(mountBehaviors)
  const consoleRoot = document.querySelector<HTMLElement>('[data-subcompact-console]')
  document.querySelectorAll<HTMLButtonElement>('.sc-palette').forEach((button) =>
    button.addEventListener('click', () => {
      const palette = button.dataset.palette
      if (!consoleRoot || !palette) return
      consoleRoot.dataset.uiTheme = palette
      document.querySelectorAll<HTMLButtonElement>('.sc-palette').forEach((other) => {
        const active = other === button
        other.classList.toggle('active', active)
        other.setAttribute('aria-pressed', String(active))
      })
    }),
  )
  return cleanup
}
