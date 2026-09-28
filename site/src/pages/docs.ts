import {
  everydayCards,
  kindLabel,
  previousNext,
  proCards,
  registryStatusLabel,
  statusClass,
  statusLabel,
} from '../catalog'
import { demos } from '../demos'
import { icon } from '../icons'
import { bookBlobUrl, bookSourceUrl, registryBlobUrl, repoUrl } from '../links'
import { isLightMode } from '../theme'
import { mountBehaviors } from '../ui/behaviors'
import { installUiTokens } from '../ui/tokens'
import type { CatalogCard, ComponentDoc, DocSection, RegistryContract } from '../types'

const base = import.meta.env.BASE_URL || '/'

const slugify = (value: string) =>
  value
    .toLowerCase()
    .replace(/&/g, ' and ')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

function prepareHtml(html: string) {
  return html
    .replaceAll('@book/', `${base}book/`)
    .replaceAll('href="/components/', `href="${base}components/`)
}

const draftNote = (card: CatalogCard) =>
  card.status === 'ready'
    ? '<b>Pre-release.</b> The API, keyboard map, and accessibility contract are documented but still need maintainer sign-off. Every registry entry is still <code>implementation_in_progress</code>, so none can be installed yet.'
    : '<b>Pre-release, in progress.</b> This is a draft. The API and conformance evidence will change, and the registry entry cannot be installed yet.'

function pageLink(href: string, label: string, active: boolean, meta = '') {
  return `<a class="docs-nav-link${active ? ' active' : ''}" href="${href}"${active ? ' aria-current="page"' : ''}>
    <span class="docs-nav-name">${label}</span>${meta ? `<span class="docs-nav-group">${meta}</span>` : ''}
  </a>`
}

function sidebarLink(card: CatalogCard, activeSlug: string) {
  const active = card.slug === activeSlug
  const count = card.components.length > 1 ? String(card.components.length) : ''
  const link = pageLink(`${base}components/${card.slug}`, card.name, active, count)
  if (!active || card.components.length < 2) return link
  const children = card.components
    .map((doc) => `<a class="docs-nav-sublink" href="#${componentAnchor(doc)}">${doc.name}</a>`)
    .join('')
  return `${link}<div class="docs-nav-children">${children}</div>`
}

export function docsSidebar(activeSlug: string) {
  return `<aside class="docs-sidebar" id="docs-sidebar" aria-label="Documentation">
    <div class="docs-sidebar-inner">
      <nav class="docs-nav" aria-label="Documentation pages">
        <div class="docs-nav-group-block">
          <span class="docs-nav-heading">Guides</span>
          ${pageLink(`${base}theming`, 'Design language', activeSlug === 'theming')}
          ${pageLink(`${base}health`, 'Book health', activeSlug === 'health')}
        </div>
        <div class="docs-nav-group-block">
          <span class="docs-nav-heading">Everyday</span>
          ${everydayCards.map((card) => sidebarLink(card, activeSlug)).join('')}
        </div>
        <div class="docs-nav-group-block">
          <span class="docs-nav-heading">Pro-app</span>
          ${proCards.map((card) => sidebarLink(card, activeSlug)).join('')}
        </div>
      </nav>
    </div>
  </aside>
  <div class="docs-scrim" data-close-nav hidden></div>`
}

function componentAnchor(doc: ComponentDoc) {
  return `component-${slugify(doc.name)}`
}

function sectionHtml(section: DocSection) {
  return `<div class="doc-section" id="${section.id}">
    <h3>${section.title}</h3>
    ${prepareHtml(section.html)}
  </div>`
}

function figureHtml(doc: ComponentDoc) {
  if (!doc.image) return ''
  return `<figure class="doc-figure">
    <img src="${prepareHtml(doc.image.src)}" alt="${doc.image.alt}"${doc.image.width ? ` width="${doc.image.width}" height="${doc.image.height}"` : ''} loading="lazy" />
    ${doc.image.caption ? `<figcaption>${doc.image.caption}</figcaption>` : ''}
  </figure>`
}

