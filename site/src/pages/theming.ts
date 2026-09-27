import { theming } from '../generated/components'
import { themeOrder, themes } from '../generated/theme'
import type { ThemeTokens } from '../generated/theme'
import { icon } from '../icons'
import { docsSidebar, docsTopbar } from './docs'

const base = import.meta.env.BASE_URL || '/'
const bookBlob = 'https://github.com/mk7s/mkit/blob/main/book/src'

const defaultTheme = 'dark'

const colorRoles = [
  ['background', 'Window behind everything'],
  ['surface', 'A panel or card'],
  ['elevated_surface', 'A raised surface, like a menu'],
  ['text', 'Primary text and icons'],
  ['text_muted', 'Secondary labels'],
  ['border', 'Dividers and outlines'],
  ['accent', 'Primary action or selection'],
  ['accent_text', 'Text on accent'],
  ['focus', 'Keyboard focus cue'],
  ['success', 'Positive state'],
  ['warning', 'Caution'],
  ['danger', 'Destructive or error'],
  ['disabled', 'Unavailable control'],
] as const

const cssVars = () => {
  const rules = themeOrder.map((key) => {
    const theme = themes[key]
    const declarations: string[] = []
    for (const [role, value] of Object.entries(theme.colors)) {
      declarations.push(`--t-color-${role.replace(/_/g, '-')}: ${value};`)
    }
    for (const [token, value] of Object.entries(theme.typography)) {
      declarations.push(`--t-type-${token.replace(/_/g, '-')}: ${value}px;`)
    }
    for (const [token, value] of Object.entries(theme.spacing)) {
      declarations.push(`--t-space-${token}: ${value}px;`)
    }
    for (const [token, value] of Object.entries(theme.radii)) {
      declarations.push(`--t-radius-${token}: ${value}px;`)
    }
    for (const [token, value] of Object.entries(theme.borders)) {
      declarations.push(`--t-border-${token}: ${value}px;`)
    }
    for (const [token, value] of Object.entries(theme.controls)) {
      declarations.push(`--t-control-${token}: ${value}px;`)
    }
    for (const [token, value] of Object.entries(theme.shadows)) {
      declarations.push(`--t-shadow-${token}: 0 ${value.y}px ${value.blur}px ${value.color};`)
    }
    for (const [token, value] of Object.entries(theme.motion)) {
      declarations.push(`--t-motion-${token.replace(/_ms$/, '')}: ${value}ms;`)
    }
    return `.theme-board[data-theme="${key}"] { ${declarations.join(' ')} }`
  })
  return `<style>${rules.join('\n')}</style>`
}

const formatToken = (theme: ThemeTokens, group: string, key: string) => {
  const value = (theme as unknown as Record<string, Record<string, unknown>>)[group]?.[key]
  if (value === undefined) return ''
  if (typeof value === 'number') {
    if (group === 'motion') return `${value} ms`
    if (group === 'typography' || group === 'spacing' || group === 'radii' || group === 'borders' || group === 'controls') {
      return `${value} px`
    }
    return String(value)
  }
  if (typeof value === 'string') return value
  if (value && typeof value === 'object') {
    const shadow = value as { y: number; blur: number; color: string }
    return `0 ${shadow.y}px ${shadow.blur}px ${shadow.color}`
  }
  return ''
}

const tokenValue = (group: string, key: string, label: string) =>
  `<span class="token-value" data-token="${group}.${key}">${label}</span>`

function colorSwatches() {
  const initial = themes[defaultTheme]
  return colorRoles
    .map(([role, note]) => {
      const value = initial.colors[role]
      return `<div class="token-chip">
        <span class="token-chip-color" data-color-role="${role}" style="background: var(--t-color-${role.replace(/_/g, '-')})"></span>
        <span class="token-chip-name">${role.replace(/_/g, ' ')}</span>
        <span class="token-chip-note">${note}</span>
        <span class="token-value" data-token="colors.${role}">${value}</span>
      </div>`
    })
    .join('')
}

