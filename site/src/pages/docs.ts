import { statusClass, previousNext, everydayCards, proCards } from '../catalog'
import { demos } from '../demos'
import { icon } from '../icons'
import { isLightMode } from '../theme'
import { mountBehaviors } from '../ui/behaviors'
import { installUiTokens } from '../ui/tokens'
import type { CatalogCard, ComponentDoc, DocSection } from '../types'

const base = import.meta.env.BASE_URL || '/'
const bookBlob = 'https://github.com/mk7s/mkit/blob/main/book/src'

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
    ? 'API, keyboard map, and accessibility contract are documented but still need maintainer sign-off.'
    : 'This is a work-in-progress draft. The API and conformance evidence will change.'

function sidebarLink(card: CatalogCard, activeSlug: string) {
  const active = card.slug === activeSlug
  return `<a class="docs-nav-link${active ? ' active' : ''}" href="${base}components/${card.slug}"${active ? ' aria-current="page"' : ''}>
    <span class="docs-nav-name">${card.name}</span>
    <span class="docs-nav-group">${card.group}</span>
  </a>`
}

export function docsSidebar(activeSlug: string) {
  const themingActive = activeSlug === 'theming'
  return `<aside class="docs-sidebar" aria-label="Documentation">
    <div class="docs-sidebar-inner">
      <nav class="docs-nav">
        <div class="docs-nav-group-block">
          <span class="docs-nav-heading">Theming</span>
          <a class="docs-nav-link${themingActive ? ' active' : ''}" href="${base}theming"${themingActive ? ' aria-current="page"' : ''}>
            <span class="docs-nav-name">Design language</span>
            <span class="docs-nav-group">E9.1</span>
          </a>
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
  </aside>`
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
    <img src="${prepareHtml(doc.image.src)}" alt="${doc.image.alt}" loading="lazy" />
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
  return `<div class="preview-block" data-demo="${doc.demoKey}">
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
  <p class="preview-note">Web preview styled with mkit-core's <code>shadcn</code> theme tokens. ${doc.image ? 'The GPUI render tab shows the harness capture of the real component.' : ''}</p>`
}

function installHtml(doc: ComponentDoc) {
  if (!doc.registryName) return ''
  const command = `cargo mkit add ${doc.registryName}`
  return `<div class="install-line">
    <span class="install-label">Install</span>
    <code>${command}</code>
    <button class="install-copy" type="button" data-copy-command="${command}" aria-label="Copy install command">${icon('copy', 14)}</button>
  </div>`
}

function componentHtml(doc: ComponentDoc) {
  const notes = doc.notes.length
    ? `<div class="doc-notes">${doc.notes.map((note) => `<p>${prepareHtml(note)}</p>`).join('')}</div>`
    : ''
  const intro = doc.intro ? `<div class="doc-intro">${prepareHtml(doc.intro)}</div>` : ''
  const source = doc.sourcePath
    ? `<a class="doc-source" href="${bookBlob}/${doc.sourcePath}" target="_blank" rel="noreferrer">Book chapter ${icon('external', 13)}</a>`
    : ''
  return `<section class="doc-component" id="${componentAnchor(doc)}">
    <div class="doc-component-head">
      <h2>${doc.name}</h2>
      ${source}
    </div>
    ${notes}
    ${doc.lede ? `<p class="doc-lede">${prepareHtml(doc.lede)}</p>` : ''}
    ${previewHtml(doc)}
    ${installHtml(doc)}
    ${intro}
    ${doc.sections.map(sectionHtml).join('')}
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

export function docsTopbar(rightMeta: string) {
  return `<header class="docs-topbar">
    <a class="brand" href="${base}" aria-label="mkit home"><span class="brand-mark"><i></i><i></i><i></i></span><span>mkit</span><em>for GPUI</em></a>
    <nav class="docs-topnav" aria-label="Main navigation">
      <a href="${base}#components">Components</a>
      <a href="${base}theming">Theming</a>
      <a href="https://github.com/mk7s/mkit/tree/main/book/src" target="_blank" rel="noreferrer">Book</a>
      <a href="https://github.com/mk7s/mkit" target="_blank" rel="noreferrer">GitHub</a>
    </nav>
    <div class="docs-topactions">
      <span class="docs-topmeta">${rightMeta}</span>
      <button class="icon-button theme-toggle" aria-label="Toggle light mode">${icon('sun')}</button>
    </div>
  </header>`
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
  const kindLabel = card.kind === 'pro' ? 'Pro-app' : 'Everyday'
  return `<div class="docs-shell">
    ${docsTopbar(`${card.group} · ${kindLabel}`)}
    <div class="docs-layout">
      ${docsSidebar(card.slug)}
      <main class="docs-main" id="top">
        <nav class="docs-breadcrumb" aria-label="Breadcrumb">
          <a href="${base}">Components</a><span>/</span><span>${kindLabel}</span><span>/</span><b>${card.name}</b>
        </nav>
        <header class="docs-header">
          <span class="docs-kicker">${card.group} · ${kindLabel}</span>
          <h1>${card.name}</h1>
          <div class="docs-meta">
            <span class="status ${statusClass(card.status)}"><i></i>${card.status}</span>
            ${card.tags.map((tag) => `<span class="docs-tag">${tag}</span>`).join('')}
          </div>
          <p class="docs-lede">${card.description}</p>
          <div class="docs-callout">${icon('book', 15)}<span>${draftNote(card)}</span></div>
        </header>
        <article class="docs-article">
          ${card.components.map(componentHtml).join('')}
        </article>
        ${pagerHtml(card.slug)}
      </main>
      ${tocHtml(card)}
    </div>
  </div>`
}

export function renderNotFound(pathname: string) {
  return `<div class="docs-shell">
    <header class="docs-topbar">
      <a class="brand" href="${base}"><span class="brand-mark"><i></i><i></i><i></i></span><span>mkit</span><em>for GPUI</em></a>
      <nav class="docs-topnav"><a href="${base}#components">Components</a></nav>
    </header>
    <main class="docs-notfound">
      <span class="section-kicker">404</span>
      <h1>That component isn't here.</h1>
      <p>The page <code>${pathname}</code> doesn't match a component in the catalog.</p>
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
  document.querySelectorAll<HTMLButtonElement>('[data-copy-command]').forEach((button) =>
    button.addEventListener('click', async () => {
      await navigator.clipboard?.writeText(button.dataset.copyCommand ?? '')
      button.innerHTML = icon('check', 14)
      setTimeout(() => (button.innerHTML = icon('copy', 14)), 1300)
    }),
  )
  return () => cleanups.forEach((cleanup) => cleanup())
}