// A shadcn-docs-style preview: a live web illustration drawn with mkit-core's
// shadcn tokens, with the GPUI harness capture one tab away as the evidence.
function previewHtml(doc: ComponentDoc) {
  const demo = demos[doc.demoKey]
  if (!demo) return figureHtml(doc)
  const id = componentAnchor(doc)
  const theme = isLightMode() ? 'light' : 'dark'
  const gpuiTab = doc.image
    ? `<button class="preview-tab" role="tab" aria-selected="false" tabindex="-1" id="${id}-tab-gpui" aria-controls="${id}-gpui">GPUI render</button>`
    : ''
  const gpuiPane = doc.image
    ? `<div class="preview-pane preview-pane--gpui" id="${id}-gpui" role="tabpanel" aria-labelledby="${id}-tab-gpui" hidden>${figureHtml(doc)}</div>`
    : ''
  return `<div class="preview-block" data-demo="${doc.demoKey}" role="region" aria-label="${doc.name} preview">
    <div class="preview-toolbar">
      <div class="preview-tabs" role="tablist" aria-label="${doc.name} preview">
        <button class="preview-tab" role="tab" aria-selected="true" id="${id}-tab-live" aria-controls="${id}-live">Preview</button>
        ${gpuiTab}
      </div>
      <button class="preview-theme" type="button" data-preview-theme aria-label="Switch preview to ${theme === 'dark' ? 'light' : 'dark'} theme" title="Switch preview theme">${icon(theme === 'dark' ? 'sun' : 'moon', 14)}</button>
    </div>
    <div class="preview-pane" id="${id}-live" role="tabpanel" aria-labelledby="${id}-tab-live">
      <div class="ui preview-canvas preview-canvas--${demo.align ?? 'center'}" data-ui-theme="${theme}" style="min-height:${demo.minHeight ?? 320}px">${demo.html}</div>
    </div>
    ${gpuiPane}
  </div>
  <p class="preview-note">A web illustration styled with mkit-core's <code>shadcn</code> theme tokens, not the GPUI component itself.${doc.image ? ' The GPUI render tab shows the harness capture of the real component.' : ''}</p>`
}

// Registry entries are not installable until they are `source_ready`, so show
// the entry and its status instead of an install command.
function registryHtml(contracts: RegistryContract[]) {
  if (!contracts.length) return ''
  const rows = contracts
    .map(
      (contract) => `<li>
        <code>${contract.name}</code>
        <span class="registry-status">${registryStatusLabel(contract.status)}</span>
        ${contract.spec ? `<a href="${registryBlobUrl}/${contract.spec}" target="_blank" rel="noreferrer">Spec<span class="sr-only"> for ${contract.name} (opens in a new tab)</span> ${icon('external', 12)}</a>` : ''}
      </li>`,
    )
    .join('')
  return `<div class="registry-line">
    <span class="install-label">Registry</span>
    <ul>${rows}</ul>
    <p>Not installable yet: <code>cargo mkit add</code> rejects entries until they are <code>source_ready</code>.</p>
  </div>`
}

const hasSection = (doc: ComponentDoc, pattern: RegExp) => doc.sections.some((section) => pattern.test(section.title))

function contractHtml(contract: RegistryContract, open: boolean, showName: boolean) {
  if (!contract.keys.length && !contract.role) return ''
  const keys = contract.keys.length
    ? `<table>
        <caption class="sr-only">Keyboard map for ${contract.name}</caption>
        <thead><tr><th scope="col">Key</th><th scope="col">When</th><th scope="col">Result</th></tr></thead>
        <tbody>${contract.keys
          .map((row) => `<tr><td><kbd>${row.keys.split('+').join('</kbd>+<kbd>')}</kbd></td><td>${row.when}</td><td>${row.action}</td></tr>`)
          .join('')}</tbody>
      </table>`
    : '<p>No keyboard interaction: this component is not a tab stop.</p>'
  const properties = contract.properties.length
    ? `<ul>${contract.properties.map((property) => `<li><code>${property.name}</code>: ${property.value}</li>`).join('')}</ul>`
    : ''
  const states = contract.states.length
    ? `<p class="contract-states">Declared states: ${contract.states.map((state) => `<code>${state}</code>`).join(' ')}</p>`
    : ''
  return `<details class="contract"${open ? ' open' : ''}>
    <summary>${icon('keyboard', 15)}<span>${showName ? `<code>${contract.name}</code> ` : ''}Keyboard and accessibility contract</span><small>from the draft spec</small></summary>
    <div class="contract-body">
      <h4>Keyboard</h4>
      ${keys}
      <h4>Accessibility</h4>
      ${contract.role ? `<p>Role: <code>${contract.role}</code></p>` : ''}
      ${properties}
      ${states}
      <p class="contract-note">Generated from <code>${contract.spec || `registry/${contract.name}/spec.md`}</code> through its conformance manifest. Pending maintainer review; platform accessibility snapshots are not all recorded yet.</p>
    </div>
  </details>`
}

function contractsHtml(doc: ComponentDoc, contracts: RegistryContract[]) {
  // Open the summary when the chapter has no keyboard section of its own.
  const open = !hasSection(doc, /keyboard/i)
  return contracts.map((contract) => contractHtml(contract, open, contracts.length > 1)).join('')
}

