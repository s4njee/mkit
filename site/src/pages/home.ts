import { allCards, everydayCards, kindLabel, proCards, registryCount, statusClass, statusLabel } from '../catalog'
import { icon } from '../icons'
import { planUrl, repoUrl, siteHref } from '../links'
import { miniPreview } from '../previews'
import type { CatalogCard, ComponentKind } from '../types'

const base = import.meta.env.BASE_URL || '/'

let activeFilter: 'all' | ComponentKind = 'all'
let query = ''

function componentCard(card: CatalogCard) {
  const count = card.components.length
  const eyebrow = `${kindLabel(card.kind)}${count > 1 ? ` · ${count} components` : ''}`
  return `<article class="component-card accent-${card.accent}" data-kind="${card.kind}" id="${card.slug}">
    <div class="card-topline"><span class="eyebrow">${eyebrow}</span><span class="status ${statusClass(card.status)}"><i aria-hidden="true"></i>${statusLabel(card.status)}</span></div>
    <div class="preview-shell" aria-hidden="true" inert>${miniPreview(card.preview)}</div>
    <div class="card-copy"><div><h3><a class="card-hit" href="${base}components/${card.slug}">${card.name}</a></h3><p>${card.description}</p></div></div>
    <div class="tag-list" aria-label="Tags">${card.tags.map((tag) => `<span>${tag}</span>`).join('')}</div>
    <div class="card-foot" aria-hidden="true"><span>Read the guide</span>${icon('arrow', 14)}</div>
  </article>`
}

function renderCards() {
  const grid = document.querySelector<HTMLDivElement>('#component-grid')
  if (!grid) return
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean)
  const visible = allCards.filter((card) => {
    const matchesFilter = activeFilter === 'all' || card.kind === activeFilter
    const names = card.components.flatMap((doc) => [doc.name, ...doc.registryNames]).join(' ')
    const haystack = `${card.name} ${card.description} ${card.tags.join(' ')} ${names}`.toLowerCase()
    return matchesFilter && terms.every((term) => haystack.includes(term))
  })
  grid.innerHTML = visible.length
    ? visible.map(componentCard).join('')
    : `<div class="empty-state"><span>${icon('search', 20)}</span><h3>No components found</h3><p>Try a different component, tag, or category.</p></div>`
  const count = document.querySelector('#result-count')
  if (count) count.textContent = `${visible.length} ${visible.length === 1 ? 'guide' : 'guides'}`
}

function roadmapTrack(label: string, cards: CatalogCard[], tone: string) {
  const total = registryCount(cards)
  const documented = registryCount(cards.filter((card) => card.status === 'ready'))
  const width = total ? Math.round((documented / total) * 100) : 0
  return `<div class="track-label${tone === 'violet' ? ' pro-label' : ''}"><b>${label}</b><small>${documented} of ${total} documented</small></div>
    <div class="track-line${tone === 'violet' ? ' violet-track' : ''}" role="img" aria-label="${documented} of ${total} ${label.toLowerCase()} components documented"><i class="filled" style="width:${width}%"></i></div>`
}

function roadmapNote() {
  const inProgress = allCards.filter((card) => card.status === 'in progress')
  const pro = inProgress.filter((card) => card.kind === 'pro').map((card) => card.name)
  const everyday = inProgress.length - pro.length
  const parts = [...pro, everyday ? `${everyday} everyday guides` : ''].filter(Boolean)
  return parts.length
    ? `<div class="track-note"><span class="signal-dot" aria-hidden="true"></span><span>In progress</span><b>${parts.join(' · ')}</b></div>`
    : ''
}

