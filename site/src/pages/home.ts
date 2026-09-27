import { allCards, everydayCards, proCards, statusClass } from '../catalog'
import { icon } from '../icons'
import { miniPreview } from '../previews'
import { mountThemeToggle } from '../theme'
import type { CatalogCard, ComponentKind } from '../types'

const base = import.meta.env.BASE_URL || '/'

let activeFilter: 'all' | ComponentKind = 'all'
let query = ''

// Copy the install command for the card's first registry component.
function installButton(card: CatalogCard) {
  const name = card.components.find((doc) => doc.registryName)?.registryName
  return name
    ? `<button class="copy-button" data-copy="${name}" aria-label="Copy cargo mkit add ${name}">${icon('copy', 14)}</button>`
    : ''
}

function componentCard(card: CatalogCard) {
  return `<article class="component-card accent-${card.accent}" data-kind="${card.kind}" data-name="${card.name.toLowerCase()} ${card.tags.join(' ')}" id="${card.slug}">
    <a class="card-hit" href="${base}components/${card.slug}" aria-label="Open ${card.name} documentation"></a>
    <div class="card-topline"><span class="eyebrow">${card.group}</span><span class="status ${statusClass(card.status)}"><i></i>${card.status}</span></div>
    <div class="preview-shell">${miniPreview(card.preview)}</div>
    <div class="card-copy"><div><h3>${card.name}</h3><p>${card.description}</p></div>${installButton(card)}</div>
    <div class="tag-list">${card.tags.map((tag) => `<span>${tag}</span>`).join('')}</div>
    <div class="card-foot"><span>Read the guide</span>${icon('arrow', 14)}</div>
  </article>`
}

function renderCards() {
  const grid = document.querySelector<HTMLDivElement>('#component-grid')
  if (!grid) return
  const visible = allCards.filter((card) => {
    const matchesFilter = activeFilter === 'all' || card.kind === activeFilter
    const haystack = `${card.name} ${card.description} ${card.tags.join(' ')}`.toLowerCase()
    return matchesFilter && haystack.includes(query.toLowerCase())
  })
  grid.innerHTML = visible.length
    ? visible.map(componentCard).join('')
    : `<div class="empty-state"><span>${icon('search', 20)}</span><h3>No components found</h3><p>Try a different component, tag, or category.</p></div>`
  const count = document.querySelector('#result-count')
  if (count) count.textContent = `${visible.length} component${visible.length === 1 ? '' : 's'}`
}