function tokenBoard() {
  const initial = themes[defaultTheme]
  const typeRows = Object.entries(initial.typography)
    .map(
      ([token, size]) => `<div class="token-row">
        <span class="token-row-name">${token.replace(/_/g, ' ')}</span>
        <span class="type-sample" style="font-size: var(--t-type-${token.replace(/_/g, '-')})">Ag</span>
        ${tokenValue('typography', token, `${size} px`)}
      </div>`,
    )
    .join('')
  const spacingRows = Object.entries(initial.spacing)
    .map(
      ([token, size]) => `<div class="token-row">
        <span class="token-row-name">${token}</span>
        <span class="spacing-bar" style="width: var(--t-space-${token})"></span>
        ${tokenValue('spacing', token, `${size} px`)}
      </div>`,
    )
    .join('')
  const radiusRows = Object.entries(initial.radii)
    .map(
      ([token, size]) => `<div class="token-row">
        <span class="token-row-name">${token}</span>
        <span class="radius-box" style="border-radius: var(--t-radius-${token})"></span>
        ${tokenValue('radii', token, token === 'pill' ? `${size} px` : `${size} px`)}
      </div>`,
    )
    .join('')
  const controlRows = Object.entries(initial.controls)
    .map(
      ([token, size]) => `<div class="token-row">
        <span class="token-row-name">${token}</span>
        <span class="control-bar" style="height: var(--t-control-${token})"></span>
        ${tokenValue('controls', token, `${size} px`)}
      </div>`,
    )
    .join('')
  const borderRows = Object.entries(initial.borders)
    .map(
      ([token, size]) => `<div class="token-row">
        <span class="token-row-name">${token}</span>
        <span class="border-sample" style="border-top-width: var(--t-border-${token})"></span>
        ${tokenValue('borders', token, `${size} px`)}
      </div>`,
    )
    .join('')
  const shadowRows = Object.entries(initial.shadows)
    .map(
      ([token, shadow]) => `<div class="token-row">
        <span class="token-row-name">${token}</span>
        <span class="shadow-sample" style="box-shadow: var(--t-shadow-${token})"></span>
        ${tokenValue('shadows', token, formatToken(initial, 'shadows', token) || shadow.color)}
      </div>`,
    )
    .join('')
  const motionRows = Object.entries(initial.motion)
    .map(
      ([token, duration]) => `<div class="token-row">
        <span class="token-row-name">${token.replace(/_ms$/, '')}</span>
        <span class="motion-sample"><i style="transition-duration: var(--t-motion-${token.replace(/_ms$/, '')})"></i></span>
        ${tokenValue('motion', token, `${duration} ms`)}
      </div>`,
    )
    .join('')

  return `<section class="theme-board-section" id="theming--token-board">
    ${cssVars()}
    <div class="docs-section-head">
      <span class="docs-kicker">Live token board</span>
      <h2>Try the built-in themes</h2>
      <p>These values are read from <code>crates/mkit-core/src/theme.rs</code> at build time. Switch a theme to preview the same tokens a component reads at runtime.</p>
    </div>
    <div class="theme-board" data-theme="${defaultTheme}">
      <div class="theme-switcher" role="tablist" aria-label="Built-in theme">
        ${themeOrder
          .map(
            (key) =>
              `<button class="theme-option${key === defaultTheme ? ' active' : ''}" role="tab" aria-selected="${key === defaultTheme}" data-theme-key="${key}">${themes[key].label}</button>`,
          )
          .join('')}
      </div>
      <div class="theme-board-body">
        <h3 class="theme-board-title">Color roles</h3>
        <div class="token-chip-grid">${colorSwatches()}</div>
        <div class="token-columns">
          <div class="token-panel"><h3 class="theme-board-title">Type scale</h3>${typeRows}</div>
          <div class="token-panel"><h3 class="theme-board-title">Spacing</h3>${spacingRows}</div>
          <div class="token-panel"><h3 class="theme-board-title">Radii</h3>${radiusRows}</div>
          <div class="token-panel"><h3 class="theme-board-title">Controls</h3>${controlRows}</div>
          <div class="token-panel"><h3 class="theme-board-title">Borders</h3>${borderRows}</div>
          <div class="token-panel"><h3 class="theme-board-title">Shadows</h3>${shadowRows}</div>
          <div class="token-panel"><h3 class="theme-board-title">Motion</h3>${motionRows}</div>
        </div>
        <div class="theme-applied">
          <span class="theme-applied-label">Applied</span>
          <div class="theme-card">
            <div class="theme-card-head">
              <span class="theme-card-dot"></span>
              <span>Render settings</span>
            </div>
            <p>Components read these tokens; the theme decides the values.</p>
            <div class="theme-card-actions">
              <button class="theme-mock-button primary">Apply</button>
              <button class="theme-mock-button">Cancel</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>`
}