export function renderHome() {
  const total = registryCount(allCards)
  return `<a class="skip-link" href="#main">Skip to content</a><div class="site-shell">
    <div class="ambient ambient-one" aria-hidden="true"></div><div class="ambient ambient-two" aria-hidden="true"></div>
    <header class="topbar">
      <a class="brand" href="${base}" aria-label="mkit home"><span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span><span>mkit</span><em>for GPUI</em></a>
      <nav class="main-nav" id="main-nav" aria-label="Main"><a href="#components">Components</a><a href="${base}theming">Theming</a><a href="#principles">Principles</a><a href="#roadmap">Roadmap</a><a href="${base}health">Book health</a></nav>
      <div class="top-actions"><button class="icon-button top-search" type="button" data-open-search aria-label="Search components and pages" aria-keyshortcuts="Meta+K Control+K">${icon('search')}</button><button class="icon-button theme-toggle" type="button" aria-label="Switch to light mode">${icon('sun')}</button><a class="github-link" href="${repoUrl}" target="_blank" rel="noreferrer">GitHub<span class="sr-only"> (opens in a new tab)</span> ${icon('arrow', 14)}</a><button class="mobile-menu" type="button" aria-controls="main-nav" aria-expanded="false" aria-label="Open menu">${icon('menu')}</button></div>
    </header>

    <main id="main" tabindex="-1">
      <section class="hero content-width" aria-labelledby="hero-title">
        <div class="hero-copy">
          <a class="announcement" href="#roadmap"><span class="pulse" aria-hidden="true"></span> Pre-release · ${total} components in development <span class="announcement-arrow" aria-hidden="true">→</span></a>
          <h1 id="hero-title">Build tools that<br/><span>feel like yours.</span></h1>
          <p class="hero-lede">An ownable component kit for GPUI. Start with the patterns you know, then take them somewhere only pro apps can go.</p>
          <div class="hero-actions"><a class="button primary" href="#components">Explore components ${icon('arrow', 16)}</a><button class="button ghost command-trigger" type="button" data-open-search aria-keyshortcuts="Meta+K Control+K">${icon('search', 15)}<span>Search</span><kbd class="command-kbd">⌘K</kbd></button></div>
          <div class="hero-meta"><span><b class="meta-dot coral" aria-hidden="true"></b>GPUI-native</span><span><b class="meta-dot violet" aria-hidden="true"></b>Source you own</span><span><b class="meta-dot cyan" aria-hidden="true"></b>Spec-first</span></div>
        </div>
        <div class="hero-visual" role="img" aria-label="Illustration of a pro-app window built from mkit components: a curve editor, a precision slider, colour swatches, and a command palette">
          <div class="visual-toolbar"><span><i class="dot red"></i><i class="dot yellow"></i><i class="dot green"></i></span><span class="visual-path">mkit / gallery / pro-app</span><span class="visual-kbd">⌘ K</span></div>
          <div class="visual-grid">
            <div class="visual-sidebar"><b>COMPONENTS</b><span class="active">Overview</span><span>Everyday</span><span>Pro-app</span><div class="sidebar-fade"></div><b class="bottom-label">THEME</b><span>shadcn / dark</span></div>
            <div class="visual-main"><div class="visual-head"><div><span class="tiny-label">PRO-APP</span><h2>Make the complex<br/><em>feel simple.</em></h2></div><span class="live-pill"><i></i>Preview</span></div><div class="signal-graph"><div class="graph-label">VIEWPORT / CURVE EDITOR</div><svg viewBox="0 0 460 170" preserveAspectRatio="none"><path class="gridline" d="M0 34H460M0 85H460M0 136H460M92 0V170M184 0V170M276 0V170M368 0V170"/><path class="signal red-line" d="M0 139 C35 118 57 134 78 105 S127 48 159 78 S208 118 232 61 S281 34 304 81 S350 124 377 63 S421 40 460 48"/><path class="signal violet-line" d="M0 112 C40 124 59 79 90 93 S146 121 177 95 S210 48 248 72 S285 116 324 89 S376 47 408 91 S439 118 460 101"/><path class="signal cyan-line" d="M0 150 C33 136 50 147 84 124 S125 110 153 132 S197 138 232 111 S279 89 308 110 S347 137 378 114 S427 99 460 110"/><circle cx="304" cy="81" r="4"/><circle cx="324" cy="89" r="4"/></svg><div class="graph-axis"><span>0</span><span>25</span><span>50</span><span>75</span><span>100</span></div></div><div class="visual-panels"><div><span class="tiny-label">PRECISION SLIDER</span><b>+12.4</b><div class="visual-range"><i></i></div></div><div><span class="tiny-label">COLOUR TOOLS</span><div class="swatches"><i></i><i></i><i></i><i></i></div></div><div><span class="tiny-label">COMMAND PALETTE</span><b class="code-line">⌘ K <em>Search actions</em></b></div></div></div>
          </div>
          <div class="visual-caption"><span>Prototype at the speed of thought.</span><span>shadcn dark</span></div>
        </div>
      </section>

      <section class="principles content-width" id="principles" aria-labelledby="principles-title"><div class="section-intro"><span class="section-kicker">01 / WHY MKIT</span><h2 id="principles-title">Familiar foundations.<br/><span>Unfamiliar territory.</span></h2><p>mkit covers the everyday surface area, then goes deeper for the tools people use to make, edit, inspect, and understand.</p></div><div class="principle-grid"><div class="principle-card"><span class="principle-number" aria-hidden="true">01</span><h3>Own the source</h3><p>Like shadcn/ui, components are copied into your app. Change the API, the look, or the behavior without waiting for upstream.</p><div class="principle-line coral-line"></div></div><div class="principle-card"><span class="principle-number" aria-hidden="true">02</span><h3>Built for pro apps</h3><p>Viewports, curves, timelines, node graphs, and inspectors — the hard, specific parts of creative and data tools.</p><div class="principle-line violet-line-bg"></div></div><div class="principle-card"><span class="principle-number" aria-hidden="true">03</span><h3>Trust the spec</h3><p>Keyboard maps, accessibility roles, theme tokens, and visual states are written down before implementation.</p><div class="principle-line cyan-line-bg"></div></div></div></section>

      <section class="component-section content-width" id="components" aria-labelledby="components-title"><div class="section-heading"><div><span class="section-kicker">02 / THE CATALOG</span><h2 id="components-title">Components, with a point of view.</h2><p class="section-sub">Every card opens a guide built from the book: a preview, the API, keyboard map, accessibility, and theme tokens. All ${total} components are pre-release.</p></div><span class="section-count" id="result-count" aria-live="polite">${allCards.length} guides</span></div><div class="catalog-toolbar"><div class="filter-tabs" role="group" aria-label="Filter by kind"><button class="filter-tab active" type="button" data-filter="all" aria-pressed="true">All <span>${allCards.length}</span></button><button class="filter-tab" type="button" data-filter="everyday" aria-pressed="false">Everyday <span>${everydayCards.length}</span></button><button class="filter-tab" type="button" data-filter="pro" aria-pressed="false">Pro-app <span>${proCards.length}</span></button></div><label class="search-box">${icon('search', 16)}<span class="sr-only">Filter components</span><input id="component-search" type="search" placeholder="Filter components…" autocomplete="off"/></label></div><div class="component-grid" id="component-grid"></div></section>

      <section class="roadmap content-width" id="roadmap" aria-labelledby="roadmap-title"><div class="roadmap-card"><div class="roadmap-copy"><span class="section-kicker">03 / THE ROAD AHEAD</span><h2 id="roadmap-title">From controls<br/>to <span>complete tools.</span></h2><p>Everyday components give you the vocabulary. Pro-app components give you the range. Together, they’re the foundation for interfaces that can keep up with expert users.</p><a class="text-link" href="${planUrl}" target="_blank" rel="noreferrer">Read the full plan<span class="sr-only"> (opens in a new tab)</span> ${icon('arrow', 15)}</a></div><div class="roadmap-track">${roadmapTrack('Everyday', everydayCards, 'coral')}${roadmapTrack('Pro-app', proCards, 'violet')}${roadmapNote()}<p class="track-foot">Documented means the book chapter and spec are written. Every component is still pre-release and awaits maintainer review.</p></div></div></section>
    </main>
    <footer class="footer content-width"><a class="brand" href="${base}"><span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span><span>mkit</span></a><span>Ownable UI for GPUI.</span><a href="${base}health">Book health</a><a href="${siteHref('llms.txt')}" data-native>llms.txt</a><span>© 2026 mk7s</span><a href="https://mk7s.dev/mkit" target="_blank" rel="noreferrer">mk7s.dev/mkit ↗<span class="sr-only"> (opens in a new tab)</span></a></footer>
  </div>`
}

