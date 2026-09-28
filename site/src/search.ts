import { allCards, kindLabel } from './catalog'
import { icon } from './icons'
import { siteHref } from './links'

// Site-wide search palette (⌘K / Ctrl+K, or `/`). It follows the WAI-ARIA
// combobox pattern: the input owns focus, arrow keys move the active option,
// and Enter opens it.

type Entry = {
  id: string
  label: string
  detail: string
  href: string
  group: 'Pages' | 'Components'
  icon: string
  haystack: string
}

const slugify = (value: string) =>
  value
    .toLowerCase()
    .replace(/&/g, ' and ')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

const humanize = (name: string) => {
  const text = name.replace(/-/g, ' ')
  return text.charAt(0).toUpperCase() + text.slice(1)
}

const escapeHtml = (value: string) =>
  value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')

function buildEntries(): Entry[] {
  const entries: Entry[] = [
    { label: 'Component catalog', detail: 'All components', href: siteHref('#components'), icon: 'grid' },
    { label: 'Design language', detail: 'Theming · tokens', href: siteHref('theming'), icon: 'layers' },
    { label: 'Book health', detail: 'Checks and coverage', href: siteHref('health'), icon: 'book' },
  ].map((page) => ({
    ...page,
    id: `page-${slugify(page.label)}`,
    group: 'Pages' as const,
    haystack: `${page.label} ${page.detail}`.toLowerCase(),
  }))

  const seen = new Set(entries.map((entry) => entry.label.toLowerCase()))
  const add = (entry: Omit<Entry, 'id' | 'group' | 'haystack'>, extra: string) => {
    const key = entry.label.toLowerCase()
    if (seen.has(key)) return
    seen.add(key)
    entries.push({
      ...entry,
      id: `result-${entries.length}`,
      group: 'Components',
      haystack: `${entry.label} ${entry.detail} ${extra}`.toLowerCase(),
    })
  }

  for (const card of allCards) {
    const kind = kindLabel(card.kind)
    const names = card.components.flatMap((doc) => [doc.name, ...doc.registryNames])
    add(
      {
        label: card.name,
        detail: kind,
        href: siteHref(`components/${card.slug}`),
        icon: card.kind === 'pro' ? 'layers' : 'grid',
      },
      `${card.description} ${card.tags.join(' ')} ${names.join(' ')}`,
    )
    for (const doc of card.components) {
      const anchor = `#component-${slugify(doc.name)}`
      const detail = `${card.name} · ${kind}`
      if (card.components.length > 1 || doc.name.toLowerCase() !== card.name.toLowerCase()) {
        add(
          { label: doc.name, detail, href: siteHref(`components/${card.slug}${anchor}`), icon: 'terminal' },
          `${doc.registryNames.join(' ')} ${card.tags.join(' ')}`,
        )
      }
      // Registry entries documented inside a shared chapter, such as Accordion.
      for (const name of doc.registryNames) {
        add(
          { label: humanize(name), detail, href: siteHref(`components/${card.slug}${anchor}`), icon: 'terminal' },
          `${name} ${doc.name}`,
        )
      }
    }
  }
  return entries
}

const entries = buildEntries()

function matches(query: string) {
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean)
  if (!terms.length) return entries
  const scored = entries
    .filter((entry) => terms.every((term) => entry.haystack.includes(term)))
    .map((entry) => {
      const label = entry.label.toLowerCase()
      const first = terms[0]
      const score = label.startsWith(first) ? 0 : label.includes(first) ? 1 : 2
      return { entry, score }
    })
  return scored.sort((a, b) => a.score - b.score).map(({ entry }) => entry)
}

function modalHtml() {
  return `<div class="command-modal" id="command-modal">
    <div class="command-backdrop" data-close-search></div>
    <div class="command-dialog" role="dialog" aria-modal="true" aria-labelledby="command-title">
      <h2 class="sr-only" id="command-title">Search the docs</h2>
      <div class="command-input">${icon('search', 18)}
        <input type="text" role="combobox" aria-expanded="true" aria-controls="command-list" aria-autocomplete="list" aria-label="Search components and pages" placeholder="Search components and pages…" autocomplete="off" spellcheck="false" />
        <button type="button" class="command-close" data-close-search aria-label="Close search">Esc</button>
      </div>
      <div class="command-results" id="command-list" role="listbox" aria-label="Results"></div>
      <div class="command-footer" aria-hidden="true"><span>↑↓ navigate</span><span>↵ open</span><span>esc close</span></div>
      <p class="sr-only" id="command-status" aria-live="polite"></p>
    </div>
  </div>`
}

let installed = false