function chapterSections() {
  return theming.sections
    .map(
      (section) => `<section class="doc-section" id="${section.id}">
        <h3>${section.title}</h3>
        ${section.html}
      </section>`,
    )
    .join('')
}

function themingToc() {
  const rows = [
    `<a class="docs-toc-link" href="#theming--token-board">Live token board</a>`,
    ...theming.sections.map(
      (section) => `<a class="docs-toc-link" href="#${section.id}">${section.title}</a>`,
    ),
  ].join('')
  return `<aside class="docs-toc" aria-label="On this page">
    <span class="docs-toc-heading">On this page</span>
    ${rows}
  </aside>`
}

export function renderTheming() {
  return `<div class="docs-shell">
    ${docsTopbar('E9 · Design language')}
    <div class="docs-layout">
      ${docsSidebar('theming')}
      <main class="docs-main" id="top">
        <nav class="docs-breadcrumb" aria-label="Breadcrumb">
          <a href="${base}">Home</a><span>/</span><span>Theming</span><span>/</span><b>Design language</b>
        </nav>
        <header class="docs-header">
          <span class="docs-kicker">E9.1 · Theming &amp; design language</span>
          <h1>${theming.title}</h1>
          <p class="docs-lede">${theming.lede}</p>
          <div class="docs-callout">${icon('book', 15)}<span>E9.2 (a redistributable icon set) and E9.3 (theme authoring) are not shipped yet. Token values below are generated from the library source.</span></div>
          <a class="doc-source" href="${bookBlob}/${theming.sourcePath}" target="_blank" rel="noreferrer">Read the book chapter ${icon('external', 13)}</a>
        </header>
        <article class="docs-article">
          ${theming.intro ? `<div class="doc-intro">${theming.intro}</div>` : ''}
          ${tokenBoard()}
          ${chapterSections()}
        </article>
      </main>
      ${themingToc()}
    </div>
  </div>`
}

export function mountTheming() {
  const board = document.querySelector<HTMLElement>('.theme-board')
  if (!board) return
  const setTheme = (key: string) => {
    board.dataset.theme = key
    const theme = themes[key]
    if (!theme) return
    document.querySelectorAll<HTMLButtonElement>('.theme-option').forEach((button) => {
      const active = button.dataset.themeKey === key
      button.classList.toggle('active', active)
      button.setAttribute('aria-selected', String(active))
    })
    board.querySelectorAll<HTMLElement>('[data-token]').forEach((element) => {
      const [group, token] = (element.dataset.token ?? '').split('.')
      const value = formatToken(theme, group, token)
      if (value) element.textContent = value
    })
  }
  document.querySelectorAll<HTMLButtonElement>('.theme-option').forEach((button) =>
    button.addEventListener('click', () => setTheme(button.dataset.themeKey ?? defaultTheme)),
  )
}
