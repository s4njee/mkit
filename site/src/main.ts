import './style.css'
import './ui/ui.css'
import { cardBySlug } from './catalog'
import { mountDocs, renderDocs, renderNotFound } from './pages/docs'
import { mountHealth, renderHealth } from './pages/health'
import { mountHome, renderHome } from './pages/home'
import { mountTheming, renderTheming } from './pages/theming'
import { installLinkInterceptor, navigate, onPopState, resolveRoute } from './router'
import { installSearch } from './search'
import { mountThemeToggle } from './theme'

const app = document.querySelector<HTMLDivElement>('#app')!

let cleanup: (() => void) | undefined
let renderedPath = ''

const prefersReducedMotion = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches

function page(): { html: string; title: string; mount: () => (() => void) | void } {
  const route = resolveRoute(location.pathname)
  if (route.name === 'theming') {
    return { html: renderTheming(), title: 'The mkit design language · mkit', mount: mountTheming }
  }
  if (route.name === 'health') {
    return { html: renderHealth(), title: 'Book health · mkit', mount: mountHealth }
  }
  if (route.name === 'component') {
    const card = cardBySlug.get(route.slug)
    if (card) return { html: renderDocs(card), title: `${card.name} · mkit`, mount: mountDocs }
  }
  if (route.name === 'home') {
    return { html: renderHome(), title: 'mkit: Rust Components backed by GPUI', mount: mountHome }
  }
  return { html: renderNotFound(location.pathname), title: 'Not found · mkit', mount: () => {} }
}

function render(options: { moveFocus?: boolean } = {}) {
  // Same-page hash changes only scroll; re-rendering would reset previews.
  if (location.pathname === renderedPath) {
    scrollToHash(true)
    return
  }
  cleanup?.()
  cleanup = undefined
  const next = page()
  app.innerHTML = next.html
  document.title = next.title
  renderedPath = location.pathname
  cleanup = next.mount() || undefined
  mountThemeToggle()
  scrollToHash()
  // After client-side navigation, move focus to the new page's main region
  // so screen reader and keyboard users start at the content.
  if (options.moveFocus && !location.hash) document.querySelector<HTMLElement>('#main')?.focus({ preventScroll: true })
}

function scrollToHash(smooth = false) {
  const target = location.hash ? document.getElementById(decodeURIComponent(location.hash.slice(1))) : null
  const behavior: ScrollBehavior = smooth && !prefersReducedMotion() ? 'smooth' : 'instant'
  if (target) requestAnimationFrame(() => target.scrollIntoView({ behavior }))
  else window.scrollTo({ top: 0, behavior: 'instant' })
}

installSearch()
installLinkInterceptor((href) => {
  navigate(href)
  render({ moveFocus: true })
})
onPopState(() => render())
render()