// Paged chapters (timeline, node editor) share one registry entry; show its
// registry line and contract once, on the first chapter.
function componentHtml(doc: ComponentDoc, seen: Set<string>) {
  const contracts = doc.contracts.filter((contract) => !seen.has(contract.name))
  contracts.forEach((contract) => seen.add(contract.name))
  const notes = doc.notes.length
    ? `<div class="doc-notes">${doc.notes.map((note) => `<p>${prepareHtml(note)}</p>`).join('')}</div>`
    : ''
  const intro = doc.intro ? `<div class="doc-intro">${prepareHtml(doc.intro)}</div>` : ''
  const source = doc.sourcePath
    ? `<a class="doc-source" href="${bookBlobUrl}/${doc.sourcePath}" target="_blank" rel="noreferrer">Book chapter<span class="sr-only"> (opens in a new tab)</span> ${icon('external', 13)}</a>`
    : ''
  return `<section class="doc-component" id="${componentAnchor(doc)}">
    <div class="doc-component-head">
      <h2>${doc.name}</h2>
      ${source}
    </div>
    ${notes}
    ${doc.lede ? `<p class="doc-lede">${prepareHtml(doc.lede)}</p>` : ''}
    ${previewHtml(doc)}
    ${registryHtml(contracts)}
    ${intro}
    ${doc.sections.map(sectionHtml).join('')}
    ${contractsHtml(doc, contracts)}
  </section>`
}

function tocHtml(card: CatalogCard) {
  const rows = card.components
    .map((doc) => {
      const sections = doc.sections
        .map((section) => `<a class="docs-toc-link nested" href="#${section.id}">${section.title}</a>`)
        .join('')
      return `<a class="docs-toc-link" href="#${componentAnchor(doc)}">${doc.name}</a>${sections}`
    })
    .join('')
  return `<aside class="docs-toc" aria-label="On this page">
    <span class="docs-toc-heading">On this page</span>
    ${rows}
  </aside>`
}

export const skipLink = '<a class="skip-link" href="#main">Skip to content</a>'

export function docsTopbar(rightMeta: string, withSidebar = true) {
  const menu = withSidebar
    ? `<button class="icon-button docs-menu" type="button" aria-controls="docs-sidebar" aria-expanded="false" aria-label="Open documentation menu">${icon('menu')}</button>`
    : ''
  return `${skipLink}<header class="docs-topbar">
    ${menu}
    <a class="brand" href="${base}" aria-label="mkit home"><span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span><span>mkit</span><em>for GPUI</em></a>
    <nav class="docs-topnav" aria-label="Main">
      <a href="${base}#components">Components</a>
      <a href="${base}theming">Theming</a>
      <a href="${base}health">Book health</a>
      <a href="${bookSourceUrl}" target="_blank" rel="noreferrer">Book source<span class="sr-only"> (opens in a new tab)</span></a>
      <a href="${repoUrl}" target="_blank" rel="noreferrer">GitHub<span class="sr-only"> (opens in a new tab)</span></a>
    </nav>
    <div class="docs-topactions">
      <span class="docs-topmeta">${rightMeta}</span>
      <button class="docs-search" type="button" data-open-search aria-label="Search components and pages" aria-keyshortcuts="Meta+K Control+K">${icon('search', 14)}<span>Search</span><kbd>⌘K</kbd></button>
      <button class="icon-button theme-toggle" type="button" aria-label="Switch to light mode">${icon('sun')}</button>
    </div>
  </header>`
}

/** Mobile drawer for the docs sidebar. Returns a cleanup. */
export function mountDocsNav() {
  const shell = document.querySelector<HTMLElement>('.docs-shell')
  const button = document.querySelector<HTMLButtonElement>('.docs-menu')
  const scrim = document.querySelector<HTMLElement>('.docs-scrim')
  const sidebar = document.getElementById('docs-sidebar')
  if (!shell || !button || !sidebar) return () => {}
  const setOpen = (open: boolean) => {
    shell.classList.toggle('nav-open', open)
    button.setAttribute('aria-expanded', String(open))
    button.setAttribute('aria-label', open ? 'Close documentation menu' : 'Open documentation menu')
    if (scrim) scrim.hidden = !open
    if (open) (sidebar.querySelector<HTMLElement>('[aria-current="page"]') ?? sidebar.querySelector<HTMLElement>('a'))?.focus()
  }
  button.addEventListener('click', () => setOpen(!shell.classList.contains('nav-open')))
  scrim?.addEventListener('click', () => setOpen(false))
  sidebar.addEventListener('click', (event) => {
    if ((event.target as HTMLElement).closest('a')) setOpen(false)
  })
  const onKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && shell.classList.contains('nav-open')) {
      setOpen(false)
      button.focus()
    }
  }
  document.addEventListener('keydown', onKeydown)
  return () => document.removeEventListener('keydown', onKeydown)
}