export function renderHome() {
  return `<div class="site-shell">
    <div class="ambient ambient-one"></div><div class="ambient ambient-two"></div>
    <header class="topbar">
      <a class="brand" href="#top" aria-label="mkit home"><span class="brand-mark"><i></i><i></i><i></i></span><span>mkit</span><em>for GPUI</em></a>
      <nav class="main-nav" aria-label="Main navigation"><a href="#components">Components</a><a href="${base}theming">Theming</a><a href="#principles">Principles</a><a href="#roadmap">Roadmap</a></nav>
      <div class="top-actions"><button class="icon-button theme-toggle" aria-label="Toggle light mode">${icon('sun')}</button><a class="github-link" href="https://github.com/mk7s/mkit" target="_blank" rel="noreferrer">GitHub ${icon('arrow', 14)}</a><button class="mobile-menu" aria-label="Open menu">☰</button></div>
    </header>

    <main id="top">
      <section class="hero content-width">
        <div class="hero-copy">
          <div class="announcement"><span class="pulse"></span> E7 everyday · E8 pro-app components <span class="announcement-arrow">↗</span></div>
          <h1>Build tools that<br/><span>feel like yours.</span></h1>
          <p class="hero-lede">An ownable component kit for GPUI. Start with the patterns you know, then take them somewhere only pro apps can go.</p>
          <div class="hero-actions"><a class="button primary" href="#components">Explore components ${icon('arrow', 16)}</a><button class="button ghost command-trigger"><span class="command-symbol">⌘</span> K <span class="command-label">Search components</span></button></div>
          <div class="hero-meta"><span><b class="meta-dot coral"></b>GPUI-native</span><span><b class="meta-dot violet"></b>Source you own</span><span><b class="meta-dot cyan"></b>Spec-tested</span></div>
        </div>
        <div class="hero-visual" aria-label="A live preview of mkit components">
          <div class="visual-toolbar"><span><i class="dot red"></i><i class="dot yellow"></i><i class="dot green"></i></span><span class="visual-path">mkit / gallery / e8</span><span class="visual-kbd">⌘ K</span></div>
          <div class="visual-grid">
            <div class="visual-sidebar"><b>COMPONENTS</b><span class="active">Overview</span><span>Everyday</span><span>Pro-app</span><div class="sidebar-fade"></div><b class="bottom-label">THEME</b><span>shadcn / dark</span></div>
            <div class="visual-main"><div class="visual-head"><div><span class="tiny-label">E8 / PRO-APP</span><h2>Make the complex<br/><em>feel simple.</em></h2></div><span class="live-pill"><i></i>Live preview</span></div><div class="signal-graph"><div class="graph-label">VIEWPORT / CURVE EDITOR</div><svg viewBox="0 0 460 170" preserveAspectRatio="none"><path class="gridline" d="M0 34H460M0 85H460M0 136H460M92 0V170M184 0V170M276 0V170M368 0V170"/><path class="signal red-line" d="M0 139 C35 118 57 134 78 105 S127 48 159 78 S208 118 232 61 S281 34 304 81 S350 124 377 63 S421 40 460 48"/><path class="signal violet-line" d="M0 112 C40 124 59 79 90 93 S146 121 177 95 S210 48 248 72 S285 116 324 89 S376 47 408 91 S439 118 460 101"/><path class="signal cyan-line" d="M0 150 C33 136 50 147 84 124 S125 110 153 132 S197 138 232 111 S279 89 308 110 S347 137 378 114 S427 99 460 110"/><circle cx="304" cy="81" r="4"/><circle cx="324" cy="89" r="4"/></svg><div class="graph-axis"><span>0</span><span>25</span><span>50</span><span>75</span><span>100</span></div></div><div class="visual-panels"><div><span class="tiny-label">PRECISION SLIDER</span><b>+12.4</b><div class="visual-range"><i></i></div></div><div><span class="tiny-label">COLOUR TOOLS</span><div class="swatches"><i></i><i></i><i></i><i></i></div></div><div><span class="tiny-label">COMMAND PALETTE</span><b class="code-line">⌘ K <em>Search actions</em></b></div></div></div>
          </div>
          <div class="visual-caption"><span>Prototype at the speed of thought.</span><span>01 / 03</span></div>
        </div>
      </section>

      <section class="principles content-width" id="principles"><div class="section-intro"><span class="section-kicker">01 / WHY MKIT</span><h2>Familiar foundations.<br/><span>Unfamiliar territory.</span></h2><p>mkit covers the everyday surface area, then goes deeper for the tools people use to make, edit, inspect, and understand.</p></div><div class="principle-grid"><div class="principle-card"><span class="principle-number">01</span><h3>Own the source</h3><p>Like shadcn/ui, components land in your app. Change the API, the look, or the behavior without waiting for upstream.</p><div class="principle-line coral-line"></div></div><div class="principle-card"><span class="principle-number">02</span><h3>Built for pro apps</h3><p>Viewports, curves, timelines, node graphs, and inspectors — the hard, specific parts of creative and data tools.</p><div class="principle-line violet-line-bg"></div></div><div class="principle-card"><span class="principle-number">03</span><h3>Trust the spec</h3><p>Keyboard maps, accessibility roles, theme tokens, and visual states are written down before implementation.</p><div class="principle-line cyan-line-bg"></div></div></div></section>

      <section class="component-section content-width" id="components"><div class="section-heading"><div><span class="section-kicker">02 / THE CATALOG</span><h2>Components, with a point of view.</h2><p class="section-sub">Every card opens a guide built from the book: API, keyboard map, accessibility, and theme tokens.</p></div><span class="section-count" id="result-count">${allCards.length} components</span></div><div class="catalog-toolbar"><div class="filter-tabs" role="tablist"><button class="filter-tab active" data-filter="all" role="tab">All <span>${allCards.length}</span></button><button class="filter-tab" data-filter="everyday" role="tab">Everyday <span>${everydayCards.length}</span></button><button class="filter-tab" data-filter="pro" role="tab">Pro-app <span>${proCards.length}</span></button></div><label class="search-box">${icon('search', 16)}<input id="component-search" type="search" placeholder="Search components..." autocomplete="off"/><kbd>⌘ K</kbd></label></div><div class="component-grid" id="component-grid"></div></section>

      <section class="roadmap content-width" id="roadmap"><div class="roadmap-card"><div class="roadmap-copy"><span class="section-kicker">03 / THE ROAD AHEAD</span><h2>From controls<br/>to <span>complete tools.</span></h2><p>E7 gives you the vocabulary. E8 gives you the range. Together, they’re the foundation for interfaces that can keep up with expert users.</p><a class="text-link" href="https://github.com/mk7s/mkit/blob/main/plan.md" target="_blank" rel="noreferrer">Read the full plan ${icon('arrow', 15)}</a></div><div class="roadmap-track"><div class="track-label"><span>E7</span><b>Everyday</b><small>9 groups</small></div><div class="track-line"><i class="filled"></i><i class="marker m1"></i><i class="marker m2"></i><i class="marker m3"></i></div><div class="track-label pro-label"><span>E8</span><b>Pro-app</b><small>14 components</small></div><div class="track-line violet-track"><i class="filled"></i><i class="marker m1"></i><i class="marker m2"></i><i class="marker m3"></i></div><div class="track-note"><span class="signal-dot"></span><span>next up</span><b>Timeline + node editor</b></div></div></div></section>
    </main>
    <footer class="footer content-width"><a class="brand" href="#top"><span class="brand-mark"><i></i><i></i><i></i></span><span>mkit</span></a><span>Ownable UI for GPUI.</span><span>© 2026 mk7s</span><a href="https://mk7s.dev/mkit" target="_blank" rel="noreferrer">mk7s.dev/mkit ↗</a></footer>
  </div><div class="command-modal" aria-hidden="true"><div class="command-backdrop"></div><div class="command-dialog" role="dialog" aria-modal="true" aria-label="Search components"><div class="command-input">${icon('search', 18)}<input placeholder="Search components and patterns..." autocomplete="off"/><kbd>ESC</kbd></div><div class="command-results"><div class="command-group-label">COMPONENTS</div>${allCards.map((card, i) => `<a class="command-result ${i === 0 ? 'active' : ''}" href="${base}components/${card.slug}"><span class="result-icon">${icon(i % 2 ? 'grid' : 'terminal', 15)}</span><span>${card.name}<small>${card.group} · ${card.kind}</small></span><kbd>${i < 3 ? `⌘ ${i + 1}` : '↵'}</kbd></a>`).join('')}</div><div class="command-footer"><span>↑↓ navigate</span><span>↵ open</span><span>esc close</span></div></div></div>`
}