/** Install the palette once; any element with `data-open-search` opens it. */
export function installSearch() {
  if (installed) return
  installed = true
  document.body.insertAdjacentHTML('beforeend', modalHtml())
  const modal = document.getElementById('command-modal')!
  const input = modal.querySelector<HTMLInputElement>('input')!
  const list = modal.querySelector<HTMLDivElement>('#command-list')!
  const status = modal.querySelector<HTMLParagraphElement>('#command-status')!
  let results: Entry[] = entries
  let active = 0
  let returnFocus: HTMLElement | null = null

  const paint = () => {
    if (!results.length) {
      list.innerHTML = `<p class="command-empty">No results. Try a component name such as “slider” or “accordion”.</p>`
      input.removeAttribute('aria-activedescendant')
      status.textContent = 'No results'
      return
    }
    let group = ''
    list.innerHTML = results
      .map((entry, index) => {
        const heading = entry.group !== group ? `<div class="command-group-label" role="presentation">${entry.group}</div>` : ''
        group = entry.group
        return `${heading}<a class="command-result${index === active ? ' active' : ''}" role="option" id="${entry.id}" aria-selected="${index === active}" href="${entry.href}" tabindex="-1" data-index="${index}">
          <span class="result-icon">${icon(entry.icon, 15)}</span>
          <span class="result-text">${escapeHtml(entry.label)}<small>${escapeHtml(entry.detail)}</small></span>
          <kbd aria-hidden="true">↵</kbd>
        </a>`
      })
      .join('')
    input.setAttribute('aria-activedescendant', results[active].id)
    status.textContent = `${results.length} result${results.length === 1 ? '' : 's'}`
  }

  const setActive = (index: number) => {
    if (!results.length) return
    active = (index + results.length) % results.length
    list.querySelectorAll<HTMLAnchorElement>('.command-result').forEach((row, i) => {
      const on = i === active
      row.classList.toggle('active', on)
      row.setAttribute('aria-selected', String(on))
      if (on) row.scrollIntoView({ block: 'nearest' })
    })
    input.setAttribute('aria-activedescendant', results[active].id)
  }

  const open = () => {
    if (modal.classList.contains('open')) return
    returnFocus = document.activeElement as HTMLElement | null
    modal.classList.add('open')
    document.body.classList.add('search-open')
    input.value = ''
    results = entries
    active = 0
    paint()
    input.focus()
  }

  const close = (restoreFocus = true) => {
    if (!modal.classList.contains('open')) return
    modal.classList.remove('open')
    document.body.classList.remove('search-open')
    if (restoreFocus && returnFocus?.isConnected) returnFocus.focus()
  }

  input.addEventListener('input', () => {
    results = matches(input.value)
    active = 0
    paint()
  })

  input.addEventListener('keydown', (event) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      setActive(active + 1)
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      setActive(active - 1)
    } else if (event.key === 'Home' && results.length) {
      event.preventDefault()
      setActive(0)
    } else if (event.key === 'End' && results.length) {
      event.preventDefault()
      setActive(results.length - 1)
    } else if (event.key === 'Enter') {
      event.preventDefault()
      list.querySelector<HTMLAnchorElement>(`[data-index="${active}"]`)?.click()
    }
  })

  // Keep Tab inside the dialog: the input and the close button.
  modal.addEventListener('keydown', (event) => {
    if (event.key !== 'Tab') return
    const close = modal.querySelector<HTMLButtonElement>('.command-close')!
    const focusables = [input, close]
    const index = focusables.indexOf(document.activeElement as HTMLInputElement)
    event.preventDefault()
    focusables[(index + (event.shiftKey ? -1 : 1) + focusables.length) % focusables.length].focus()
  })

  list.addEventListener('mousemove', (event) => {
    const row = (event.target as HTMLElement).closest<HTMLAnchorElement>('.command-result')
    if (row && Number(row.dataset.index) !== active) setActive(Number(row.dataset.index))
  })
  // The router's link interceptor handles navigation; just close first.
  list.addEventListener('click', (event) => {
    if ((event.target as HTMLElement).closest('.command-result')) close(false)
  })
  modal.querySelectorAll('[data-close-search]').forEach((element) => element.addEventListener('click', () => close()))

  document.addEventListener('click', (event) => {
    const trigger = (event.target as HTMLElement).closest('[data-open-search]')
    if (!trigger) return
    event.preventDefault()
    open()
  })

  document.addEventListener('keydown', (event) => {
    const target = event.target as HTMLElement
    const typing = target.closest('input, textarea, select, [contenteditable="true"]')
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault()
      if (modal.classList.contains('open')) close()
      else open()
    } else if (event.key === '/' && !typing && !modal.classList.contains('open')) {
      event.preventDefault()
      open()
    } else if (event.key === 'Escape' && modal.classList.contains('open')) {
      event.preventDefault()
      close()
    }
  })
}