function pagerHtml(slug: string) {
  const { previous, next } = previousNext(slug)
  if (!previous && !next) return ''
  const previousLink = previous
    ? `<a class="docs-pager-link previous" href="${base}components/${previous.slug}"><span>Previous</span><b>${previous.name}</b></a>`
    : '<span></span>'
  const nextLink = next
    ? `<a class="docs-pager-link next" href="${base}components/${next.slug}"><span>Next</span><b>${next.name}</b></a>`
    : '<span></span>'
  return `<nav class="docs-pager" aria-label="Component pagination">${previousLink}${nextLink}</nav>`
}

export function renderDocs(card: CatalogCard) {
  const kind = kindLabel(card.kind)
  const count = card.components.length
  return `<div class="docs-shell">
    ${docsTopbar(`${kind} · pre-release`)}
    <div class="docs-layout">
      ${docsSidebar(card.slug)}
      <main class="docs-main" id="main" tabindex="-1">
        <nav class="docs-breadcrumb" aria-label="Breadcrumb">
          <a href="${base}#components">Components</a><span aria-hidden="true">/</span><span>${kind}</span><span aria-hidden="true">/</span><b aria-current="page">${card.name}</b>
        </nav>
        <header class="docs-header">
          <span class="docs-kicker">${kind} ${count > 1 ? `· ${count} components` : 'component'}</span>
          <h1>${card.name}</h1>
          <div class="docs-meta">
            <span class="status ${statusClass(card.status)}"><i aria-hidden="true"></i>${statusLabel(card.status)}</span>
            <span class="status prerelease"><i aria-hidden="true"></i>Pre-release</span>
            ${card.tags.map((tag) => `<span class="docs-tag">${tag}</span>`).join('')}
          </div>
          <p class="docs-lede">${card.description}</p>
          <div class="docs-callout">${icon('book', 15)}<span>${draftNote(card)}</span></div>
        </header>
        <article class="docs-article">
          ${(() => {
            const seen = new Set<string>()
            return card.components.map((doc) => componentHtml(doc, seen)).join('')
          })()}
        </article>
        ${pagerHtml(card.slug)}
      </main>
      ${tocHtml(card)}
    </div>
  </div>`
}

const escapeHtml = (value: string) =>
  value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')

export function renderNotFound(pathname: string) {
  return `<div class="docs-shell">
    ${docsTopbar('', false)}
    <main class="docs-notfound" id="main" tabindex="-1">
      <span class="section-kicker">404</span>
      <h1>That page isn't here.</h1>
      <p>The page <code>${escapeHtml(pathname)}</code> doesn't match a component or guide. Try search, or go back to the catalog.</p>
      <a class="button primary" href="${base}#components">Back to components ${icon('arrow', 16)}</a>
    </main>
  </div>`
}

/** Wire previews on a rendered docs page. Returns a cleanup for demo timers. */
export function mountDocs() {
  installUiTokens()
  const cleanups: (() => void)[] = []
  document.querySelectorAll<HTMLElement>('.preview-block').forEach((block) => {
    const canvas = block.querySelector<HTMLElement>('.preview-canvas')
    const demo = demos[block.dataset.demo ?? '']
    if (canvas && demo) {
      mountBehaviors(canvas)
      const cleanup = demo.mount?.(canvas)
      if (cleanup) cleanups.push(cleanup)
    }
    const tabs = [...block.querySelectorAll<HTMLButtonElement>('.preview-tab')]
    const select = (tab: HTMLButtonElement) =>
      tabs.forEach((other) => {
        const on = other === tab
        other.setAttribute('aria-selected', String(on))
        other.tabIndex = on ? 0 : -1
        const pane = document.getElementById(other.getAttribute('aria-controls') ?? '')
        if (pane) pane.hidden = !on
      })
    tabs.forEach((tab, index) => {
      tab.addEventListener('click', () => select(tab))
      tab.addEventListener('keydown', (event) => {
        const delta = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0
        if (!delta) return
        const next = tabs[(index + delta + tabs.length) % tabs.length]
        next.focus()
        select(next)
      })
    })
    const toggle = block.querySelector<HTMLButtonElement>('[data-preview-theme]')
    toggle?.addEventListener('click', () => {
      if (!canvas) return
      const next = canvas.dataset.uiTheme === 'dark' ? 'light' : 'dark'
      canvas.dataset.uiTheme = next
      toggle.innerHTML = icon(next === 'dark' ? 'sun' : 'moon', 14)
      toggle.setAttribute('aria-label', `Switch preview to ${next === 'dark' ? 'light' : 'dark'} theme`)
    })
  })
  cleanups.push(mountDocsNav())
  return () => cleanups.forEach((cleanup) => cleanup())
}