export function mountHome() {
  renderCards()

  document.querySelectorAll<HTMLButtonElement>('.filter-tab').forEach((button) =>
    button.addEventListener('click', () => {
      activeFilter = button.dataset.filter as typeof activeFilter
      document.querySelectorAll('.filter-tab').forEach((tab) => tab.classList.toggle('active', tab === button))
      renderCards()
    }),
  )

  document.querySelector<HTMLInputElement>('#component-search')?.addEventListener('input', (event) => {
    query = (event.target as HTMLInputElement).value
    renderCards()
  })

  const modal = document.querySelector<HTMLDivElement>('.command-modal')
  const modalInput = document.querySelector<HTMLInputElement>('.command-input input')
  if (!modal || !modalInput) return
  const openModal = () => {
    modal.classList.add('open')
    modal.setAttribute('aria-hidden', 'false')
    modalInput.value = ''
    modalInput.focus()
  }
  const closeModal = () => {
    modal.classList.remove('open')
    modal.setAttribute('aria-hidden', 'true')
  }

  document.querySelectorAll('.command-trigger, .search-box').forEach((element) =>
    element.addEventListener('click', (event) => {
      if (element.classList.contains('search-box') && (event.target as HTMLElement).tagName === 'INPUT') return
      event.preventDefault()
      openModal()
    }),
  )
  document.querySelector('.command-backdrop')?.addEventListener('click', closeModal)
  document.querySelector('.command-input kbd')?.addEventListener('click', closeModal)
  const onKeydown = (event: KeyboardEvent) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault()
      openModal()
    }
    if (event.key === 'Escape') closeModal()
  }
  document.addEventListener('keydown', onKeydown)
  modalInput.addEventListener('input', () => {
    const value = modalInput.value.toLowerCase()
    document.querySelectorAll<HTMLAnchorElement>('.command-result').forEach((row) =>
      row.classList.toggle('hidden', !row.textContent?.toLowerCase().includes(value)),
    )
  })
  document.querySelectorAll<HTMLAnchorElement>('.command-result').forEach((row) =>
    row.addEventListener('click', () => closeModal()),
  )

  mountThemeToggle()

  document.querySelector('.mobile-menu')?.addEventListener('click', () => document.querySelector('.main-nav')?.classList.toggle('open'))

  // Delegated, because filtering and search re-render the grid.
  document.querySelector('#component-grid')?.addEventListener('click', async (event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>('.copy-button')
    if (!button) return
    event.preventDefault()
    event.stopPropagation()
    await navigator.clipboard?.writeText(`cargo mkit add ${button.dataset.copy}`)
    button.innerHTML = icon('check', 14)
    button.classList.add('copied')
    setTimeout(() => {
      button.innerHTML = icon('copy', 14)
      button.classList.remove('copied')
    }, 1300)
  })

  return () => document.removeEventListener('keydown', onKeydown)
}