export function mountHome() {
  renderCards()

  document.querySelectorAll<HTMLButtonElement>('.filter-tab').forEach((button) =>
    button.addEventListener('click', () => {
      activeFilter = button.dataset.filter as typeof activeFilter
      document.querySelectorAll<HTMLButtonElement>('.filter-tab').forEach((tab) => {
        tab.classList.toggle('active', tab === button)
        tab.setAttribute('aria-pressed', String(tab === button))
      })
      renderCards()
    }),
  )

  const filter = document.querySelector<HTMLInputElement>('#component-search')
  if (filter) {
    filter.value = query
    filter.addEventListener('input', () => {
      query = filter.value
      renderCards()
    })
  }

  const nav = document.querySelector<HTMLElement>('.main-nav')
  const menu = document.querySelector<HTMLButtonElement>('.mobile-menu')
  const setMenu = (open: boolean) => {
    nav?.classList.toggle('open', open)
    menu?.setAttribute('aria-expanded', String(open))
    menu?.setAttribute('aria-label', open ? 'Close menu' : 'Open menu')
  }
  menu?.addEventListener('click', () => setMenu(!nav?.classList.contains('open')))
  nav?.addEventListener('click', (event) => {
    if ((event.target as HTMLElement).closest('a')) setMenu(false)
  })
  const onKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && nav?.classList.contains('open')) {
      setMenu(false)
      menu?.focus()
    }
  }
  document.addEventListener('keydown', onKeydown)
  return () => document.removeEventListener('keydown', onKeydown)
}
